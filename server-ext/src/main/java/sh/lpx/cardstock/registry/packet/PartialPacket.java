package sh.lpx.cardstock.registry.packet;

import org.jetbrains.annotations.NotNull;
import sh.lpx.cardstock.registry.packet.server.ServerPacket;

import java.util.Optional;

public class PartialPacket {
    private Integer firstLenByte;
    private Integer id;

    private byte[] packet;
    private int cursor;

    public PartialPacket() {
        this.reset();
    }

    public @NotNull Optional<@NotNull ServerPacket> next(int b) {
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
                ServerPacket packet = ServerPacket.read(id, PacketByteBuf.wrap(this.packet));
                this.reset();
                return Optional.of(packet);
            }
            return Optional.empty();
        }

        this.packet[this.cursor++] = (byte) b;
        if (this.cursor == this.packet.length) {
            ServerPacket packet = ServerPacket.read(id, PacketByteBuf.wrap(this.packet));
            this.reset();
            return Optional.of(packet);
        }
        return Optional.empty();
    }

    private void reset() {
        this.firstLenByte = null;
        this.id = null;
        this.packet = null;
        this.cursor = 0;
    }
}
