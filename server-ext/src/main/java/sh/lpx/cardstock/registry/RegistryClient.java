package sh.lpx.cardstock.registry;

import org.jetbrains.annotations.NotNull;
import org.jetbrains.annotations.Nullable;
import sh.lpx.cardstock.registry.packet.PacketByteBuf;
import sh.lpx.cardstock.registry.packet.client.ClientHandshakePacket;
import sh.lpx.cardstock.registry.packet.client.ClientPacket;

import java.io.*;
import java.net.Socket;

public class RegistryClient
    implements Closeable
{
    private final Socket socket;

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

    @Override
    public void close()
        throws IOException
    {
        this.socket.close();
    }
}
