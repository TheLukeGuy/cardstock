package sh.lpx.cardstock.registry.packet;

import org.jetbrains.annotations.NotNull;
import org.jetbrains.annotations.Nullable;
import sh.lpx.cardstock.registry.packet.client.ClientPacket;
import sh.lpx.cardstock.registry.packet.server.ServerPacket;

import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.nio.charset.StandardCharsets;
import java.util.Arrays;
import java.util.function.Consumer;

@SuppressWarnings("unused")
public class PacketByteBuf {
    private final ByteBuffer buf;

    private PacketByteBuf(@NotNull ByteBuffer buf) {
        this.buf = buf;
    }

    public static PacketByteBuf allocateDefault() {
        return allocateDefault(0);
    }

    public static PacketByteBuf allocateDefault(int additional) {
        return allocate(1024 + additional);
    }

    public static PacketByteBuf allocate(int capacity) {
        return wrap(ByteBuffer.allocate(capacity));
    }

    public static PacketByteBuf wrap(@NotNull ByteBuffer buf) {
        buf.order(ByteOrder.BIG_ENDIAN);
        return new PacketByteBuf(buf);
    }

    public static PacketByteBuf wrap(byte @NotNull [] bytes) {
        return wrap(ByteBuffer.wrap(bytes));
    }

    public void writeToOtherFromBeginning(@NotNull Consumer<byte[]> writer) {
        this.writeToOtherFromBeginning(null, writer);
    }

    public void writeToOtherFromBeginning(@Nullable Consumer<Integer> useLen, @NotNull Consumer<byte[]> writer) {
        if (!this.buf.hasArray()) {
            throw new IllegalStateException("The buffer isn't backed by an array.");
        }

        this.buf.rewind();
        int len = this.buf.remaining();
        if (useLen != null) {
            useLen.accept(len);
        }

        if (!this.buf.hasRemaining()) {
            return;
        }
        byte[] backing = this.buf.array();
        int startPos = this.buf.position();
        int startIndex = this.buf.arrayOffset() + this.buf.position();

        writer.accept(Arrays.copyOfRange(backing, startIndex, startIndex + len));
        this.buf.position(startPos + len);
    }

    public @NotNull ServerPacket readPacket() {
        int len = this.readUnsignedShort();
        int id = this.readUnsignedByte();

        byte[] payload = new byte[len];
        this.readExact(payload);
        return ServerPacket.read(id, wrap(payload));
    }

    public void writePacket(@NotNull ClientPacket packet) {
        PacketByteBuf buf = allocateDefault();
        packet.write(buf);
        buf.writeToOtherFromBeginning(
            len -> {
                this.writeUnsignedShort(len);
                this.writeUnsignedByte(packet.id());
            },
            this::writeAll
        );
    }

    public String readString() {
        int len = this.readUnsignedShort();
        byte[] buf = new byte[len];
        this.readExact(buf);
        return new String(buf, StandardCharsets.UTF_8);
    }

    public void writeString(@NotNull String s) {
        this.writeUnsignedShort(s.length());
        byte[] bytes = s.getBytes(StandardCharsets.UTF_8);
        this.writeAll(bytes);
    }

    public void readExact(byte @NotNull [] buf) {
        this.buf.get(buf);
    }

    public void writeAll(byte @NotNull [] buf) {
        this.buf.put(buf);
    }

    public byte readSignedByte() {
        return this.buf.get();
    }

    public void writeSignedByte(byte b) {
        this.buf.put(b);
    }

    public int readUnsignedByte() {
        return Byte.toUnsignedInt(this.buf.get());
    }

    public void writeUnsignedByte(int b) {
        this.buf.put((byte) b);
    }

    public short readSignedShort() {
        return this.buf.getShort();
    }

    public void writeSignedShort(short s) {
        this.buf.putShort(s);
    }

    public int readUnsignedShort() {
        return Short.toUnsignedInt(this.buf.getShort());
    }

    public void writeUnsignedShort(int s) {
        this.buf.putShort((short) s);
    }

    public int readSignedInt() {
        return this.buf.getInt();
    }

    public void writeSignedInt(int i) {
        this.buf.putInt(i);
    }

    public long readUnsignedInt() {
        return Integer.toUnsignedLong(this.buf.getInt());
    }

    public void writeUnsignedInt(long i) {
        this.buf.putInt((int) i);
    }

    public long readSignedLong() {
        return this.buf.getLong();
    }

    public void writeSignedLong(long l) {
        this.buf.putLong(l);
    }
}
