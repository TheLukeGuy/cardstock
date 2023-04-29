package sh.lpx.cardstock.registry.packet.client;

import org.jetbrains.annotations.NotNull;
import sh.lpx.cardstock.registry.packet.PacketByteBuf;

public record ClientDisablePluginPacket(@NotNull String name)
    implements ClientPacket
{
    @Override
    public int id() {
        return 0x03;
    }

    @Override
    public void write(@NotNull PacketByteBuf buf) {
        buf.writeString(this.name);
    }
}
