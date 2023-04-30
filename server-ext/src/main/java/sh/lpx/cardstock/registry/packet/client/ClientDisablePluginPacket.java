package sh.lpx.cardstock.registry.packet.client;

public record ClientDisablePluginPacket()
    implements ClientPacket
{
    @Override
    public int id() {
        return 0x03;
    }
}
