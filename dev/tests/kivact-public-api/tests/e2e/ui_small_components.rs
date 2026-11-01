//! Role: Tests the standard toggle, progress, disclosure, split and icon button.
//!
//! On real windows: a toggle button's role, pressed state, keys, accessibility
//! actions and the state its recipe reads; a progress bar's range, value text,
//! clamping and indeterminate state; a disclosure's trigger, content mounting,
//! accordion over one list of keys and animated height; a split's handles
//! by key, pointer, assistive technology and edit phases; and a button that
//! shows only its icon handing `icon_only` to its recipe.

use crate::support::{mount, mount_with};
use kivact::interaction::DisclosureState;
use kivact::prelude::*;
use kivact::style::{Align, Style, Theme};
use kivact::test::{RenderOptions, Screen};
use kivact::ui::{
    self, ButtonState, ControlSize, Look, ProgressState, ToggleButtonState, UiRecipes, button,
    disclosure, progress, split, toggle,
};
use kivact::view::{A11yAction, MouseButton, Orientation, callback};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

fn near(a: f32, b: f32, tolerance: f32) -> bool {
    (a - b).abs() <= tolerance
}

fn rect(screen: &Screen, id: &str) -> kivact::draw::Rect {
    screen.get_by_test_id(id).rect().expect("laid out")
}

/// Render `f` with `recipes` installed in the light theme.
fn with_recipes<V: IntoView>(recipes: UiRecipes, f: impl FnOnce() -> V + 'static) -> Screen {
    let (screen, ()) = mount_with(
        RenderOptions {
            theme: Theme::light().with(recipes),
            ..RenderOptions::from_env()
        },
        move || ((), column(f()).style(style().align_items(Align::Start))),
    );
    screen
}

// ---------------------------------------------------------------- toggle

#[test]
fn a_toggle_is_a_button_with_a_pressed_state_that_every_input_flips() {
    let (screen, on) = mount(|| {
        let on = signal(false);
        (
            (on),
            column((
                toggle("Bold", on),
                toggle("Off", signal(false)).disabled(true),
            )),
        )
    });
    let bold = screen.get_by_role(Role::Button, "Bold");
    assert_eq!(bold.is_checked(), Some(false));
    bold.click();
    assert!(on.get_untracked());
    assert_eq!(bold.is_checked(), Some(true));
    bold.focus();
    bold.press_key(Key::Space);
    assert!(!on.get_untracked(), "Space flips it");
    bold.press_key(Key::Enter);
    assert!(on.get_untracked(), "Enter flips a toggle button");
    bold.a11y(A11yAction::Toggle);
    assert!(!on.get_untracked());
    on.set(true);
    screen.run_until_idle();
    assert_eq!(bold.is_checked(), Some(true), "the binding is the state");
    let off = screen.get_by_role(Role::Button, "Off");
    assert!(off.is_disabled());
    assert_eq!(off.is_checked(), Some(false));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_toggle_hands_its_state_to_the_recipe_and_the_callers_style_wins() {
    fn sized(_: &Theme, state: &Look<ToggleButtonState>) -> Style {
        let wide = if state.pressed { 90 } else { 60 };
        let wide = if state.icon_only { 30 } else { wide };
        style().size(wide + if state.leading_icon { 1 } else { 0 }, 20)
    }
    let screen = with_recipes(UiRecipes::default().toggle(|_| sized), || {
        let on = signal(false);
        (
            toggle("Plain", on).test_id("plain"),
            toggle("Icon", signal(false))
                .icon(icon("x"))
                .hide_label(true)
                .test_id("icon"),
            toggle("Labelled", signal(false))
                .icon(icon("x"))
                .test_id("labelled"),
            toggle("Mine", signal(false))
                .style(style().width(77))
                .test_id("mine"),
        )
    });
    assert_eq!(rect(&screen, "plain").width, 60.0);
    screen.get_by_role(Role::Button, "Plain").click();
    assert_eq!(rect(&screen, "plain").width, 90.0, "pressed");
    assert_eq!(
        rect(&screen, "icon").width,
        31.0,
        "icon only, with its icon"
    );
    assert_eq!(rect(&screen, "labelled").width, 61.0);
    assert_eq!(rect(&screen, "mine").width, 77.0, "the caller's style wins");
    assert_eq!(
        rect(&screen, "mine").height,
        20.0,
        "and the recipe fills the rest"
    );
    // An icon-only toggle keeps its accessible name.
    screen.get_by_role(Role::Button, "Icon");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

// ----------------------------------------------------------- icon button

#[test]
fn a_button_with_an_icon_and_no_visible_label_hands_icon_only_to_its_recipe() {
    fn sized(_: &Theme, state: &Look<ButtonState>) -> Style {
        style().size(if state.icon_only { 30 } else { 70 }, 20)
    }
    let screen = with_recipes(UiRecipes::default().button(|_| sized), || {
        let hide = signal(true);
        (
            button("Plain").test_id("plain"),
            button("Icon").icon(icon("x")).test_id("labelled"),
            button("Add")
                .icon(icon("x"))
                .hide_label(true)
                .test_id("icon-only"),
            button("Reactive")
                .icon(icon("x"))
                .hide_label(hide)
                .test_id("reactive"),
            // The label hidden without an icon is not an icon button.
            button("Hidden").hide_label(true).test_id("hidden"),
            ui::button("Toggler")
                .icon(icon("x"))
                .hide_label(move || hide.get())
                .on_press(move || hide.update(|h| *h = !*h))
                .test_id("flip"),
        )
    });
    assert_eq!(rect(&screen, "plain").width, 70.0);
    assert_eq!(rect(&screen, "labelled").width, 70.0);
    assert_eq!(rect(&screen, "icon-only").width, 30.0);
    assert_eq!(rect(&screen, "reactive").width, 30.0);
    assert_eq!(rect(&screen, "hidden").width, 70.0);
    assert_eq!(rect(&screen, "flip").width, 30.0);
    screen.get_by_role(Role::Button, "Toggler").click();
    assert_eq!(rect(&screen, "flip").width, 70.0, "the label shows again");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

// -------------------------------------------------------------- progress

#[test]
fn a_progress_bar_publishes_a_range_of_zero_to_one_and_clamps() {
    let (screen, value) = mount(|| {
        let value = signal(Some(0.4));
        (
            value,
            column((
                progress(value).name("Upload").style(style().width(200)),
                progress(Some(7.0)).name("Over"),
                progress(Some(-1.0)).name("Under"),
                progress(Some(f64::NAN)).name("Bad"),
                progress(None::<f64>).name("Busy"),
            )),
        )
    });
    let bar = screen.get_by_role(Role::ProgressBar, "Upload");
    assert_eq!(bar.value_range(), (Some(0.0), Some(1.0), Some(0.4)));
    assert_eq!(bar.value().as_deref(), Some("40%"));
    value.set(Some(0.75));
    screen.run_until_idle();
    assert_eq!(bar.value_range().2, Some(0.75));
    assert_eq!(bar.value().as_deref(), Some("75%"));
    let now = |name| screen.get_by_role(Role::ProgressBar, name).value_range().2;
    assert_eq!(now("Over"), Some(1.0));
    assert_eq!(now("Under"), Some(0.0));
    assert_eq!(now("Bad"), Some(0.0));
    // An indeterminate bar reports no value.
    assert_eq!(now("Busy"), None);
    // The value turning into `None` and back keeps the bar.
    value.set(None);
    screen.run_until_idle();
    value.set(Some(0.5));
    screen.run_until_idle();
    assert_eq!(bar.value().as_deref(), Some("50%"));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_progress_bar_hands_its_value_and_size_to_the_recipes_of_its_track_and_fill() {
    let seen = Arc::new(Mutex::new(Vec::new()));
    let log = seen.clone();
    let recipes = UiRecipes::default().progress(move |table| {
        table
            .track(|_| {
                |_: &Theme, state: &Look<ProgressState>| {
                    style().width(200).height(match state.size {
                        ControlSize::Small => 4,
                        _ => 10,
                    })
                }
            })
            .fill(move |_| {
                let log = log.clone();
                move |_: &Theme, state: &Look<ProgressState>| {
                    log.lock().expect("log").push(state.value);
                    match state.value {
                        Some(value) => style().height(pct(100)).width(pct(value as f32 * 100.0)),
                        None => style().height(pct(100)).width(pct(100)),
                    }
                }
            })
    });
    let screen = with_recipes(recipes, || {
        let value = signal(Some(0.25));
        (
            progress(value).name("Half").test_id("bar"),
            progress(Some(1.0))
                .name("Thin")
                .size(ControlSize::Small)
                .test_id("thin"),
            progress(None::<f64>).name("Busy").test_id("busy"),
        )
    });
    assert_eq!(rect(&screen, "bar").height, 10.0);
    assert_eq!(rect(&screen, "thin").height, 4.0);
    let fill = screen.get_by_test_id("bar").children()[0]
        .rect()
        .expect("fill laid out");
    assert!(near(fill.width, 50.0, 0.6), "a quarter of 200: {fill:?}");
    assert!(seen.lock().expect("log").contains(&Some(0.25)));
    assert!(
        seen.lock().expect("log").contains(&None),
        "indeterminate is None"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

// ------------------------------------------------------------ disclosure

#[test]
fn a_disclosure_trigger_opens_and_closes_content_that_is_built_each_time() {
    let builds = Rc::new(RefCell::new(0));
    let counted = builds.clone();
    let (screen, open) = mount(move || {
        let open = signal(false);
        (
            open,
            disclosure("Details", open, move || {
                *counted.borrow_mut() += 1;
                text("The details")
            }),
        )
    });
    let trigger = screen.get_by_role(Role::Button, "Details");
    assert_eq!(trigger.is_expanded(), Some(false));
    assert!(screen.query_by_text("The details").is_none());
    assert_eq!(*builds.borrow(), 0);
    trigger.click();
    assert!(open.get_untracked());
    screen.get_by_text("The details");
    assert_eq!(trigger.is_expanded(), Some(true));
    trigger.focus();
    trigger.press_key(Key::Space);
    assert!(!open.get_untracked(), "Space closes it");
    assert!(screen.query_by_text("The details").is_none());
    trigger.press_key(Key::Enter);
    assert!(open.get_untracked(), "Enter opens it");
    assert_eq!(*builds.borrow(), 2, "built again on every opening");
    trigger.a11y(A11yAction::Collapse);
    assert!(!open.get_untracked());
    trigger.a11y(A11yAction::Expand);
    assert!(open.get_untracked());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_kept_mounted_disclosure_builds_its_content_once_and_keeps_its_state() {
    let (screen, open) = mount(|| {
        let open = signal(true);
        (
            open,
            disclosure("Settings", open, || {
                let name = signal(String::new());
                kivact::ui::text_input("Name", name)
            })
            .keep_mounted(true),
        )
    });
    let field = screen.get_by_role(Role::TextBox, "Name");
    field.focus();
    screen.type_text("Ada");
    open.set(false);
    screen.run_until_idle();
    assert!(
        screen.query_by_role(Role::TextBox, "Name").is_none()
            || !screen.get_by_role(Role::TextBox, "Name").is_visible(),
        "hidden while closed"
    );
    open.set(true);
    screen.run_until_idle();
    assert_eq!(
        screen.get_by_role(Role::TextBox, "Name").value().as_deref(),
        Some("Ada")
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn an_accordion_is_a_column_of_disclosures_over_one_list_of_keys() {
    let (screen, open) = mount(|| {
        let open = signal(vec!["one"]);
        let section = move |key: &'static str| {
            let is_open = binding(
                move || open.with(|keys| keys.contains(&key)),
                move |on| {
                    open.update(|keys| {
                        keys.retain(|k| *k != key);
                        if on {
                            keys.push(key);
                        }
                    })
                },
            );
            disclosure(format!("Section {key}"), is_open, move || {
                text(format!("Body {key}"))
            })
        };
        (
            open,
            column(vec![section("one"), section("two"), section("three")]),
        )
    });
    screen.get_by_text("Body one");
    assert!(screen.query_by_text("Body two").is_none());
    screen.get_by_role(Role::Button, "Section three").click();
    screen.get_by_role(Role::Button, "Section two").click();
    assert_eq!(open.get_untracked(), ["one", "three", "two"]);
    screen.get_by_text("Body two");
    screen.get_by_text("Body three");
    screen.get_by_role(Role::Button, "Section one").click();
    assert!(screen.query_by_text("Body one").is_none());
    open.set(vec![]);
    screen.run_until_idle();
    assert!(screen.query_by_text("Body two").is_none());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_disclosure_recipes_know_the_open_state_and_a_disabled_one_ignores_input() {
    fn row_of(_: &Theme, _: &Look<DisclosureState>) -> Style {
        style().width(100)
    }
    fn trigger(_: &Theme, state: &Look<DisclosureState>) -> Style {
        style().height(if state.open { 40 } else { 20 })
    }
    let screen = with_recipes(
        UiRecipes::default().disclosure(|table| {
            table
                .root(|_| row_of)
                .trigger(|_| trigger)
                .indicator(|_| |_: &Theme, _: &Look<DisclosureState>| style().size(8, 8))
                .content(|_| |_: &Theme, _: &Look<DisclosureState>| style().height(33))
        }),
        || {
            (
                disclosure("A", signal(false), || text("a")).test_id("closed"),
                disclosure("B", signal(true), || text("b")).test_id("open"),
                disclosure("C", signal(false), || text("c")).disabled(true),
            )
        },
    );
    assert_eq!(
        screen
            .get_by_role(Role::Button, "A")
            .rect()
            .expect("laid")
            .height,
        20.0
    );
    assert_eq!(
        screen
            .get_by_role(Role::Button, "B")
            .rect()
            .expect("laid")
            .height,
        40.0
    );
    assert_eq!(rect(&screen, "open").width, 100.0);
    let disabled = screen.get_by_role(Role::Button, "C");
    assert!(disabled.is_disabled());
    assert_eq!(disabled.is_expanded(), Some(false));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn an_animated_disclosure_moves_its_height_and_the_callers_style_still_wins() {
    let (screen, open) = mount(|| {
        let open = signal(false);
        (
            open,
            column((
                disclosure("Details", open, || {
                    column((text("One"), text("Two"), text("Three")))
                })
                .animated(true)
                .test_id("section"),
                text("After").test_id("after"),
            ))
            .style(style().width(200)),
        )
    });
    screen.set_reduced_motion(false);
    let closed = rect(&screen, "after").y;
    open.set(true);
    screen.advance(Duration::from_millis(60));
    let opening = rect(&screen, "after").y;
    screen.advance(Duration::from_millis(400));
    let done = rect(&screen, "after").y;
    assert!(
        closed < opening && opening < done,
        "{closed} {opening} {done}"
    );
    open.set(false);
    screen.advance(Duration::from_millis(60));
    let closing = rect(&screen, "after").y;
    assert!(
        closing < done && closing > closed,
        "{closed} {closing} {done}"
    );
    screen.advance(Duration::from_millis(400));
    assert_eq!(rect(&screen, "after").y, closed);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

// ----------------------------------------------------------------- split

fn mount_split(orientation: Orientation, sizes: Vec<f32>) -> (Screen, kivact::Signal<Vec<f32>>) {
    mount(move || {
        let sizes = signal(sizes);
        (
            sizes,
            split(
                vec![
                    text("One").test_id("one").into_view(),
                    text("Two").test_id("two").into_view(),
                    text("Three").test_id("three").into_view(),
                ],
                sizes,
            )
            .orientation(orientation)
            .grip(true)
            .style(style().width(606).height(300)),
        )
    })
}

fn handles(screen: &Screen) -> Vec<kivact::test::Element<'_>> {
    screen.all_by_role(Role::Splitter, "Resize")
}

#[test]
fn a_split_shares_the_room_by_its_fractions_with_a_handle_between_each_pair() {
    let (screen, sizes) = mount_split(Orientation::Horizontal, vec![0.25, 0.25, 0.5]);
    let handles = handles(&screen);
    assert_eq!(handles.len(), 2);
    assert_eq!(handles[0].orientation(), Orientation::Vertical);
    let width = |id: &str| rect(&screen, id).width;
    assert!(near(width("one") / width("three"), 0.5, 0.02));
    assert!(near(width("one"), width("two"), 1.0));
    assert_eq!(handles[0].value_range().2.map(f64::round), Some(25.0));
    assert_eq!(handles[1].value_range().2.map(f64::round), Some(50.0));
    // A list of the wrong length draws equal panes.
    sizes.set(vec![1.0]);
    screen.run_until_idle();
    assert!(near(width("one"), width("three"), 1.0));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_split_handle_moves_by_keys_within_the_minimum_and_resets_to_the_middle() {
    let (screen, sizes) = mount_split(Orientation::Horizontal, vec![0.25, 0.25, 0.5]);
    let first = handles(&screen).into_iter().next().expect("two handles");
    first.focus();
    first.press_key(Key::ArrowRight);
    let now = sizes.get_untracked();
    assert!(
        near(now[0], 0.26, 0.002) && near(now[1], 0.24, 0.002),
        "{now:?}"
    );
    assert!(near(now[2], 0.5, 0.002), "the third pane is untouched");
    first.press_key(Key::ArrowDown);
    assert!(
        near(sizes.get_untracked()[0], 0.26, 0.002),
        "only the axis' keys"
    );
    first.press_key(Key::Home);
    assert!(
        near(sizes.get_untracked()[0], 0.1, 0.002),
        "the 10 % minimum"
    );
    first.press_key(Key::End);
    let now = sizes.get_untracked();
    assert!(
        near(now[1], 0.1, 0.002),
        "the next pane keeps its minimum: {now:?}"
    );
    assert!(near(now[0] + now[1], 0.5, 0.002), "the pair keeps its sum");
    first.press_key(Key::Enter);
    let now = sizes.get_untracked();
    assert!(near(now[0], now[1], 0.002), "Enter shares the pair equally");
    first.a11y_with_value(A11yAction::SetValue, "20");
    assert!(near(sizes.get_untracked()[0], 0.2, 0.002));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_split_handle_follows_the_pointer_by_the_pixels_the_panes_share() {
    let (screen, sizes) = mount_split(Orientation::Horizontal, vec![0.5, 0.25, 0.25]);
    let first = handles(&screen).into_iter().next().expect("two handles");
    let total = rect(&screen, "one").width / sizes.get_untracked()[0];
    let from = first.rect().expect("laid out").center();
    let from = kivact::draw::Point::new(
        from.x - first.rect().expect("laid out").x,
        from.y - first.rect().expect("laid out").y,
    );
    first.drag(from, kivact::draw::Point::new(from.x + 60.0, from.y));
    let after = sizes.get_untracked();
    assert!(
        near(after[0], 0.5 + 60.0 / total, 0.01),
        "{after:?} of {total}"
    );
    first.drag(from, kivact::draw::Point::new(from.x + 5000.0, from.y));
    assert!(
        near(sizes.get_untracked()[1], 0.1, 0.01),
        "{:?}",
        sizes.get_untracked()
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_stacked_split_moves_along_y() {
    let (screen, sizes) = mount_split(Orientation::Vertical, vec![0.5, 0.25, 0.25]);
    let handle = handles(&screen).into_iter().next().expect("handles");
    assert_eq!(handle.orientation(), Orientation::Horizontal);
    assert!(
        rect(&screen, "two").y > rect(&screen, "one").y,
        "top to bottom"
    );
    handle.focus();
    handle.press_key(Key::ArrowDown);
    assert!(near(sizes.get_untracked()[0], 0.51, 0.002));
    handle.press_key(Key::ArrowRight);
    assert!(
        near(sizes.get_untracked()[0], 0.51, 0.002),
        "only the axis' keys"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_split_reports_edit_phases_and_restores_a_canceled_drag() {
    use kivact::ui::EditPhase::{Begin, Cancel, Commit};
    let log = Rc::new(RefCell::new(Vec::new()));
    let seen = log.clone();
    let (screen, (sizes, shown)) = mount(move || {
        let sizes = signal(vec![0.3f32, 0.7]);
        let shown = signal(true);
        // Keep the observer above the disposable split's owner.
        let observer =
            callback(move |phase| seen.borrow_mut().push((phase, sizes.get_untracked()[0])));
        (
            (sizes, shown),
            node()
                .style(style().size(400, 100))
                .child(show(shown, move || {
                    split((text("First"), text("Second")), sizes)
                        .on_edit(move |phase| observer.try_call(phase))
                })),
        )
    });
    let handle = || screen.get_by_role(Role::Splitter, "Resize");
    handle().press_key(Key::ArrowRight);
    assert_eq!(log.borrow().len(), 2);
    assert_eq!(log.borrow()[0].0, Begin);
    assert_eq!(log.borrow()[1].0, Commit);
    assert!(near(log.borrow()[1].1, 0.31, 0.002));
    for mode in 0..3 {
        screen.advance(Duration::from_secs(1));
        let initial = sizes.get_untracked()[0];
        let start = handle().rect().expect("laid out").center();
        screen.mouse_move(start.x, start.y);
        screen.mouse_down(MouseButton::Primary);
        assert_eq!(log.borrow().last().map(|e| e.0), Some(Begin));
        screen.mouse_move(start.x + 20.0, start.y);
        assert!(sizes.get_untracked()[0] > initial);
        match mode {
            0 => screen.mouse_up(MouseButton::Primary),
            1 => screen.press_key(Key::Escape),
            _ => {
                shown.set(false);
                screen.run_until_idle();
            }
        }
        if mode > 0 {
            assert!(near(sizes.get_untracked()[0], initial, 0.002), "restored");
            screen.mouse_up(MouseButton::Primary);
        }
        assert_eq!(
            log.borrow().last().map(|e| e.0),
            Some(if mode == 0 { Commit } else { Cancel })
        );
    }
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_split_disabled_by_the_caller_ignores_its_handles_and_a_grip_follows_its_flag() {
    let (screen, (sizes, grip)) = mount(|| {
        let sizes = signal(vec![0.5f32, 0.5]);
        let grip = signal(false);
        (
            (sizes, grip),
            split((text("A"), text("B")), sizes)
                .grip(grip)
                .disabled(true)
                .style(style().width(300).height(100)),
        )
    });
    let handle = screen.get_by_role(Role::Splitter, "Resize");
    assert!(handle.is_disabled());
    assert_eq!(sizes.get_untracked(), [0.5, 0.5]);
    let children = handle.children().len();
    grip.set(true);
    screen.run_until_idle();
    assert_eq!(handle.children().len(), children + 1, "the grip appears");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
