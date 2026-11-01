// Misuse: `signal(50)` holds an `i32`; a slider edits an `f64`.
use kivact::prelude::*;

fn volume() -> impl IntoView {
    slider("Volume", signal(50))
}

fn main() {
    let _ = volume;
}
