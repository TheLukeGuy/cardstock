package sh.lpx.cardstock.registry.packet.client;

import org.jetbrains.annotations.NotNull;
import org.jetbrains.annotations.Nullable;
import sh.lpx.cardstock.registry.packet.PacketByteBuf;

public record ClientSelectPluginPacket(@NotNull String name, @Nullable String authors)
    implements ClientPacket
{
    @Override
    public int id() {
        return 0x01;
    }

    @Override
    public void write(@NotNull PacketByteBuf buf) {
        buf.writeString(this.name);
        buf.writeOptional(this.authors, PacketByteBuf::writeString);
    }
}
