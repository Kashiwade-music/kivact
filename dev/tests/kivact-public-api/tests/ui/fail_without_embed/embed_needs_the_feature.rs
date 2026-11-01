use kivact::embed::Parent;
use kivact::prelude::*;

fn main() {
    let _ = App::new("t").embed(Parent::x11(1), || text("Editor"));
}
