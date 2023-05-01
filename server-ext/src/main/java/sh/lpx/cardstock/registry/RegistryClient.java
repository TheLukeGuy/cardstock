package sh.lpx.cardstock.registry;

import org.jetbrains.annotations.NotNull;
import org.jetbrains.annotations.Nullable;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import sh.lpx.cardstock.Cardstock;
import sh.lpx.cardstock.registry.packet.PacketByteBuf;
import sh.lpx.cardstock.registry.packet.PartialPacket;
import sh.lpx.cardstock.registry.packet.client.ClientHandshakePacket;
import sh.lpx.cardstock.registry.packet.client.ClientPacket;
import sh.lpx.cardstock.registry.packet.server.*;

import java.io.*;
import java.net.Socket;
import java.util.concurrent.ArrayBlockingQueue;
import java.util.concurrent.BlockingQueue;

public class RegistryClient
    implements Closeable
{
    private final Logger logger = LoggerFactory.getLogger(RegistryClient.class);

    private final Socket socket;
    private boolean didHandshake = false;

    private final RegisterResponse registerResponse = new RegisterResponse();
    private final BlockingQueue<RegisterResponse.Complete> registerResponseQueue = new ArrayBlockingQueue<>(1);

    private final InputStream inputStream;
    private final OutputStream outputStream;

    private RegistryClient(
        @NotNull Socket socket,
        @NotNull InputStream inputStream,
        @NotNull OutputStream outputStream
    ) {
        this.socket = socket;
        this.inputStream = inputStream;
        this.outputStream = outputStream;
    }

    public static @NotNull RegistryClient connect(@NotNull String addr, @Nullable ClientHandshakePacket handshake)
        throws IOException
    {
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
        RegistryClient client = new RegistryClient(socket, socket.getInputStream(), socket.getOutputStream());
        if (handshake != null) {
            client.sendPacket(handshake);
        }
        return client;
    }

    @SuppressWarnings("InfiniteLoopStatement")
    public void run() {
        while (true) {
            try {
                this.nextPacket();
            } catch (Exception e) {
                this.logger.warn("Failed to handle a packet.", e);
                // Cardstock TODO: Stop trying if there are too many consecutive failed packets
            }
        }
    }

    @SuppressWarnings("InfiniteLoopStatement")
    private void nextPacket()
        throws IOException
    {
        PartialPacket partial = new PartialPacket();
        while (true) {
            partial.next(this.inputStream.read()).ifPresent(this::actOnPacket);
        }
    }

    private void actOnPacket(@NotNull ServerPacket packet) {
        switch (packet) {
            case ServerHandshakePacket ignored -> {
                if (ignored.adsEnabled()) {
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
            default -> this.logger.warn("Ignoring packet: {}", packet);
        }
    }

    public void sendPacket(@NotNull ClientPacket packet)
        throws IOException
    {
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
        this.socket.close();
    }
}
