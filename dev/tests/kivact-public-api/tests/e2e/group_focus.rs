//! Role: Tests for the group focus state style of a node below a group.
//!
//! `Style::group_focus`: a node is styled while the focus is on the nearest
//! ancestor made a group or on a node below it (CSS `:focus-within`).

use crate::support::mount;
use kivact::prelude::*;

/// A row of a button and an actions cell hidden until the focus is inside
/// the row (its group); the nested group follows its own focus, and a row
/// without the focus inside stays hidden.
#[test]
fn group_focus_styles_a_child_by_the_focus_inside_its_group() {
    use crate::paint::{distance, pixel};
    use kivact::style::rgb;
    const ACTIONS: u32 = 0xff0000;
    let (screen, ()) = mount(|| {
        let cell = |id: &'static str| node().style(style().size(100, 40)).test_id(id);
        let button = |id: &'static str| {
            cell(id)
                .role(Role::Button)
                .focusable(true)
                .name(id)
                .style(style().size(100, 40))
        };
        let actions = |id: &'static str| {
            cell(id).style(
                style()
                    .background(rgb(ACTIONS))
                    .opacity(0.0)
                    .group_focus(style().opacity(1.0)),
            )
        };
        let view = column((
            row((
                actions("actions"),
                button("name"),
                // A nested group: what is below it follows the focus in it.
                row((actions("inner"), button("inner button")))
                    .style(style().group())
                    .test_id("nested"),
            ))
            .style(style().group().background(rgb(0xffffff)))
            .test_id("row"),
            row((actions("other actions"), button("other button")))
                .style(style().group())
                .test_id("other row"),
            button("outside"),
        ));
        ((), view)
    });
    let at = |id: &str| {
        let r = screen.get_by_test_id(id).rect().expect("laid out");
        pixel(
            &screen.screenshot(),
            (r.x + 50.0) as u32,
            (r.y + 20.0) as u32,
        )
    };
    let shown = |id: &str| distance(at(id), ACTIONS) <= 8;
    let all = ["actions", "inner", "other actions"];
    let states = || all.map(&shown);
    assert_eq!(states(), [false; 3], "hidden until the focus is inside");
    screen.get_by_test_id("name").click();
    assert_eq!(states(), [true, false, false], "a button of the row");
    screen.get_by_test_id("inner button").click();
    assert_eq!(
        states(),
        [true, true, false],
        "the outer group holds the focus inside its nested one too"
    );
    screen.get_by_test_id("other button").click();
    assert_eq!(
        states(),
        [false, false, true],
        "the focus moved to another row"
    );
    screen.press_tab(false);
    assert_eq!(states(), [false; 3], "the focus left every group");
    screen.press_tab(true);
    assert_eq!(states(), [false, false, true], "back with Shift+Tab");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// The focused button removed from its row: the focus moves out of the
/// row, and the row's group focus ends with it.
#[test]
fn removing_the_focused_node_ends_its_groups_focus() {
    use crate::paint::{distance, pixel};
    use kivact::style::rgb;
    const ACTIONS: u32 = 0xff0000;
    let (screen, shown) = mount(|| {
        let shown = signal(true);
        let view = column((
            row((
                node()
                    .style(
                        style()
                            .size(100, 40)
                            .background(rgb(ACTIONS))
                            .opacity(0.0)
                            .group_focus(style().opacity(1.0))
                            .transition(
                                kivact::style::TransitionProp::Paint,
                                kivact::motion::Transition::tween(
                                    100,
                                    kivact::motion::Easing::Linear,
                                ),
                            ),
                    )
                    .test_id("actions"),
                show(shown, || {
                    node()
                        .role(Role::Button)
                        .focusable(true)
                        .name("inside")
                        .style(style().size(100, 40))
                }),
            ))
            .style(style().group().background(rgb(0xffffff))),
            node()
                .role(Role::Button)
                .focusable(true)
                .name("outside")
                .style(style().size(100, 40)),
        ));
        (shown, view)
    });
    let at = || {
        let r = screen.get_by_test_id("actions").rect().expect("laid out");
        pixel(
            &screen.screenshot(),
            (r.x + 50.0) as u32,
            (r.y + 20.0) as u32,
        )
    };
    let ms = std::time::Duration::from_millis;
    screen.get_by_role(Role::Button, "inside").click();
    screen.advance(ms(150));
    assert!(distance(at(), ACTIONS) <= 8, "the focus is inside the row");
    shown.set(false);
    screen.run_until_idle();
    assert_eq!(screen.focused().map(|f| f.name()), Some("outside".into()));
    screen.advance(ms(50));
    let mid = at();
    assert!(
        distance(mid, ACTIONS) > 40 && distance(mid, 0xffffff) > 40,
        "fading out with the removal: {mid:06x}"
    );
    screen.advance(ms(100));
    assert!(distance(at(), 0xffffff) <= 8, "the focus left the row");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// A row made a group while the focus is inside it, and made none again:
/// what is below it follows at once.
#[test]
fn a_group_made_or_unmade_around_the_focus_restyles_what_is_below_it() {
    use crate::paint::{distance, pixel};
    use kivact::style::rgb;
    const ACTIONS: u32 = 0xff0000;
    let (screen, grouped) = mount(|| {
        let grouped = signal(false);
        let view = row((
            node()
                .style(
                    style()
                        .size(100, 40)
                        .background(rgb(ACTIONS))
                        .opacity(0.0)
                        .group_focus(style().opacity(1.0))
                        .transition(
                            kivact::style::TransitionProp::Paint,
                            kivact::motion::Transition::tween(100, kivact::motion::Easing::Linear),
                        ),
                )
                .test_id("actions"),
            node()
                .role(Role::Button)
                .focusable(true)
                .name("inside")
                .style(style().size(100, 40)),
        ))
        .style(move || {
            let base = style().background(rgb(0xffffff));
            if grouped.get() { base.group() } else { base }
        });
        (grouped, view)
    });
    let at = || {
        let r = screen.get_by_test_id("actions").rect().expect("laid out");
        pixel(
            &screen.screenshot(),
            (r.x + 50.0) as u32,
            (r.y + 20.0) as u32,
        )
    };
    let ms = std::time::Duration::from_millis;
    let moving = |mid: u32| distance(mid, ACTIONS) > 40 && distance(mid, 0xffffff) > 40;
    screen.get_by_role(Role::Button, "inside").click();
    screen.advance(ms(150));
    assert!(distance(at(), 0xffffff) <= 8, "no group yet");
    grouped.set(true);
    screen.run_until_idle();
    screen.advance(ms(50));
    assert!(moving(at()), "fading in: {:06x}", at());
    screen.advance(ms(100));
    assert!(
        distance(at(), ACTIONS) <= 8,
        "made a group around the focus"
    );
    grouped.set(false);
    screen.run_until_idle();
    screen.advance(ms(50));
    assert!(moving(at()), "fading out: {:06x}", at());
    screen.advance(ms(100));
    assert!(distance(at(), 0xffffff) <= 8, "no group any more");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
