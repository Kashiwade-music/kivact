//! Role: Tests enter and exit paint: a node's starting paint and its exit before removal.
//!
//! On real windows with the virtual clock: a mounted node moves from its
//! enter paint to its paint, a removed one is drawn moving to its exit
//! paint and is then gone from the layout, while it takes no pointer,
//! focus or accessibility node from the removal on; reduced motion mounts
//! and removes at once; and the standard toast, dialog, popup (a select's
//! list) and tooltip take both from their recipes (the Game kit's toast
//! slides in from the right this way).

use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use crate::paint::{distance, pixel};
use crate::support::{mount, mount_with};
use kivact::draw::Rect;
use kivact::motion::{Easing, Transition};
use kivact::prelude::*;
use kivact::style::{Style, TransitionProp, rgb};
use kivact::test::{RenderOptions, Screen};
use kivact::ui::{
    ListItem, Look, Toast, Toaster, UiRecipes, dialog, select, toast_region, toaster, tooltip,
};

const RED: u32 = 0xff0000;

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

/// Fade in and out over 100 ms, linearly.
fn fade() -> Style {
    style()
        .enter(style().opacity(0.0))
        .exit(style().opacity(0.0))
        .transition(
            TransitionProp::Paint,
            Transition::tween(100, Easing::Linear),
        )
}

/// Slide in from 40 px to the right and back out, fading, over 100 ms.
fn slide() -> Style {
    style()
        .enter(style().translate_x(40).opacity(0.0))
        .exit(style().translate_x(40).opacity(0.0))
        .transition(
            TransitionProp::Paint,
            Transition::tween(100, Easing::Linear),
        )
}

/// The pixel at the centre of `r`.
fn centre(screen: &Screen, r: Rect) -> u32 {
    pixel(
        &screen.screenshot(),
        (r.x + r.width / 2.0) as u32,
        (r.y + r.height / 2.0) as u32,
    )
}

/// The pixel 2 px left of `r`, at its middle: on the surface around a
/// text or a row.
fn beside(screen: &Screen, r: Rect) -> u32 {
    pixel(
        &screen.screenshot(),
        (r.x - 2.0) as u32,
        (r.y + r.height / 2.0) as u32,
    )
}

#[test]
fn a_node_enters_from_its_enter_paint_and_is_drawn_until_its_exit_ends() {
    let presses = Rc::new(Cell::new(0));
    let counted = presses.clone();
    let (screen, shown) = mount(move || {
        let shown = signal(false);
        let view = column((
            show(shown, move || {
                let counted = counted.clone();
                column(button("Inside").on_press(move || counted.set(counted.get() + 1)))
                    .style(style().size(160, 80).background(rgb(RED)))
                    .style(fade())
                    .test_id("box")
            }),
            text("below").test_id("below"),
            button("Outside"),
        ))
        .style(style().align_items(kivact::style::Align::Start));
        (shown, view)
    });
    let below = |screen: &Screen| screen.get_by_test_id("below").rect().expect("laid out").y;
    let resting = below(&screen);
    let page = pixel(&screen.screenshot(), 150, 70);

    shown.set(true);
    screen.run_until_idle();
    let r = screen.get_by_test_id("box").rect().expect("laid out");
    let probe = |screen: &Screen| {
        pixel(
            &screen.screenshot(),
            (r.x + 150.0) as u32,
            (r.y + 70.0) as u32,
        )
    };
    assert!(
        distance(probe(&screen), page) <= 8,
        "starts at its enter paint"
    );
    screen.advance(ms(50));
    let mid = probe(&screen);
    assert!(
        distance(mid, page) > 40 && distance(mid, RED) > 40,
        "on its way: {mid:06x}"
    );
    screen.advance(ms(100));
    assert!(distance(probe(&screen), RED) <= 8, "arrives at its paint");
    assert_eq!(below(&screen), resting + 80.0);

    screen.get_by_role(Role::Button, "Inside").focus();
    let inside = screen
        .get_by_role(Role::Button, "Inside")
        .rect()
        .expect("laid out");
    shown.set(false);
    screen.run_until_idle();
    // Gone for everything but the eye: no accessibility node, no focus,
    // no hit; still drawn and still laid out.
    assert!(screen.query_by_test_id("box").is_none());
    assert!(screen.query_by_role(Role::Button, "Inside").is_none());
    assert!(screen.focused().is_none_or(|f| f.name() != "Inside"));
    screen.click_at(
        inside.x + inside.width / 2.0,
        inside.y + inside.height / 2.0,
    );
    assert_eq!(presses.get(), 0, "a leaving button takes no press");
    assert!(
        distance(probe(&screen), RED) <= 8,
        "the exit starts from its paint"
    );
    assert_eq!(
        below(&screen),
        resting + 80.0,
        "it keeps its place while it leaves"
    );
    screen.advance(ms(50));
    let mid = probe(&screen);
    assert!(
        distance(mid, page) > 40 && distance(mid, RED) > 40,
        "leaving: {mid:06x}"
    );
    screen.advance(ms(100));
    assert!(
        distance(probe(&screen), page) <= 8,
        "not drawn after its exit"
    );
    assert_eq!(below(&screen), resting, "and removed from the layout");
    let ticks = screen.frame_ticks();
    screen.advance(ms(200));
    assert_eq!(screen.frame_ticks(), ticks, "nothing moves any more");

    // Shown again: a new node enters again.
    shown.set(true);
    screen.run_until_idle();
    assert!(distance(probe(&screen), page) <= 8);
    screen.advance(ms(150));
    assert!(distance(probe(&screen), RED) <= 8);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn reduced_motion_mounts_and_removes_at_once() {
    let (screen, shown) = mount(move || {
        let shown = signal(false);
        let view = column((
            show(shown, || {
                node()
                    .style(style().size(160, 80).background(rgb(RED)))
                    .style(slide())
                    .test_id("box")
            }),
            text("below").test_id("below"),
        ))
        .style(style().align_items(kivact::style::Align::Start));
        (shown, view)
    });
    screen.set_reduced_motion(true);
    let resting = screen.get_by_test_id("below").rect().expect("laid out").y;
    shown.set(true);
    screen.run_until_idle();
    let r = screen.get_by_test_id("box").rect().expect("laid out");
    assert_eq!((r.x, r.width), (0.0, 160.0), "at its place at once");
    assert!(distance(centre(&screen, r), RED) <= 8);
    shown.set(false);
    screen.run_until_idle();
    assert_eq!(
        screen.get_by_test_id("below").rect().expect("laid out").y,
        resting,
        "removed at once"
    );
    assert!(distance(centre(&screen, r), RED) > 40);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// The standard recipes of the toast, the dialog surface, the select's
/// list and the tooltip with `slide`, on a red fill.
fn sliding_recipes() -> UiRecipes {
    fn red(s: Style) -> Style {
        s.background(rgb(RED)).merge(&slide())
    }
    UiRecipes::default()
        .toast(|table| table.toast(|prev| move |t: &Theme, l: &Look<_>| red(prev(t, l))))
        .dialog(|table| table.surface(|prev| move |t: &Theme, l: &Look<_>| red(prev(t, l))))
        .select(|table| table.surface(|prev| move |t: &Theme, l: &Look<_>| red(prev(t, l))))
        .tooltip(|table| table.surface(|prev| move |t: &Theme, l: &Look<_>| red(prev(t, l))))
}

/// `rect` of the element is 40 px right of where it ends at the start,
/// between the two after 50 ms and at its end after 150 ms.
#[track_caller]
fn slides_in(screen: &Screen, rect: impl Fn() -> Rect, what: &str) -> Rect {
    let start = rect();
    screen.advance(ms(50));
    let mid = rect();
    screen.advance(ms(100));
    let end = rect();
    assert!(
        (start.x - end.x - 40.0).abs() < 0.5,
        "{what} starts 40 px right: {start:?} {end:?}"
    );
    assert!(
        mid.x > end.x + 1.0 && mid.x < start.x - 1.0,
        "{what} on its way: {mid:?} {end:?}"
    );
    assert!(
        distance(beside(screen, end), RED) <= 8,
        "{what} arrives opaque"
    );
    end
}

/// The element drawn at `end` is gone from the accessibility tree, drawn
/// half way out after 50 ms and not drawn after 150 ms.
#[track_caller]
fn slides_out(screen: &Screen, end: Rect, gone: impl Fn() -> bool, what: &str) {
    assert!(gone(), "{what} leaves the accessibility tree at once");
    assert!(
        distance(beside(screen, end), RED) <= 8,
        "{what} is still drawn"
    );
    screen.advance(ms(50));
    let mid = beside(screen, end);
    assert!(distance(mid, RED) > 40, "{what} fades: {mid:06x}");
    screen.advance(ms(100));
    assert!(
        distance(beside(screen, end), RED) > 200,
        "{what} is not drawn after its exit"
    );
}

#[test]
fn the_standard_toast_dialog_popup_and_tooltip_enter_and_exit_by_their_recipes() {
    let options = RenderOptions {
        theme: Theme::light().with(sliding_recipes()),
        ..RenderOptions::from_env()
    };
    let (screen, (toaster, open)): (Screen, (Toaster, Signal<bool>)) = mount_with(options, || {
        let toaster = toaster();
        let open = signal(false);
        let fruit = signal(None::<&str>);
        let view = column((
            select(
                "Fruit",
                vec![
                    ListItem::new("apple", "Apple"),
                    ListItem::new("pear", "Pear"),
                ],
                fruit,
            ),
            tooltip("A tip").attach(button("Tipped")),
            dialog("Settings", open).content(|| text("Body")),
            toast_region(toaster),
        ))
        .style(style().align_items(kivact::style::Align::Start).gap(8));
        ((toaster, open), view)
    });

    // The Game kit's toast: in from the right, out to the right.
    let id = toaster.push(Toast::new("Saved"));
    screen.run_until_idle();
    let toast = || screen.get_by_text("Saved").rect().expect("laid out");
    let end = slides_in(&screen, toast, "the toast");
    toaster.dismiss(id);
    screen.run_until_idle();
    slides_out(
        &screen,
        end,
        || screen.query_by_text("Saved").is_none(),
        "the toast",
    );

    open.set(true);
    screen.run_until_idle();
    let surface = || {
        screen
            .get_by_role(Role::Heading, "Settings")
            .rect()
            .expect("laid out")
    };
    let end = slides_in(&screen, surface, "the dialog");
    screen.press_key(Key::Escape);
    assert!(!open.get_untracked());
    slides_out(
        &screen,
        end,
        || screen.query_by_role(Role::Dialog, "Settings").is_none(),
        "the dialog",
    );

    screen.get_by_role(Role::ComboBox, "Fruit").click();
    let list = || {
        screen
            .get_by_role(Role::Option, "Pear")
            .rect()
            .expect("laid out")
    };
    let end = slides_in(&screen, list, "the select's list");
    screen.press_key(Key::Escape);
    slides_out(
        &screen,
        end,
        || screen.query_by_role(Role::ListBox, "Fruit").is_none(),
        "the select's list",
    );

    screen.get_by_role(Role::Button, "Tipped").hover();
    screen.advance(ms(500));
    let tip = || screen.get_by_text("A tip").rect().expect("laid out");
    let end = slides_in(&screen, tip, "the tooltip");
    screen.mouse_leave();
    slides_out(
        &screen,
        end,
        || screen.query_by_role(Role::Tooltip, "A tip").is_none(),
        "the tooltip",
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// A scroll container dragged by its scrollbar, removed with an exit: the
/// drag ends with the removal, and the leaving container is drawn where
/// it was scrolled to.
#[test]
fn a_leaving_scroll_container_lets_go_of_its_scrollbar_drag() {
    const BLUE: u32 = 0x0000ff;
    let (screen, shown) = mount(move || {
        let shown = signal(true);
        let view = column(show(shown, || {
            node()
                .style(
                    style()
                        .size(100, 100)
                        .overflow(kivact::style::Overflow::Scroll)
                        .scrollbar_width(8)
                        .scrollbar_color(rgb(0x00ff00), rgb(0)),
                )
                .style(fade())
                .test_id("scroller")
                .child(column((
                    node().style(style().size(90, 250).shrink(0.0).background(rgb(RED))),
                    node().style(style().size(90, 250).shrink(0.0).background(rgb(BLUE))),
                )))
        }));
        (shown, view)
    });
    screen.advance(ms(150));
    let r = screen.get_by_test_id("scroller").rect().expect("laid out");
    let content = |screen: &Screen| {
        pixel(
            &screen.screenshot(),
            (r.x + 40.0) as u32,
            (r.y + 50.0) as u32,
        )
    };
    screen.mouse_move(r.x + 96.0, r.y + 8.0);
    screen.mouse_down(kivact::view::MouseButton::Primary);
    screen.mouse_move(r.x + 96.0, r.y + 50.0);
    assert!(
        distance(content(&screen), BLUE) <= 8,
        "dragged to the blue half"
    );
    shown.set(false);
    screen.run_until_idle();
    screen.mouse_move(r.x + 96.0, r.y + 8.0);
    assert!(
        distance(content(&screen), BLUE) <= 8,
        "the leaving container scrolls no more"
    );
    screen.mouse_up(kivact::view::MouseButton::Primary);
    screen.advance(ms(150));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// Removal leaves exit boxes in flow while keyed moves still filter overlays.
#[test]
fn keyed_reorders_with_overlays_keep_exit_space_until_removal_finishes() {
    let (screen, rows) = mount(|| {
        let rows = signal(vec![1u32, 2, 3, 4]);
        let view = column((
            for_each(
                rows,
                |id| *id,
                |id, _| {
                    let anchor = node_ref();
                    (
                        node()
                            .role(Role::Button)
                            .name(format!("exit sibling {id}"))
                            .focusable(true)
                            .node_ref(anchor)
                            .style(style().size(120, 24))
                            .style(fade()),
                        kivact::view::overlay(
                            anchor,
                            kivact::view::Placement::Center,
                            text(format!("overlay sibling {id}")),
                        ),
                    )
                },
            ),
            text("after siblings").test_id("after-siblings"),
        ));
        (rows, view)
    });
    screen.advance(ms(150));
    let below = || {
        screen
            .get_by_test_id("after-siblings")
            .rect()
            .expect("laid out")
            .y
    };
    let full = below();
    rows.set(vec![4, 3, 2, 1]);
    screen.run_until_idle();
    let positions: Vec<_> = [4, 3, 2, 1]
        .iter()
        .map(|id| {
            screen
                .get_by_role(Role::Button, &format!("exit sibling {id}"))
                .rect()
                .expect("laid out")
                .y
        })
        .collect();
    assert!(positions.windows(2).all(|pair| pair[0] < pair[1]));
    assert_eq!(below(), full, "overlays take no flow space");
    screen.get_by_role(Role::Button, "exit sibling 2").focus();
    rows.set(vec![3, 1]);
    screen.run_until_idle();
    assert!(
        screen
            .query_by_role(Role::Button, "exit sibling 2")
            .is_none()
    );
    assert!(screen.query_by_text("overlay sibling 2").is_none());
    assert!(
        screen
            .get_by_role(Role::Button, "exit sibling 1")
            .is_focused()
    );
    assert_eq!(below(), full, "both leaving rows retain their flow space");
    screen.advance(ms(150));
    assert_eq!(below(), full - 48.0, "exit completion unlinks both rows");
    rows.set(vec![6, 1, 3, 5]);
    screen.run_until_idle();
    screen.advance(ms(150));
    assert_eq!(below(), full, "fresh rows can reuse removed storage");
    screen.get_by_text("overlay sibling 6");
    screen.get_by_text("overlay sibling 5");
    rows.set(Vec::new());
    screen.run_until_idle();
    screen.advance(ms(150));
    assert_eq!(below(), full - 96.0);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
