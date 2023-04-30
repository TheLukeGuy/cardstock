package sh.lpx.cardstock.registry.packet.server;

public record ServerHandshakePacket(boolean adsEnabled)
    implements ServerPacket {}
