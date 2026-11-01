// Without the experimental feature `gpu-interop`, `kivact::gpu_interop`
// does not exist (nor `App::gpu_preference`).
use kivact::gpu_interop::adapter;
use kivact::prelude::*;

fn main() {
    let _ = adapter;
    let _ = App::new("t").gpu_preference(Default::default());
}
