// A value that is not a view cannot be a child.
use kivact::prelude::*;

struct Score(u32);

fn main() {
    let _ = column(Score(1));
}
