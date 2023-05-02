package sh.lpx.cardstock.registry;

import org.bukkit.Server;
import org.jetbrains.annotations.NotNull;
import org.jetbrains.annotations.Nullable;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import sh.lpx.cardstock.Cardstock;
import sh.lpx.cardstock.registry.packet.PacketByteBuf;
import sh.lpx.cardstock.registry.packet.PartialPacket;
import sh.lpx.cardstock.registry.packet.client.ClientDisconnectPacket;
import sh.lpx.cardstock.registry.packet.client.ClientHandshakePacket;
import sh.lpx.cardstock.registry.packet.client.ClientPacket;
import sh.lpx.cardstock.registry.packet.server.*;

import java.io.*;
import java.net.Socket;
import java.util.Optional;
import java.util.concurrent.ArrayBlockingQueue;
import java.util.concurrent.BlockingQueue;

public class RegistryClient
    implements AutoCloseable
{
    private static final int ERROR_TOLERANCE = 5;
    private static final boolean ERROR_TOLERANCE_SET = ERROR_TOLERANCE >= 0;

    private final Logger logger = LoggerFactory.getLogger(RegistryClient.class);
    private final Server server;

    private final Socket socket;
    private boolean didHandshake = false;
    private boolean shutDown = false;

    private final RegisterResponse registerResponse = new RegisterResponse();
    private final BlockingQueue<RegisterResponse.Complete> registerResponseQueue = new ArrayBlockingQueue<>(1);

    private final InputStream inputStream;
    private final OutputStream outputStream;

    private RegistryClient(
        @NotNull Server server,
        @NotNull Socket socket,
        @NotNull InputStream inputStream,
        @NotNull OutputStream outputStream
    ) {
        this.server = server;
        this.socket = socket;
        this.inputStream = inputStream;
        this.outputStream = outputStream;
    }

    public static @NotNull RegistryClient connect(
        @NotNull String addr,
        @Nullable ClientHandshakePacket handshake,
        @NotNull Server server
    ) throws IOException {
        if (!addr.contains(":")) {
            throw new IllegalArgumentException("The address is in an invalid format.");
        }

        int separatorIndex = addr.lastIndexOf(":");
        String host;
        int port;
        if (separatorIndex != -1) {
            host = addr.substring(0, separatorIndex);
            port = Integer.parseInt(addr.substring(separatorIndex + 1));
        } else {
            host = null;
            port = Integer.parseInt(addr);
        }

        Socket socket = new Socket(host, port);
        RegistryClient client = new RegistryClient(server, socket, socket.getInputStream(), socket.getOutputStream());
        if (handshake != null) {
            client.sendPacket(handshake);
        }
        return client;
    }

    public void run() {
        if (this.shutDown) {
            return;
        }

        PartialPacket partial = new PartialPacket();
        int errors = 0;
        while (true) {
            try {
                int nextByte;
                try {
                    nextByte = this.inputStream.read();
                    if (nextByte == -1) {
                        throw new EOFException();
                    }
                } catch (IOException e) {
                    break;
                }

                Optional<ServerPacket> next = partial.next(nextByte);
                if (next.isPresent() && this.actOnPacket(next.get()) == PacketHandleResult.DISCONNECT) {
                    break;
                }
                if (ERROR_TOLERANCE_SET) {
                    errors = 0;
                }
            } catch (Exception e) {
                this.logger.warn("Failed to handle a packet.", e);
                if (ERROR_TOLERANCE_SET) {
                    if (errors++ == ERROR_TOLERANCE) {
                        this.logger.error("Failed to handle too many packets.");
                        break;
                    }
                }
            }
        }

        if (!this.shutDown) {
            Cardstock.LOGGER.error("We're no longer connected to the registry server; aborting.");
            this.shutDown = true;
            this.server.shutdown();
        }
    }

    private @NotNull PacketHandleResult actOnPacket(@NotNull ServerPacket packet) {
        switch (packet) {
            case ServerHandshakePacket handshakePacket -> {
                if (handshakePacket.adsEnabled()) {
                    Cardstock.LOGGER.warn(
                        "Your configured registry server will send you ads. "
                            + "These ads are not officially endorsed by Cardstock or any plugin."
                    );
                }
                this.didHandshake = true;
            }
            case ServerPacket ignored && !this.didHandshake ->
                throw new IllegalStateException("Received a non-handshake packet before handshake.");
            case ServerMsgPacket msgPacket -> this.registerResponse.addMsg(msgPacket.logFn(), msgPacket.contents());
            case ServerDenyPacket ignored -> this.registerResponse.setDenied();
            case ServerDonePacket ignored -> this.registerResponseQueue.add(this.registerResponse.reset());
            case ServerDisconnectPacket ignored -> {
                Cardstock.LOGGER.error("The registry server has disconnected us.");
                return PacketHandleResult.DISCONNECT;
            }
            default -> this.logger.warn("Ignoring packet: {}", packet);
        }
        return PacketHandleResult.OK;
    }

    private enum PacketHandleResult {
        OK,
        DISCONNECT,
    }

    public void sendPacket(@NotNull ClientPacket packet)
        throws IOException
    {
        if (this.shutDown) {
            return;
        }

        PacketByteBuf buf = PacketByteBuf.allocateDefault(3);
        buf.writePacket(packet);
        try {
            buf.writeToOtherFromBeginning(bytes -> {
                try {
                    this.outputStream.write(bytes);
                } catch (IOException e) {
                    throw new UncheckedIOException(e);
                }
            });
        } catch (UncheckedIOException e) {
            throw e.getCause();
        }
    }

    public RegisterResponse.@NotNull Complete takeRegisterResponse() {
        if (this.shutDown) {
            return new RegisterResponse.Complete(false, new RegisterResponse.Msg[0]);
        }
        while (true) {
            try {
                return this.registerResponseQueue.take();
            } catch (InterruptedException e) {
                // Continue looping
            }
        }
    }

    @Override
    public void close()
        throws IOException
    {
        try {
            this.sendPacket(new ClientDisconnectPacket());
        } catch (IOException e) {
            Cardstock.LOGGER.warn("Failed to gracefully disconnect from the server.");
        }
        this.shutDown = true;
        this.socket.close();
    }
}
