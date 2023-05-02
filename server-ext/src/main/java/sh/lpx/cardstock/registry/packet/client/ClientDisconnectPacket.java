package sh.lpx.cardstock.registry.packet.client;

public record ClientDisconnectPacket()
    implements ClientPacket
{
    @Override
    public int id() {
        return 0x05;
    }
}
