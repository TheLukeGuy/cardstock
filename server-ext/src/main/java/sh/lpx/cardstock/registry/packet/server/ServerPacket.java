package sh.lpx.cardstock.registry.packet.server;

import org.jetbrains.annotations.NotNull;
import sh.lpx.cardstock.registry.packet.PacketByteBuf;

public interface ServerPacket {
    static @NotNull ServerPacket read(int id, @NotNull PacketByteBuf buf) {
        return switch (id) {
            case 0x00 -> new ServerHandshakePacket();
            default -> throw new IllegalArgumentException(String.format("The packet ID is invalid. (0x%02x)", id));
        };
    }
}
