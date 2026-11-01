// Usage: an action's input type may be decided by the call to `run`.
use kivact::prelude::*;

fn echo() -> impl IntoView {
    let echo = action(|value| async move { Ok(value) });
    let reload = action(|| async { Ok(()) });
    button("Echo").on_press(move || {
        echo.run(42_u32);
        reload.run(());
    })
}

fn main() {
    let _ = echo;
}
