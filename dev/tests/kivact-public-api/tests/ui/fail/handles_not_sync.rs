// Every public handle is !Sync.
use kivact::{prelude::*, Binding, Callback, task::Task};

fn assert_sync<T: Sync>() {}

fn main() {
    assert_sync::<Signal<i32>>();
    assert_sync::<Value<i32>>();
    assert_sync::<Var<i32>>();
    assert_sync::<Callback<i32>>();
    assert_sync::<Binding<i32>>();
    assert_sync::<kivact::Selector<u64>>();
    assert_sync::<NodeRef>();
    assert_sync::<Resource<i32>>();
    assert_sync::<Action<i32, i32>>();
    assert_sync::<Task>();
    assert_sync::<kivact::motion::FrameLoop>();
    assert_sync::<kivact::motion::Animated<f32>>();
}
