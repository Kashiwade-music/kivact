// Misuse: a check box edits its binding; `checked` is the component's own state,
// so it is not a modifier of the prelude (`NodeExt` is for component authors).
use kivact::prelude::*;

fn terms() -> impl IntoView {
    checkbox("Terms", signal(false)).checked(true)
}

fn main() {
    let _ = terms;
}
