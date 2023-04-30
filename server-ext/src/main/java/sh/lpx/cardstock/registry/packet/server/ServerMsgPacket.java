package sh.lpx.cardstock.registry.packet.server;

import org.jetbrains.annotations.NotNull;
import org.slf4j.Logger;

import java.util.function.BiConsumer;

public record ServerMsgPacket(@NotNull BiConsumer<@NotNull Logger, String> logFn, @NotNull String contents)
    implements ServerPacket {}
