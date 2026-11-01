//! Role: Tests the radio group, slider, toast region and splitter of `kivact::ui`.
//!
//! On real windows through the public API: their roles, names and values,
//! the keys, the pointer and assistive technology that edit them, the edits
//! they report (`EditPhase`) and the value a canceled drag writes back, and
//! their layout and keys in a right-to-left subtree.

use std::cell::RefCell;
use std::rc::Rc;

use crate::support::mount_with;
use kivact::prelude::*;
use kivact::test::{RenderOptions, Screen, WindowMode};
use kivact::ui::{EditPhase, ScalarRange, Toast, toaster};
use kivact::ui::{range_slider, splitter, toast_region};
use kivact::view::{A11yAction, MouseButton, Orientation, ReadingDirection, provide_direction};

use EditPhase::{Begin, Cancel, Commit};

type Log = Rc<RefCell<Vec<EditPhase>>>;

fn options() -> RenderOptions {
    RenderOptions {
        window: WindowMode::Real,
        reduced_motion: true,
        ..RenderOptions::from_env()
    }
}

fn logger(log: &Log) -> impl Fn(EditPhase) + 'static {
    let log = log.clone();
    move |phase| log.borrow_mut().push(phase)
}

fn rect(screen: &Screen, role: Role, name: &str) -> kivact::draw::Rect {
    screen.get_by_role(role, name).rect().expect("laid out")
}

// ---------------------------------------------------------------------------
// Radio group.

fn sizes() -> Vec<ListItem<&'static str>> {
    vec![
        ListItem::new("s", "Small"),
        ListItem::new("m", "Medium").description("The usual"),
        ListItem::new("x", "Huge").disabled(true),
        ListItem::new("l", "Large"),
    ]
}

#[test]
fn a_radio_group_chooses_by_keys_pointer_and_invoke_each_choice_one_edit() {
    let log = Log::default();
    let seen = log.clone();
    let (screen, value) = mount_with(options(), move || {
        let value = signal(Some("m"));
        let view = column((
            button("Before"),
            radio_group("Size", sizes(), value).on_edit(logger(&seen)),
        ));
        (value, view)
    });
    let group = screen.get_by_role(Role::RadioGroup, "Size");
    assert_eq!(group.children().len(), 4);
    assert_eq!(
        screen.get_by_role(Role::RadioButton, "Medium").is_checked(),
        Some(true)
    );
    screen.find_by_text("The usual");
    // Tab enters at the chosen option; the arrows choose and skip the
    // disabled one.
    screen.get_by_role(Role::Button, "Before").focus();
    screen.press_tab(false);
    assert!(screen.get_by_role(Role::RadioButton, "Medium").is_focused());
    screen.press_key(Key::ArrowDown);
    assert_eq!(value.get_untracked(), Some("l"));
    assert!(screen.get_by_role(Role::RadioButton, "Large").is_focused());
    screen.press_key(Key::ArrowUp);
    assert_eq!(value.get_untracked(), Some("m"));
    screen.get_by_role(Role::RadioButton, "Small").click();
    assert_eq!(value.get_untracked(), Some("s"));
    screen
        .get_by_role(Role::RadioButton, "Large")
        .a11y(A11yAction::Activate);
    assert_eq!(value.get_untracked(), Some("l"));
    assert_eq!(
        *log.borrow(),
        [Begin, Commit, Begin, Commit, Begin, Commit, Begin, Commit],
        "each choice is a whole edit; nothing is left to cancel"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_horizontal_radio_group_runs_from_the_right_in_a_right_to_left_subtree() {
    let (screen, value) = mount_with(options(), || {
        let value = signal(Some("s"));
        let view = provide_direction(ReadingDirection::Rtl, move || {
            radio_group(
                "Size",
                vec![ListItem::new("s", "Small"), ListItem::new("l", "Large")],
                value,
            )
            .orientation(Orientation::Horizontal)
        });
        (value, view)
    });
    let (small, large) = (
        rect(&screen, Role::RadioButton, "Small"),
        rect(&screen, Role::RadioButton, "Large"),
    );
    assert!(small.x > large.x, "the first option is on the right");
    screen.get_by_role(Role::RadioButton, "Small").focus();
    screen.press_key(Key::ArrowLeft);
    assert_eq!(value.get_untracked(), Some("l"), "ArrowLeft moves on");
    screen.press_key(Key::ArrowRight);
    assert_eq!(value.get_untracked(), Some("s"));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

// ---------------------------------------------------------------------------
// Slider.

#[test]
fn a_slider_steps_by_keys_and_actions_and_a_canceled_drag_writes_the_start_back() {
    let log = Log::default();
    let seen = log.clone();
    let (screen, value) = mount_with(options(), move || {
        let value = signal(40.0);
        let view = slider("Volume", value)
            .on_edit(logger(&seen))
            .test_id("volume")
            .style(style().width(220));
        (value, column(view).style(style().padding(20)))
    });
    let thumb = screen.get_by_role(Role::Slider, "Volume");
    assert_eq!(thumb.value_range(), (Some(0.0), Some(100.0), Some(40.0)));
    thumb.focus();
    screen.press_key(Key::ArrowRight);
    assert_eq!(value.get_untracked(), 41.0);
    screen.press_key(Key::End);
    assert_eq!(value.get_untracked(), 100.0);
    thumb.a11y(A11yAction::Decrement);
    assert_eq!(value.get_untracked(), 99.0);
    thumb.a11y_with_value(A11yAction::SetValue, "25");
    assert_eq!(value.get_untracked(), 25.0);
    assert_eq!(
        *log.borrow(),
        [Begin, Commit, Begin, Commit, Begin, Commit, Begin, Commit]
    );
    log.borrow_mut().clear();
    // The thumb sits at its fraction along the track.
    let track = screen.get_by_test_id("volume").rect().expect("laid out");
    let at = thumb.rect().expect("laid out");
    let center = at.x + at.width / 2.0;
    let expected = track.x + at.width / 2.0 + 0.25 * (track.width - at.width);
    assert!((center - expected).abs() <= 1.0, "{center} vs {expected}");
    // A press on the track moves the value; Escape during the drag writes
    // the value from before the press back.
    screen.mouse_move(track.x + track.width - 1.0, track.y + track.height / 2.0);
    screen.mouse_down(MouseButton::Primary);
    assert_eq!(value.get_untracked(), 100.0);
    screen.press_key(Key::Escape);
    assert_eq!(value.get_untracked(), 25.0);
    screen.mouse_up(MouseButton::Primary);
    assert_eq!(*log.borrow(), [Begin, Cancel]);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_right_to_left_slider_runs_from_the_right_and_a_vertical_one_from_the_bottom() {
    let (screen, (rtl, vertical)) = mount_with(options(), || {
        let (rtl, vertical) = (signal(0.0), signal(0.0));
        let view = column((
            provide_direction(ReadingDirection::Rtl, move || {
                slider("Balance", rtl)
                    .test_id("rtl")
                    .style(style().width(200))
            }),
            slider("Level", vertical)
                .orientation(Orientation::Vertical)
                .test_id("vertical"),
        ))
        .style(style().padding(20).gap(20));
        ((rtl, vertical), view)
    });
    let track = screen.get_by_test_id("rtl").rect().expect("laid out");
    let thumb = rect(&screen, Role::Slider, "Balance");
    assert!(
        thumb.x + thumb.width >= track.x + track.width - 1.0,
        "zero is at the right edge"
    );
    screen.get_by_role(Role::Slider, "Balance").focus();
    screen.press_key(Key::ArrowLeft);
    assert_eq!(rtl.get_untracked(), 1.0, "ArrowLeft increases");
    // A press near the left edge is near the maximum.
    screen.mouse_move(track.x + 2.0, track.y + track.height / 2.0);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_up(MouseButton::Primary);
    assert!(rtl.get_untracked() > 95.0, "{}", rtl.get_untracked());
    let column = screen.get_by_test_id("vertical").rect().expect("laid out");
    assert!(column.height > column.width, "a vertical track is a column");
    let thumb = rect(&screen, Role::Slider, "Level");
    assert!(thumb.y + thumb.height >= column.y + column.height - 1.0);
    screen.get_by_role(Role::Slider, "Level").focus();
    screen.press_key(Key::ArrowUp);
    assert_eq!(vertical.get_untracked(), 1.0);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_range_slider_names_its_thumbs_keeps_them_apart_and_cancels_both() {
    let log = Log::default();
    let seen = log.clone();
    let (screen, (low, high)) = mount_with(options(), move || {
        let (low, high) = (signal(20.0), signal(60.0));
        let view = range_slider("Price", low, high)
            .on_edit(logger(&seen))
            .test_id("price")
            .style(style().width(220));
        ((low, high), column(view).style(style().padding(20)))
    });
    screen.get_by_role(Role::Group, "Price");
    let minimum = screen.get_by_role(Role::Slider, "Minimum");
    screen.get_by_role(Role::Slider, "Maximum");
    minimum.focus();
    minimum.a11y_with_value(A11yAction::SetValue, "90");
    assert_eq!(
        low.get_untracked(),
        60.0,
        "the low end stops at the high one"
    );
    assert_eq!(high.get_untracked(), 60.0);
    log.borrow_mut().clear();
    let track = screen.get_by_test_id("price").rect().expect("laid out");
    screen.mouse_move(track.x + track.width - 1.0, track.y + track.height / 2.0);
    screen.mouse_down(MouseButton::Primary);
    assert_eq!(high.get_untracked(), 100.0, "the nearest thumb follows");
    screen.press_key(Key::Escape);
    assert_eq!((low.get_untracked(), high.get_untracked()), (60.0, 60.0));
    screen.mouse_up(MouseButton::Primary);
    assert_eq!(*log.borrow(), [Begin, Cancel]);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_drag_of_a_range_sliders_filled_span_moves_both_ends_and_stops_at_the_bounds() {
    let log = Log::default();
    let seen = log.clone();
    let (screen, (low, high)) = mount_with(options(), move || {
        let (low, high) = (signal(20.0), signal(40.0));
        let view = range_slider("Price", low, high)
            .range(0.0..=100.0)
            .step(1.0)
            .on_edit(logger(&seen))
            .test_id("price")
            .style(style().width(220));
        ((low, high), column(view).style(style().padding(20)))
    });
    let track = screen.get_by_test_id("price").rect().expect("laid out");
    let (from, to) = (
        rect(&screen, Role::Slider, "Minimum"),
        rect(&screen, Role::Slider, "Maximum"),
    );
    let usable = track.width - from.width;
    // The middle of the span between the thumbs' centres.
    let x = (from.x + from.width / 2.0 + to.x + to.width / 2.0) / 2.0;
    let y = track.y + track.height / 2.0;
    screen.mouse_move(x, y);
    screen.mouse_down(MouseButton::Primary);
    assert_eq!((low.get_untracked(), high.get_untracked()), (20.0, 40.0));
    screen.mouse_move(x + 0.2 * usable, y);
    assert_eq!((low.get_untracked(), high.get_untracked()), (40.0, 60.0));
    // Past the end, the span stops where the high end meets the bound.
    screen.mouse_move(x + usable, y);
    assert_eq!((low.get_untracked(), high.get_untracked()), (80.0, 100.0));
    screen.press_key(Key::Escape);
    assert_eq!((low.get_untracked(), high.get_untracked()), (20.0, 40.0));
    screen.mouse_up(MouseButton::Primary);
    assert_eq!(*log.borrow(), [Begin, Cancel]);
    log.borrow_mut().clear();
    screen.mouse_move(x, y);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(x - 0.1 * usable, y);
    screen.mouse_up(MouseButton::Primary);
    assert_eq!((low.get_untracked(), high.get_untracked()), (10.0, 30.0));
    assert_eq!(*log.borrow(), [Begin, Commit]);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_slider_takes_inclusive_bounds_and_a_step() {
    let (screen, value) = mount_with(options(), || {
        let value = signal(4.0);
        let view = slider("Level", value).range(0.0..=10.0).step(2.0);
        (value, column(view).style(style().padding(20).width(220)))
    });
    let thumb = screen.get_by_role(Role::Slider, "Level");
    assert_eq!(thumb.value_range(), (Some(0.0), Some(10.0), Some(4.0)));
    thumb.focus();
    screen.press_key(Key::ArrowRight);
    assert_eq!(value.get_untracked(), 6.0);
    thumb.a11y_with_value(A11yAction::SetValue, "6.7");
    assert_eq!(value.get_untracked(), 6.0, "the value snaps to the step");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

// ---------------------------------------------------------------------------
// Splitter.

fn sidebar() -> ScalarRange {
    ScalarRange::new(100.0, 300.0).expect("a valid range")
}

#[test]
fn a_splitter_sizes_the_leading_pane_by_keys_drag_and_actions_and_cancels_a_drag() {
    let log = Log::default();
    let seen = log.clone();
    let (screen, size) = mount_with(options(), move || {
        let size = signal(200.0);
        let view = splitter("Sidebar width", size, sidebar())
            .start(node().test_id("start").style(style().grow(1)))
            .end(node().test_id("end").style(style().grow(1)))
            .on_edit(logger(&seen))
            .style(style().size(500, 100));
        (size, view)
    });
    let handle = screen.get_by_role(Role::Splitter, "Sidebar width");
    assert_eq!(handle.orientation(), Orientation::Vertical);
    let start = screen.get_by_test_id("start").rect().expect("laid out");
    assert_eq!(start.width, 200.0);
    handle.focus();
    screen.press_key(Key::ArrowRight);
    assert_eq!(size.get_untracked(), 202.0);
    handle.a11y(A11yAction::Increment);
    assert_eq!(size.get_untracked(), 204.0);
    screen.press_key(Key::Home);
    assert_eq!(size.get_untracked(), 100.0);
    assert_eq!(
        screen
            .get_by_test_id("start")
            .rect()
            .expect("laid out")
            .width,
        100.0
    );
    assert_eq!(*log.borrow(), [Begin, Commit, Begin, Commit, Begin, Commit]);
    log.borrow_mut().clear();
    let at = handle.rect().expect("laid out");
    let (x, y) = (at.x + at.width / 2.0, at.y + at.height / 2.0);
    screen.mouse_move(x, y);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(x + 60.0, y);
    assert_eq!(size.get_untracked(), 160.0);
    screen.press_key(Key::Escape);
    assert_eq!(size.get_untracked(), 100.0, "Escape writes the start back");
    screen.mouse_up(MouseButton::Primary);
    assert_eq!(*log.borrow(), [Begin, Cancel]);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_right_to_left_splitter_leads_from_the_right() {
    let (screen, size) = mount_with(options(), || {
        let size = signal(200.0);
        let view = provide_direction(ReadingDirection::Rtl, move || {
            splitter("Sidebar width", size, sidebar())
                .start(node().test_id("start").style(style().grow(1)))
                .end(node().test_id("end").style(style().grow(1)))
                .style(style().size(500, 100))
        });
        (size, view)
    });
    let start = screen.get_by_test_id("start").rect().expect("laid out");
    let end = screen.get_by_test_id("end").rect().expect("laid out");
    assert!(start.x > end.x, "the leading pane is on the right");
    let handle = screen.get_by_role(Role::Splitter, "Sidebar width");
    handle.focus();
    screen.press_key(Key::ArrowLeft);
    assert_eq!(size.get_untracked(), 202.0, "ArrowLeft grows it");
    let at = handle.rect().expect("laid out");
    let (x, y) = (at.x + at.width / 2.0, at.y + at.height / 2.0);
    screen.mouse_move(x, y);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(x - 40.0, y);
    screen.mouse_up(MouseButton::Primary);
    assert_eq!(size.get_untracked(), 242.0, "a drag to the left grows it");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

// ---------------------------------------------------------------------------
// Toast region.

#[test]
fn a_toast_region_shows_toasts_and_their_close_action_and_escape_dismiss_them() {
    let (screen, ran) = mount_with(options(), || {
        let toaster = toaster();
        let ran = signal(0);
        let view = column((
            button("Save").on_press(move || {
                toaster.push(
                    Toast::new("Saved")
                        .body("All changes are on disk")
                        .duration(None)
                        .action("Undo", move || ran.update(|n| *n += 1)),
                );
            }),
            toast_region(toaster),
        ));
        (ran, view)
    });
    assert_eq!(
        screen.all_by_role(Role::Group, "").len(),
        2,
        "both live regions"
    );
    screen.get_by_role(Role::Button, "Save").click();
    let toast = screen.get_by_role(Role::Status, "Saved");
    screen.find_by_text("All changes are on disk");
    screen.get_by_role(Role::Button, "Undo").click();
    assert_eq!(ran.get_untracked(), 1);
    assert!(
        screen.query_by_role(Role::Status, "Saved").is_none(),
        "the action dismisses it"
    );
    let _ = toast;
    screen.get_by_role(Role::Button, "Save").click();
    screen.get_by_role(Role::Button, "Close").click();
    assert!(screen.query_by_role(Role::Status, "Saved").is_none());
    screen.get_by_role(Role::Button, "Save").click();
    screen.get_by_role(Role::Button, "Close").focus();
    screen.press_key(Key::Escape);
    assert!(screen.query_by_role(Role::Status, "Saved").is_none());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_region_that_holds_all_on_hover_holds_every_toast_under_the_pointer() {
    let (screen, toaster) = mount_with(options(), || {
        let toaster = toaster();
        (
            toaster,
            column((
                button("Away"),
                toast_region(toaster).hold_all_on_hover(true),
            )),
        )
    });
    let second = std::time::Duration::from_secs(1);
    toaster.push(Toast::new("First").duration(Some(second)));
    toaster.push(Toast::new("Second").duration(Some(second)));
    screen.run_until_idle();
    // Over the first toast, the second one's time stops as well.
    screen.get_by_text("First").hover();
    screen.advance(std::time::Duration::from_secs(5));
    assert!(screen.query_by_role(Role::Status, "Second").is_some());
    screen.get_by_role(Role::Button, "Away").hover();
    screen.advance(std::time::Duration::from_secs(5));
    assert!(screen.query_by_role(Role::Status, "First").is_none());
    assert!(screen.query_by_role(Role::Status, "Second").is_none());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn toast_recipes_know_each_toasts_depth_and_what_the_groups_above_show() {
    use kivact::interaction::ToastState;
    use kivact::ui::{ToastGroupState, Tone, UiRecipes};
    let theme = Theme::light();
    let recipes = UiRecipes::clone(&theme.get::<UiRecipes>()).toast(|toast| {
        toast
            .toast(|prev| {
                move |theme: &Theme, state: &kivact::ui::Look<ToastState>| {
                    prev(theme, state).min_height(80.0 + 10.0 * state.depth as f32)
                }
            })
            .group(|prev| {
                move |theme: &Theme, state: &kivact::ui::Look<ToastGroupState>| {
                    let s = prev(theme, state);
                    if state.assertive && state.count > 0 && state.above > 0 {
                        s.margin_top(30)
                    } else {
                        s.margin_top(0)
                    }
                }
            })
    });
    let options = RenderOptions {
        theme: theme.with(recipes),
        ..options()
    };
    let (screen, toaster) = mount_with(options, || {
        let toaster = toaster();
        (toaster, toast_region(toaster))
    });
    toaster.push(Toast::new("Older").duration(None));
    toaster.push(Toast::new("Newer").duration(None));
    screen.run_until_idle();
    let older = rect(&screen, Role::Status, "Older");
    let newer = rect(&screen, Role::Status, "Newer");
    assert!(
        (older.height - newer.height - 10.0).abs() <= 0.5,
        "{older:?} {newer:?}"
    );
    toaster.push(Toast::new("Failed").tone(Tone::Danger).duration(None));
    screen.run_until_idle();
    let newer = rect(&screen, Role::Status, "Newer");
    let failed = rect(&screen, Role::Alert, "Failed");
    let gap = failed.y - (newer.y + newer.height);
    assert!(
        gap >= 30.0 - 0.5,
        "the assertive group sits 30 px lower: {gap}"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn the_callers_style_and_name_win_over_the_recipes_of_a_radio_group_and_a_slider() {
    use kivact::ui::{AxisState, UiRecipes};
    use kivact::view::Orientation;
    // Recipes that size the group and the track, which the caller overrides.
    let theme = Theme::light();
    let recipes = UiRecipes::clone(&theme.get::<UiRecipes>())
        .radio_group(|r| {
            r.group(|prev| {
                move |t: &Theme, o: &kivact::ui::Look<Orientation>| prev(t, o).width(400)
            })
        })
        .slider(|r| {
            r.track(|prev| move |t: &Theme, s: &kivact::ui::Look<AxisState>| prev(t, s).width(400))
        });
    let options = RenderOptions {
        theme: theme.with(recipes),
        ..options()
    };
    let (screen, ()) = mount_with(options, || {
        let view = column((
            radio_group("Size", sizes(), signal(Some("s")))
                .style(style().width(150))
                .test_id("radios"),
            slider("Volume", signal(10.0))
                .style(style().width(160))
                .test_id("slider"),
            range_slider("Price", signal(1.0), signal(9.0))
                .name("Own range")
                .style(style().width(170))
                .test_id("range"),
        ))
        .style(style().align_items(kivact::style::Align::Start));
        ((), view)
    });
    let width = |id: &str| screen.get_by_test_id(id).rect().expect("laid out").width;
    assert_eq!(width("radios"), 150.0);
    assert_eq!(width("slider"), 160.0);
    assert_eq!(width("range"), 170.0);
    screen.get_by_role(Role::Group, "Own range");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// Generic code sees the one `disabled` of `NodeBuilder`; the component reads
/// it, so its own input is disabled too.
fn disabled<B: NodeBuilder>(builder: B) -> B {
    builder.disabled(true)
}

#[test]
fn disabled_through_node_builder_disables_a_components_own_input() {
    let (screen, ()) = mount_with(options(), || {
        let view = column((
            disabled(slider("Volume", signal(10.0))),
            disabled(splitter("Side", signal(200.0), sidebar())),
        ));
        ((), view)
    });
    // The thumb and the handle are descendants of the disabled root: they are
    // disabled only because the component gave the root's flag to its behavior.
    assert!(screen.get_by_role(Role::Slider, "Volume").is_disabled());
    assert!(screen.get_by_role(Role::Splitter, "Side").is_disabled());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_disabled_slider_disables_its_track_with_its_thumb() {
    let (screen, ()) = mount_with(options(), || {
        let view = column((
            slider("Volume", signal(10.0))
                .disabled(true)
                .test_id("single"),
            range_slider("Price", signal(1.0), signal(9.0))
                .disabled(true)
                .test_id("range"),
        ));
        ((), view)
    });
    assert!(screen.get_by_test_id("single").is_disabled());
    assert!(screen.get_by_role(Role::Slider, "Volume").is_disabled());
    assert!(screen.get_by_test_id("range").is_disabled());
    assert!(screen.get_by_role(Role::Slider, "Minimum").is_disabled());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
