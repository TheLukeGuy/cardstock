package sh.lpx.cardstock.registry.packet.client;

import org.jetbrains.annotations.NotNull;
import sh.lpx.cardstock.registry.packet.PacketByteBuf;

public interface ClientPacket {
    int id();

    default void write(@NotNull PacketByteBuf buf) {}
}
