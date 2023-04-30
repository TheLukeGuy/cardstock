package sh.lpx.cardstock.registry.packet.client;

import org.jetbrains.annotations.NotNull;
import sh.lpx.cardstock.registry.packet.PacketByteBuf;

public record ClientRegisterCmdPacket(@NotNull String name)
    implements ClientPacket
{
    @Override
    public int id() {
        return 0x04;
    }

    @Override
    public void write(@NotNull PacketByteBuf buf) {
        buf.writeString(this.name);
    }
}
