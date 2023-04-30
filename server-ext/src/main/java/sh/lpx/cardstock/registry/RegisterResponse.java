package sh.lpx.cardstock.registry;

import org.jetbrains.annotations.NotNull;

import java.util.ArrayList;
import java.util.List;

public class RegisterResponse {
    private boolean denied = false;
    private final List<String> msgs = new ArrayList<>();

    public void setDenied() {
        this.denied = true;
    }

    public void addMsg(@NotNull String msg) {
        this.msgs.add(msg);
    }

    public @NotNull Complete reset() {
        Complete complete = new Complete(this.denied, this.msgs.toArray(new String[0]));
        this.denied = false;
        this.msgs.clear();
        return complete;
    }

    public record Complete(boolean denied, @NotNull String @NotNull [] msgs) {}
}
