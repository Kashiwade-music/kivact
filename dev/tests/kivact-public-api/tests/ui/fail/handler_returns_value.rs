// A handler returns `()` or `Result<(), UiError>`, not another value.
use kivact::prelude::*;

fn main() {
    let _ = node().on_press(|| 5);
}
