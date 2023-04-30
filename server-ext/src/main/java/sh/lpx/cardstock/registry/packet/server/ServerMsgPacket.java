package sh.lpx.cardstock.registry.packet.server;

import org.jetbrains.annotations.NotNull;

public record ServerMsgPacket(@NotNull String msg)
    implements ServerPacket {}
