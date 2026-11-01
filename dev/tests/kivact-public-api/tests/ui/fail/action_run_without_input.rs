// Misuse: an action without input still takes its input, `()`.
use kivact::prelude::*;

fn reload() -> impl IntoView {
    let refresh = action(|| async { Ok(()) });
    button("Reload").on_press(move || refresh.run())
}

fn main() {
    let _ = reload;
}
