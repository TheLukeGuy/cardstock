package sh.lpx.cardstock.registry.packet.client;

public record ClientEnablePluginPacket()
    implements ClientPacket
{
    @Override
    public int id() {
        return 0x02;
    }
}
