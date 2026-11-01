//! Role: Tests for the editor's time ruler, zoom range and shortest clip.
//!
//! The editor's time ruler: tick spacing, labels, the zoom range and the
//! shortest clip, through public APIs only.

use super::*;

const CANDIDATES: [f64; 6] = [1.0, 2.0, 5.0, 10.0, 30.0, 60.0];

type Asked = Rc<RefCell<Vec<f64>>>;

/// Clear the log, settle, and return the times the ruler asked labels for
/// since, ascending. Change something the ruler draws from first.
fn labelled(screen: &Screen, asked: &Asked) -> Vec<f64> {
    asked.borrow_mut().clear();
    drawn(screen, asked)
}

/// The times asked since the log was last cleared, ascending.
fn drawn(screen: &Screen, asked: &Asked) -> Vec<f64> {
    screen.run_until_idle();
    screen.screenshot();
    let mut seen = asked.borrow().clone();
    seen.sort_by(f64::total_cmp);
    seen.dedup();
    seen
}

/// The distance between neighbouring labels, which must all be equal.
fn interval(times: &[f64]) -> f64 {
    assert!(times.len() >= 3, "{times:?}");
    let step = times[1] - times[0];
    for pair in times.windows(2) {
        assert_eq!(pair[1] - pair[0], step, "{times:?}");
    }
    step
}

fn ctrl_wheel(screen: &Screen, canvas: &kivact::test::Element<'_>, dy: f32, times: usize) {
    let ctrl = Modifiers {
        ctrl: true,
        ..Modifiers::default()
    };
    canvas.focus();
    screen.key_down(Key::Unidentified, KeyCode::ControlLeft, ctrl);
    for _ in 0..times {
        canvas.scroll(0.0, dy);
    }
    screen.key_up(
        Key::Unidentified,
        KeyCode::ControlLeft,
        Modifiers::default(),
    );
}

fn logging(asked: &Asked) -> impl Fn(f64) -> String + 'static {
    let asked = asked.clone();
    move |time| {
        asked.borrow_mut().push(time);
        format!("{time}")
    }
}

#[test]
fn candidates_change_with_zoom_and_grow_tenfold_past_the_last() {
    let asked = Asked::default();
    let format = logging(&asked);
    let (screen, zoom) = kivact::test::render_with_handle(crate::options(), move || {
        let zoom = signal(1.0);
        (
            zoom,
            app_dope_sheet(
                signal(Vec::new()),
                ui::TimeState {
                    zoom: zoom.into(),
                    ..ui::TimeState::new()
                },
            )
            .lanes(vec![ui::Lane::new(0, "Opacity")])
            .zoom_range(0.01, 8.0)
            .time_format(format)
            .ruler_ticks(ui::RulerTicks::new(CANDIDATES, 4))
            .style(style().size(600, 160)),
        )
    });
    for (z, expected) in [
        (8.0, 10.0),
        (4.0, 30.0),
        (1.0, 60.0),
        (0.5, 600.0),
        (0.05, 6000.0),
    ] {
        zoom.set(z);
        let times = labelled(&screen, &asked);
        let step = interval(&times);
        // 56 px apart at least; past the last candidate the interval is
        // that candidate times 10, then times 100.
        assert_eq!(step, expected, "zoom {z}: {times:?}");
        assert!(step * z >= 56.0, "zoom {z}: {step}");
    }
    clean(&screen);
}

#[test]
fn spacing_and_the_default_candidates_choose_the_interval() {
    // No valid candidate falls back to 1, 2, 5, 10, 20, 50...: 56 px at
    // zoom 1 needs 100.
    let asked = Asked::default();
    let format = logging(&asked);
    let (screen, ()) = kivact::test::render_with_handle(crate::options(), move || {
        (
            (),
            app_timeline(signal(Vec::new()), ui::TimeState::new())
                .lanes(vec![ui::Lane::new(0, "Video")])
                .time_format(format)
                .ruler_ticks(ui::RulerTicks::new([f64::NAN, -1.0, 0.0], 1))
                .style(style().size(800, 200)),
        )
    });
    assert_eq!(interval(&drawn(&screen, &asked)), 100.0);
    clean(&screen);

    // The default candidates and spacing: also 100.
    let asked = Asked::default();
    let format = logging(&asked);
    let (screen, ()) = kivact::test::render_with_handle(crate::options(), move || {
        (
            (),
            app_timeline(signal(Vec::new()), ui::TimeState::new())
                .lanes(vec![ui::Lane::new(0, "Video")])
                .time_format(format)
                .style(style().size(800, 200)),
        )
    });
    assert_eq!(interval(&drawn(&screen, &asked)), 100.0);
    clean(&screen);

    // A spacing under 8 px is raised to 8: 10 is the first fit.
    let asked = Asked::default();
    let format = logging(&asked);
    let (screen, ()) = kivact::test::render_with_handle(crate::options(), move || {
        (
            (),
            app_timeline(signal(Vec::new()), ui::TimeState::new())
                .lanes(vec![ui::Lane::new(0, "Video")])
                .time_format(format)
                .ruler_ticks(ui::RulerTicks::new(CANDIDATES, 4).min_spacing(1.0))
                .style(style().size(800, 200)),
        )
    });
    assert_eq!(interval(&drawn(&screen, &asked)), 10.0);
    clean(&screen);
}

#[test]
fn labels_are_the_time_format_result_and_keep_their_distance() {
    let asked = Asked::default();
    let log = asked.clone();
    let (screen, (zoom, prefix)) = kivact::test::render_with_handle(crate::options(), move || {
        let zoom = signal(1.0);
        let prefix = signal(String::from("T"));
        (
            (zoom, prefix),
            app_dope_sheet(
                signal(Vec::new()),
                ui::TimeState {
                    zoom: zoom.into(),
                    ..ui::TimeState::new()
                },
            )
            .lanes(vec![ui::Lane::new(0, "Opacity")])
            .zoom_range(0.05, 8.0)
            .time_format(move |frame| {
                log.borrow_mut().push(frame);
                let seconds = (frame / 30.0).floor() as i64;
                format!(
                    "{}{:02}:{:02}:{:02}",
                    prefix.get(),
                    seconds / 3600,
                    seconds / 60 % 60,
                    seconds % 60
                )
            })
            .ruler_ticks(ui::RulerTicks::new(
                [30.0, 60.0, 150.0, 300.0, 900.0, 1800.0],
                4,
            ))
            .style(style().size(700, 160)),
        )
    });
    let with_t = screen.screenshot().hash();
    prefix.set("U".into());
    screen.run_until_idle();
    assert_ne!(
        screen.screenshot().hash(),
        with_t,
        "the label text comes from time_format"
    );
    prefix.set("T".into());
    screen.run_until_idle();
    assert_eq!(screen.screenshot().hash(), with_t);

    // A label "T00:00:00" is under 56 px wide at 9 px text, so labels whose
    // ticks are 56 px or more apart cannot overlap.
    for z in [8.0, 4.0, 2.0, 1.0, 0.5, 0.2, 0.05] {
        zoom.set(z);
        let times = labelled(&screen, &asked);
        let apart = interval(&times) * z;
        assert!(apart >= 56.0, "zoom {z}: {apart} px apart");
    }
    clean(&screen);
}

#[test]
fn ctrl_wheel_and_the_zoom_binding_stay_inside_the_zoom_range() {
    let (screen, zoom) = kivact::test::render_with_handle(crate::options(), || {
        let zoom = signal(1.0);
        (
            zoom,
            app_timeline(
                signal(Vec::new()),
                ui::TimeState {
                    zoom: zoom.into(),
                    ..ui::TimeState::new()
                },
            )
            .name("Ranged")
            .lanes(vec![ui::Lane::new(0, "Video")])
            .zoom_range(0.5, 8.0)
            .style(style().size(600, 160)),
        )
    });
    let canvas = screen.get_by_role(Role::Canvas, "Ranged");
    ctrl_wheel(&screen, &canvas, -1.0, 40);
    assert_eq!(zoom.get_untracked(), 8.0, "stops at the maximum");
    ctrl_wheel(&screen, &canvas, 1.0, 80);
    assert_eq!(zoom.get_untracked(), 0.5, "and at the minimum");

    // A bound zoom outside the range is shown and used at the bound.
    zoom.set(0.1);
    screen.run_until_idle();
    assert!(
        canvas.value().is_some_and(|v| v.ends_with("zoom 50%")),
        "{:?}",
        canvas.value()
    );
    zoom.set(20.0);
    screen.run_until_idle();
    assert!(
        canvas.value().is_some_and(|v| v.ends_with("zoom 800%")),
        "{:?}",
        canvas.value()
    );
    clean(&screen);
}

#[test]
fn the_default_zoom_range_is_a_quarter_to_four() {
    let (screen, zoom) = kivact::test::render_with_handle(crate::options(), || {
        let zoom = signal(1.0);
        (
            zoom,
            app_timeline(
                signal(Vec::new()),
                ui::TimeState {
                    zoom: zoom.into(),
                    ..ui::TimeState::new()
                },
            )
            .name("Default")
            .lanes(vec![ui::Lane::new(0, "Video")])
            .style(style().size(600, 160)),
        )
    });
    let canvas = screen.get_by_role(Role::Canvas, "Default");
    ctrl_wheel(&screen, &canvas, -1.0, 40);
    assert_eq!(zoom.get_untracked(), 4.0);
    ctrl_wheel(&screen, &canvas, 1.0, 80);
    assert_eq!(zoom.get_untracked(), 0.25);
    clean(&screen);
}

#[test]
fn invalid_zoom_ranges_are_reported_and_keep_the_default() {
    let (screen, zoom) = kivact::test::render_with_handle(crate::options(), || {
        let zoom = signal(1.0);
        (
            zoom,
            app_timeline(
                signal(Vec::new()),
                ui::TimeState {
                    zoom: zoom.into(),
                    ..ui::TimeState::new()
                },
            )
            .name("Invalid")
            .lanes(vec![ui::Lane::new(0, "Video")])
            .zoom_range(f64::NAN, 2.0)
            .zoom_range(0.0, 2.0)
            .zoom_range(3.0, 2.0)
            .zoom_range(0.5, f64::INFINITY)
            .style(style().size(600, 160)),
        )
    });
    assert_eq!(screen.errors().len(), 4, "{:?}", screen.errors());
    let canvas = screen.get_by_role(Role::Canvas, "Invalid");
    ctrl_wheel(&screen, &canvas, -1.0, 40);
    assert_eq!(zoom.get_untracked(), 4.0, "the default maximum");
    ctrl_wheel(&screen, &canvas, 1.0, 80);
    assert_eq!(zoom.get_untracked(), 0.25, "the default minimum");
}

fn clip_editor(min: Option<f64>) -> (Screen, Signal<Vec<ui::Clip>>) {
    kivact::test::render_with_handle(crate::options(), move || {
        let data = signal(vec![
            ui::Clip::new(1, 0, 30.0, 100.0, "Clip").color(rgb(0x8b7cff)),
        ]);
        let editor = app_timeline(
            data,
            ui::TimeState {
                selection: (signal(vec![1])).into(),
                ..ui::TimeState::new()
            },
        );
        let editor = match min {
            Some(min) => editor.min_duration(min),
            None => editor,
        };
        (data, editor)
    })
}

#[test]
fn trimming_never_shortens_a_clip_below_min_duration() {
    let (screen, data) = clip_editor(Some(40.0));
    screen
        .get_by_role(Role::Button, "Clip end trim")
        .drag(Point::new(329.0, 40.0), Point::new(129.0, 40.0));
    assert_eq!(data.get_untracked()[0].duration, 40.0);
    // Already at the least length: the start edge cannot move in.
    screen
        .get_by_role(Role::Button, "Clip start trim")
        .drag(Point::new(231.0, 40.0), Point::new(331.0, 40.0));
    assert_eq!(data.get_untracked()[0].start, 30.0);
    assert_eq!(data.get_untracked()[0].duration, 40.0);
    clean(&screen);

    let (screen, data) = clip_editor(Some(40.0));
    screen
        .get_by_role(Role::Button, "Clip start trim")
        .drag(Point::new(231.0, 40.0), Point::new(431.0, 40.0));
    assert_eq!(data.get_untracked()[0].start, 90.0);
    assert_eq!(data.get_untracked()[0].duration, 40.0);
    clean(&screen);
}

#[test]
fn the_default_least_clip_length_is_one_unit() {
    let (screen, data) = clip_editor(None);
    screen
        .get_by_role(Role::Button, "Clip end trim")
        .drag(Point::new(329.0, 40.0), Point::new(129.0, 40.0));
    assert_eq!(data.get_untracked()[0].duration, 1.0);
    clean(&screen);
}

#[test]
fn the_playhead_badge_and_value_use_the_time_format() {
    let (screen, shown) = kivact::test::render_with_handle(crate::options(), || {
        let shown = signal(None::<String>);
        (
            shown,
            app_timeline(
                signal(Vec::new()),
                ui::TimeState {
                    playhead: (signal(45.0)).into(),
                    ..ui::TimeState::new()
                },
            )
            .lanes(vec![ui::Lane::new(0, "Video")])
            .time_format(move |time| match shown.get() {
                // Only the playhead's time differs, so the ruler stays put.
                Some(text) if time == 45.0 => text,
                _ => format!("{time}"),
            })
            .style(style().size(600, 160)),
        )
    });
    let marker = screen.get_by_role(Role::Slider, "Playhead");
    assert_eq!(marker.value().as_deref(), Some("45"));
    let plain = screen.screenshot().hash();
    shown.set(Some("00:01:15".into()));
    screen.run_until_idle();
    assert_eq!(marker.value().as_deref(), Some("00:01:15"));
    assert_eq!(
        marker.value_range().2,
        Some(45.0),
        "the number stays logical"
    );
    assert_ne!(
        screen.screenshot().hash(),
        plain,
        "the badge shows the formatted time"
    );
    shown.set(None);
    screen.run_until_idle();
    assert_eq!(screen.screenshot().hash(), plain);
    clean(&screen);
}

#[test]
fn playhead_advertises_set_value_and_writes_the_binding() {
    let (screen, frame) = kivact::test::render_with_handle(crate::options(), || {
        let frame = signal(10.0);
        (
            frame,
            app_timeline(
                signal(Vec::new()),
                ui::TimeState {
                    playhead: frame.into(),
                    ..ui::TimeState::new()
                },
            ),
        )
    });
    let marker = screen.get_by_role(Role::Slider, "Playhead");
    assert!(marker.a11y_actions().contains(&A11yAction::SetValue));
    marker.a11y_with_value(A11yAction::SetValue, "42");
    assert_eq!(frame.get_untracked(), 42.0);
    clean(&screen);
}

#[test]
fn selected_item_drag_preserves_the_group_but_click_selects_one() {
    let (screen, (items, selection)) = kivact::test::render_with_handle(crate::options(), || {
        let items = signal(
            (1..=3)
                .map(|id| {
                    ui::Clip::new(
                        id,
                        ((id - 1) as usize) as u64,
                        30.0,
                        100.0,
                        format!("Clip {id}"),
                    )
                    .color(rgb(0x8b7cff))
                })
                .collect::<Vec<_>>(),
        );
        let selection = signal(vec![1, 2, 3]);
        (
            (items, selection),
            app_timeline(
                items,
                ui::TimeState {
                    selection: selection.into(),
                    ..ui::TimeState::new()
                },
            )
            .style(style().size(600, 220)),
        )
    });
    let clip = screen.get_by_role(Role::Button, "Clip 2");
    let center = clip.rect().expect("selected clip").center();
    clip.drag(center, Point::new(center.x + 20.0, center.y));
    assert_eq!(selection.get_untracked(), vec![1, 2, 3]);
    assert!(items.get_untracked().iter().all(|item| item.start == 50.0));
    clip.a11y(A11yAction::Activate);
    assert_eq!(selection.get_untracked(), vec![1, 2, 3]);
    clip.a11y(A11yAction::Select);
    assert_eq!(selection.get_untracked(), vec![2]);
    selection.set(vec![1, 2, 3]);
    clip.click();
    assert_eq!(selection.get_untracked(), vec![2]);
    clean(&screen);
}

#[test]
fn reactive_lanes_can_hide_every_item_and_readding_retires_old_locators() {
    let (screen, lanes) = kivact::test::render_with_handle(crate::options(), || {
        let lanes = signal(vec![ui::Lane::new(u64::MAX, "Track")]);
        (
            lanes,
            app_timeline(
                signal(vec![ui::Clip::new(
                    u64::MAX,
                    u64::MAX,
                    10.0,
                    100.0,
                    "Retained clip",
                )]),
                ui::TimeState::new(),
            )
            .lanes(lanes)
            .style(style().size(600, 200)),
        )
    });
    let old = screen.get_by_role(Role::Button, "Retained clip");
    lanes.set(Vec::new());
    screen.run_until_idle();
    assert_eq!(old.name(), "");
    assert!(
        screen
            .query_by_role(Role::Button, "Retained clip")
            .is_none()
    );
    lanes.set(vec![ui::Lane::new(u64::MAX, "Track")]);
    screen.run_until_idle();
    assert_eq!(old.name(), "");
    assert!(
        screen
            .get_by_role(Role::Button, "Retained clip")
            .is_visible()
    );
    clean(&screen);
}
