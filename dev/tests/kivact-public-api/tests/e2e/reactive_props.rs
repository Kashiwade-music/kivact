//! Role: Verify reactive standard-control props preserve the mounted control.
use crate::support::mount;
use kivact::interaction::{RangeThumb, ScalarRange, use_range_scalar, use_scalar};
use kivact::prelude::*;
use kivact::ui::ControlSize;

#[test]
fn reactive_component_props_update_without_remounting() {
    let (screen, (size, step, value, low, high)) = mount(|| {
        let size = signal(ControlSize::Small);
        let step = signal(1.0);
        let value = signal(10.0);
        let low = signal(20.0);
        let high = signal(80.0);
        let range = ScalarRange::new(0.0, 100.0).unwrap();
        let scalar = use_scalar(value, range).key_step(step).unwrap();
        let ends = use_range_scalar(low, high, range).key_step(step).unwrap();
        (
            (size, step, value, low, high),
            column((
                button("Resize").size(size).test_id("resize"),
                node().part(scalar.thumb()).name("Scalar").test_id("scalar"),
                node()
                    .part(ends.thumb(RangeThumb::Low))
                    .name("Low")
                    .test_id("low"),
                node()
                    .part(ends.thumb(RangeThumb::High))
                    .name("High")
                    .test_id("high"),
            )),
        )
    });
    let button = screen.get_by_test_id("resize");
    button.focus();
    let before = button.rect().unwrap();
    size.set(ControlSize::Large);
    screen.run_until_idle();
    let after = button.rect().unwrap();
    assert!(after.width > before.width || after.height > before.height);
    assert!(button.is_focused());
    screen.get_by_test_id("scalar").press_key(Key::ArrowRight);
    screen.get_by_test_id("low").press_key(Key::ArrowRight);
    screen.get_by_test_id("high").press_key(Key::ArrowLeft);
    assert_eq!((value.get(), low.get(), high.get()), (11.0, 21.0, 79.0));
    step.set(5.0);
    screen.run_until_idle();
    screen.get_by_test_id("scalar").press_key(Key::ArrowRight);
    screen.get_by_test_id("low").press_key(Key::ArrowRight);
    screen.get_by_test_id("high").press_key(Key::ArrowLeft);
    assert_eq!((value.get(), low.get(), high.get()), (16.0, 26.0, 74.0));
    assert!(screen.errors().is_empty());
}
