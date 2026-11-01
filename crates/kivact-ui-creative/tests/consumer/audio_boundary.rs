//! Role: Application-owned routing and levels and arbitrary fader ranges.
use super::*;

#[test]
fn mixer_routing_is_bound_and_meter_uses_supplied_level() {
    let (screen, (gain, mute, route)) = kivact::test::render_with_handle(crate::options(), || {
        let gain = signal(-30.0);
        let mute = signal(true);
        let route = signal(0_usize);
        (
            (gain, mute, route),
            ui::mixer_strip(
                "Channel",
                rgb(0x4080a0),
                ui::ChannelState {
                    gain: gain.into(),
                    mute: mute.into(),
                    ..ui::ChannelState::new()
                },
            )
            .gain_range(-60.0, 6.0)
            .level(0.75)
            .routes(
                ["Bus A", "Bus B"]
                    .into_iter()
                    .map(|item| item.to_string())
                    .collect::<Vec<_>>(),
                route,
            ),
        )
    });
    let fader = screen.get_by_role(Role::Slider, "Volume");
    assert_eq!(fader.value_range(), (Some(-60.0), Some(6.0), Some(-30.0)));
    fader.a11y_with_value(A11yAction::SetValue, "-12");
    assert_eq!(gain.get_untracked(), -12.0);
    let peak = screen.get_by_role(Role::Meter, "Peak");
    assert_eq!(peak.value_range().2, Some(0.75));
    gain.set(6.0);
    mute.set(false);
    screen.run_until_idle();
    assert_eq!(peak.value_range().2, Some(0.75));
    let selector = screen.get_by_role(Role::ComboBox, "Input routing");
    selector.click();
    screen.get_by_role(Role::Option, "Bus B").click();
    assert_eq!(route.get_untracked(), 1);
    route.set(0);
    screen.run_until_idle();
    assert_eq!(selector.value().as_deref(), Some("Bus A"));
    clean(&screen);
    let (screen, ()) = kivact::test::render_with_handle(crate::options(), || {
        (
            (),
            row((
                ui::mixer_strip("Empty", rgb(0x4080a0), ui::ChannelState::new()),
                ui::mixer_strip("No routes", rgb(0x4080a0), ui::ChannelState::new())
                    .routes(Vec::<String>::new(), signal(0)),
            )),
        )
    });
    assert!(
        screen
            .query_by_role(Role::ComboBox, "Input routing")
            .is_none()
    );
    assert!(screen.query_by_role(Role::Button, "EQ-4").is_none());
    clean(&screen);
}

#[test]
fn fader_maps_arbitrary_range_with_square_root_and_square_inverse() {
    let (screen, value) = kivact::test::render_with_handle(crate::options(), || {
        let value = signal(25.0);
        (
            value,
            ui::fader("Position", value)
                .range(10.0, 70.0)
                .step(0.001)
                .precision(3)
                .style(style().size(48, 200)),
        )
    });
    let fader = screen.get_by_role(Role::Slider, "Position");
    // 25 is one quarter of the configured range and starts halfway up the track.
    // Dragging another quarter of the full extent reaches n=.75, t=.5625.
    fader.drag(Point::new(24.0, 100.0), Point::new(24.0, 50.0));
    assert!(
        (value.get_untracked() - 43.75).abs() < 0.01,
        "{}",
        value.get_untracked()
    );
    fader.a11y_with_value(A11yAction::SetValue, "70");
    assert_eq!(fader.value_range().2, Some(70.0));
    clean(&screen);
}

#[test]
fn meter_draws_the_supplied_level_equally_in_every_channel() {
    let (screen, ()) = kivact::test::render_with_handle(crate::options(), || {
        (
            (),
            ui::meter("Channels", 0.8)
                .channels(4)
                .style(style().size(120, 180)),
        )
    });
    let rect = screen
        .get_by_role(Role::Meter, "Channels")
        .rect()
        .expect("meter");
    let shot = screen.screenshot();
    let y = (rect.y + 48.0) as u32;
    let first = shot.pixel((rect.x + 16.0) as u32, y);
    for x in [45.0, 74.0, 103.0] {
        assert_eq!(shot.pixel((rect.x + x) as u32, y), first);
    }
    clean(&screen);
}

#[test]
fn transport_reports_buttons_and_start_without_a_clock() {
    let (screen, (state, starts)) = kivact::test::render_with_handle(crate::options(), || {
        let state = signal(ui::TransportState::default());
        let starts = signal(0);
        (
            (state, starts),
            ui::transport(state).on_to_start(move || starts.update(|n| *n += 1)),
        )
    });
    screen.get_by_role(Role::Button, "Play").click();
    assert!(state.get_untracked().playing);
    screen.advance(Duration::from_millis(500));
    assert!(state.get_untracked().playing);
    screen.get_by_role(Role::Button, "Record").click();
    assert!(state.get_untracked().recording);
    screen.get_by_role(Role::Button, "Cycle").click();
    assert!(!state.get_untracked().cycling);
    screen.get_by_role(Role::Button, "To Start").click();
    assert_eq!(starts.get_untracked(), 1);
    screen.get_by_role(Role::Button, "Stop").click();
    assert!(!state.get_untracked().playing);
    clean(&screen);
}
