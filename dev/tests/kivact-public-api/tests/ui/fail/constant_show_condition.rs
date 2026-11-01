// Misuse: a constant condition for `show` never changes; pass the signal or a closure.
use kivact::prelude::*;

fn panel(open: Signal<bool>) -> impl IntoView {
    show(open.get(), || text("Details"))
}

fn main() {
    let _ = panel;
}
