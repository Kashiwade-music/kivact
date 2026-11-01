// Misuse: a `Signal<i32>` where a `String` property is taken; format it.
use kivact::prelude::*;

fn count(n: Signal<i32>) -> Node {
    text(n)
}

fn main() {
    let _ = count;
}
