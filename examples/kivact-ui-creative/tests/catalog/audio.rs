//! Role: Catalog playback clock and application-side mixer DSP interaction evidence.
use super::{evidence, mount};
use kivact::view::{A11yAction, Role};
use std::time::Duration;

#[test]
fn creative_catalog_owns_playback_and_mixer_dsp() {
    let screen = mount();
    let toolbar = screen.get_by_role(Role::Toolbar, "Creative toolbar");
    let frame = screen.get_by_test_id("catalog-playback-frame");
    let value = || {
        frame
            .value()
            .expect("frame")
            .parse::<f64>()
            .expect("numeric frame")
    };
    toolbar.get_by_role(Role::Button, "Play").click();
    screen.advance(Duration::from_millis(500));
    assert!(value() > 0.0);
    toolbar.get_by_role(Role::Button, "Stop").click();
    let stopped = value();
    screen.advance(Duration::from_millis(500));
    assert_eq!(value(), stopped);
    toolbar.get_by_role(Role::Button, "To Start").click();
    assert_eq!(value(), 0.0);
    screen.get_by_test_id("creative-nav-transport").click();
    screen.advance(Duration::from_millis(400));
    screen
        .get_by_test_id("creative-section-5")
        .get_by_role(Role::Button, "To End")
        .click();
    assert_eq!(value(), 192.0);
    toolbar.get_by_role(Role::Button, "Play").click();
    screen.advance(Duration::from_millis(500));
    assert!(
        value() > 0.0 && value() < 192.0,
        "catalog loop wraps the clock"
    );
    toolbar.get_by_role(Role::Button, "Stop").click();
    evidence(&screen, "creative-transport-application-clock");
    screen.get_by_test_id("creative-nav-mixer").click();
    screen.advance(Duration::from_millis(400));
    let channel = screen.get_by_role(Role::Group, "Kick");
    let meter = channel.get_by_role(Role::Meter, "Peak");
    let level = meter.value_range().2.expect("meter");
    assert!(level > 0.0);
    channel.get_by_role(Role::Button, "M").click();
    assert_eq!(meter.value_range().2, Some(0.0));
    channel.get_by_role(Role::Button, "M").click();
    channel
        .get_by_role(Role::Slider, "Volume")
        .a11y_with_value(A11yAction::SetValue, "-30");
    assert!(meter.value_range().2.expect("gain level") < level / 10.0);
    let route = channel.get_by_role(Role::ComboBox, "Input routing");
    route.click();
    screen.get_by_role(Role::Option, "Main Out 1-2").click();
    assert_eq!(route.value().as_deref(), Some("Main Out 1-2"));
    evidence(&screen, "creative-mixer-application-dsp-routing");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
