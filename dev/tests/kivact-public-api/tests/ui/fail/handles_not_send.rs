// Every public handle is !Send.
use kivact::{prelude::*, Binding, Callback, task::Task};

fn assert_send<T: Send>() {}

fn main() {
    assert_send::<Signal<i32>>();
    assert_send::<Value<i32>>();
    assert_send::<Var<i32>>();
    assert_send::<Callback<i32>>();
    assert_send::<Binding<i32>>();
    assert_send::<kivact::Selector<u64>>();
    assert_send::<NodeRef>();
    assert_send::<Resource<i32>>();
    assert_send::<Action<i32, i32>>();
    assert_send::<Task>();
    assert_send::<kivact::motion::FrameLoop>();
    assert_send::<kivact::motion::Animated<f32>>();
}
