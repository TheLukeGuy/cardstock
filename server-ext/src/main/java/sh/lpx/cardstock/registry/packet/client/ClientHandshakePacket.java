package sh.lpx.cardstock.registry.packet.client;

import org.jetbrains.annotations.NotNull;
import sh.lpx.cardstock.registry.packet.PacketByteBuf;

public record ClientHandshakePacket(@NotNull String version)
    implements ClientPacket
{
    @Override
    public int id() {
        return 0x00;
    }

    @Override
    public void write(@NotNull PacketByteBuf buf) {
        buf.writeString(this.version);
    }
}
