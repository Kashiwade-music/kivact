// Misuse: a non-Copy prop moved into a factory that runs more than once.
// Clone it inside the factory (or turn it into a `Value` first).
use kivact::prelude::*;

fn greeting(name: String, shown: Signal<bool>) -> impl IntoView {
    show(shown, move || text(name))
}

fn main() {
    let _ = greeting;
}
