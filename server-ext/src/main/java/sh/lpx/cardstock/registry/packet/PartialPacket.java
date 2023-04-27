package sh.lpx.cardstock.registry.packet;

import org.jetbrains.annotations.NotNull;

import java.nio.ByteBuffer;
import java.util.Optional;

public class PartialPacket {
    private Byte firstLenByte = null;
    private Byte id = null;

    private byte[] packet = null;
    private int cursor = 0;

    public @NotNull Optional<Complete> next(byte b) {
        if (this.firstLenByte == null) {
            this.firstLenByte = b;
            return Optional.empty();
        }

        if (this.packet == null) {
            int len = (this.firstLenByte << 16) | b;
            this.packet = new byte[len];
            return Optional.empty();
        }

        if (this.id == null) {
            this.id = b;
            if (this.packet.length == 0) {
                Complete complete = new Complete(this.id, ByteBuffer.wrap(this.packet));
                return Optional.of(complete);
            }
            return Optional.empty();
        }

        this.packet[this.cursor++] = b;
        if (this.cursor == this.packet.length) {
            Complete complete = new Complete(this.id, ByteBuffer.wrap(this.packet));
            return Optional.of(complete);
        }
        return Optional.empty();
    }

    public record Complete(byte id, @NotNull ByteBuffer packet) {}
}
