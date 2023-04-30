package sh.lpx.cardstock.registry;

import org.jetbrains.annotations.NotNull;
import org.slf4j.Logger;

import java.util.ArrayList;
import java.util.List;
import java.util.function.BiConsumer;

public class RegisterResponse {
    private boolean denied = false;
    private final List<Msg> msgs = new ArrayList<>();

    public void setDenied() {
        this.denied = true;
    }

    public void addMsg(@NotNull BiConsumer<@NotNull Logger, String> logFn, @NotNull String contents) {
        this.msgs.add(new Msg(logFn, contents));
    }

    public @NotNull Complete reset() {
        Complete complete = new Complete(this.denied, this.msgs.toArray(new Msg[0]));
        this.denied = false;
        this.msgs.clear();
        return complete;
    }

    public record Msg(@NotNull BiConsumer<@NotNull Logger, String> logFn, @NotNull String contents) {}

    public record Complete(boolean denied, @NotNull Msg @NotNull [] msgs) {}
}
