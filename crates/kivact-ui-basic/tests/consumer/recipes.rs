//! Role: The Lyra recipes read what the standard components hand them.
//!
//! Tab orientation, control sizes, the tooltip arrow, popup triggers and
//! button icons, the growing text area, invalid toggles and the stroked
//! check box, all on real windows.

use super::{assert_clean, distance, pixel};
use kivact::prelude::*;
use kivact::primitive::NodeExt;
use kivact::style::Align;
use kivact::test::Screen;
use kivact::ui::{ButtonVariant, ControlSize, ListItem};
use kivact::view::MouseButton;
use kivact_ui_basic as basic;
use std::time::Duration;

/// A page painted with the `background` token of the theme in effect.
fn page(child: impl IntoView) -> Node {
    column(child).style(
        style()
            .background(basic::Color::Background)
            .padding(24)
            .gap(12)
            .grow(1)
            .align_items(Align::Start),
    )
}

fn rect(screen: &Screen, id: &str) -> kivact::draw::Rect {
    screen.get_by_test_id(id).rect().expect("laid out")
}

#[test]
fn vertical_standard_tabs_stack_full_width_tabs_beside_their_panel() {
    let (screen, ()) = kivact::test::render_with_handle(crate::options(), || {
        let options = vec![
            ListItem::new(1, "Account"),
            ListItem::new(2, "Password settings"),
        ];
        (
            (),
            page(
                kivact::ui::tabs("Tabs", options, signal(1))
                    .orientation(kivact::view::Orientation::Vertical)
                    .test_id("list")
                    .content(|key| text(format!("Panel {key}"))),
            ),
        )
    });
    let list = rect(&screen, "list");
    let tab = |name| {
        screen
            .get_by_role(Role::Tab, name)
            .rect()
            .expect("laid out")
    };
    let (first, second) = (tab("Account"), tab("Password settings"));
    assert!(
        list.height > 32.0,
        "the list is not squeezed to a row: {list:?}"
    );
    assert_eq!(first.width, second.width, "every tab spans the list");
    assert_eq!(first.width, list.width - 6.0, "inside the 3 px padding");
    assert!(
        second.y >= first.y + first.height,
        "stacked: {first:?} {second:?}"
    );
    let panel = screen
        .get_by_role(Role::TabPanel, "Account")
        .rect()
        .expect("laid out");
    assert!(
        panel.x >= list.x + list.width,
        "the panel is beside the list"
    );
    assert_clean(&screen);
    screen.assert_layout();
}

#[test]
fn sizes_line_up_across_buttons_inputs_and_selects() {
    let (screen, ()) = kivact::test::render_with_handle(crate::options(), || {
        let options = vec![ListItem::new(1, "Apple")];
        let field = |size, id: &str| {
            kivact::ui::text_input(id.to_owned(), signal(String::new()))
                .size(size)
                .test_id(id)
        };
        let select = |size, id: &str| {
            kivact::ui::select(id.to_owned(), options.clone(), signal(None::<u32>))
                .size(size)
                .test_id(id)
        };
        (
            (),
            page((
                kivact::ui::button("XS")
                    .size(ControlSize::ExtraSmall)
                    .test_id("button-xs"),
                kivact::ui::button("SM")
                    .size(ControlSize::Small)
                    .test_id("button-sm"),
                field(ControlSize::ExtraSmall, "input-xs"),
                field(ControlSize::Small, "input-sm"),
                select(ControlSize::ExtraSmall, "select-xs"),
                select(ControlSize::Small, "select-sm"),
                select(ControlSize::Medium, "select-md"),
            )),
        )
    });
    for (id, height) in [
        ("button-xs", 24.0),
        ("button-sm", 28.0),
        ("input-xs", 24.0),
        ("input-sm", 28.0),
        ("select-xs", 24.0),
        ("select-sm", 28.0),
        ("select-md", 32.0),
    ] {
        assert_eq!(rect(&screen, id).height, height, "{id}");
    }
    assert_clean(&screen);
}

#[test]
fn a_popup_trigger_stays_put_while_pressed_and_shows_that_it_is_open() {
    let (screen, open) = kivact::test::render_with_handle(crate::options(), || {
        let open = signal(false);
        let popup = kivact::interaction::use_popup(open);
        (
            open,
            page((
                kivact::ui::button("Plain")
                    .variant(ButtonVariant::Outline)
                    .test_id("plain"),
                kivact::ui::button("Opens")
                    .variant(ButtonVariant::Outline)
                    .part(popup.trigger())
                    .test_id("trigger"),
            )),
        )
    });
    let press = |id: &str| {
        let rest = rect(&screen, id);
        screen.mouse_move(rest.x + 4.0, rest.y + rest.height / 2.0);
        screen.mouse_down(MouseButton::Primary);
        let held = rect(&screen, id);
        screen.mouse_up(MouseButton::Primary);
        held.y - rest.y
    };
    assert_eq!(
        press("plain"),
        1.0,
        "a plain button nudges down (active:translate-y-px)"
    );
    assert_eq!(press("trigger"), 0.0, "a popup trigger does not");
    open.set(true);
    screen.run_until_idle();
    screen.mouse_move(500.0, 500.0);
    let trigger = rect(&screen, "trigger");
    let fill = pixel(&screen, trigger.x + 4.0, trigger.y + 4.0);
    assert!(
        distance(fill, 0xf5f5f5) <= 2,
        "aria-expanded:bg-muted: {fill:06x}"
    );
    let plain = rect(&screen, "plain");
    let rest = pixel(&screen, plain.x + 4.0, plain.y + 4.0);
    assert!(
        distance(rest, 0xffffff) <= 2,
        "a closed outline button: {rest:06x}"
    );
}

#[test]
fn an_icon_tightens_the_padding_on_its_side() {
    let (screen, ()) = kivact::test::render_with_handle(crate::options(), || {
        (
            (),
            page((
                kivact::ui::button("Download")
                    .icon(kivact::control::icon("download").test_id("lead"))
                    .test_id("leading"),
                kivact::ui::button("Next")
                    .trailing_icon(kivact::control::icon("chevron-right").test_id("trail"))
                    .test_id("trailing"),
                kivact::ui::button("Add")
                    .size(ControlSize::Small)
                    .icon(kivact::control::icon("plus").test_id("small-lead"))
                    .test_id("small"),
            )),
        )
    });
    let (leading, lead) = (rect(&screen, "leading"), rect(&screen, "lead"));
    // The 1 px border, then the padding (CSS `box-sizing: border-box`).
    assert_eq!(
        lead.x - leading.x,
        9.0,
        "border + has-data-[icon=inline-start]:pl-2"
    );
    let (trailing, trail) = (rect(&screen, "trailing"), rect(&screen, "trail"));
    assert_eq!(
        trailing.x + trailing.width - (trail.x + trail.width),
        9.0,
        "border + has-data-[icon=inline-end]:pr-2"
    );
    let (small, small_lead) = (rect(&screen, "small"), rect(&screen, "small-lead"));
    assert_eq!(
        small_lead.x - small.x,
        7.0,
        "border + pl-1.5 in a small button"
    );

    assert_clean(&screen);
}

#[test]
fn a_tooltip_draws_its_arrow_toward_the_trigger() {
    let (screen, ()) = kivact::test::render_with_handle(crate::options(), || {
        (
            (),
            page(
                column(
                    kivact::ui::tooltip("Add to library")
                        .attach(kivact::ui::button("Hover").test_id("anchor")),
                )
                .style(style().padding_top(80).padding_left(80)),
            ),
        )
    });
    screen.get_by_test_id("anchor").focus();
    screen.advance(Duration::from_millis(500));
    let anchor = rect(&screen, "anchor");
    let tip = screen
        .get_by_role(Role::Tooltip, "Add to library")
        .rect()
        .expect("shown");
    assert!(tip.y + tip.height <= anchor.y, "above the trigger");
    let centre = anchor.x + anchor.width / 2.0;
    let arrow = pixel(&screen, centre, tip.y + tip.height + 2.0);
    assert!(
        distance(arrow, 0x0a0a0a) <= 16,
        "the arrow below the tip: {arrow:06x}"
    );
    let beside = pixel(&screen, tip.x + 2.0, tip.y + tip.height + 2.0);
    assert!(
        distance(beside, 0xffffff) <= 2,
        "nothing beside the arrow: {beside:06x}"
    );
    assert_clean(&screen);
}

#[test]
fn a_text_area_grows_with_its_lines_from_its_minimum_height() {
    let (screen, notes) = kivact::test::render_with_handle(crate::options(), || {
        let notes = signal(String::new());
        (
            notes,
            page(kivact::ui::text_area("Notes", notes).test_id("notes")),
        )
    });
    let empty = rect(&screen, "notes").height;
    assert!(empty >= 64.0, "min-h-16: {empty}");
    notes.set("1\n2\n3\n4\n5\n6".to_owned());
    screen.run_until_idle();
    let six = rect(&screen, "notes").height;
    assert!(
        six >= empty + 3.0 * 16.0 - 0.5,
        "six lines grow it: {empty} -> {six}"
    );
    assert_clean(&screen);
}

#[test]
fn invalid_toggles_draw_a_destructive_edge() {
    let (screen, ()) = kivact::test::render_with_handle(crate::options(), || {
        let off = signal(false);
        (
            (),
            page((
                kivact::ui::checkbox("Valid", off).test_id("valid"),
                kivact::ui::checkbox("Terms", off)
                    .invalid(true)
                    .test_id("terms"),
                kivact::ui::switch("Alerts", off)
                    .invalid(true)
                    .test_id("alerts"),
            )),
        )
    });
    // The 1 px border of the drawn part, 1 px inside the row's left edge.
    let edge = |id: &str| {
        let r = rect(&screen, id);
        pixel(&screen, r.x + 1.5, r.y + r.height / 2.0)
    };
    assert!(distance(edge("valid"), 0xe5e5e5) <= 8, "border-input");
    assert!(
        distance(edge("terms"), 0xe7000b) <= 24,
        "aria-invalid:border-destructive"
    );
    assert!(
        distance(edge("alerts"), 0xe7000b) <= 48,
        "an invalid switch"
    );
    assert!(screen.get_by_role(Role::CheckBox, "Terms").invalid());
    assert_clean(&screen);
}

#[test]
fn an_unchecked_check_box_shows_what_is_behind_it() {
    let (screen, ()) = kivact::test::render_with_handle(crate::options(), || {
        (
            (),
            column(kivact::ui::checkbox("Shows through", signal(false)).test_id("box"))
                .style(style().padding(24).background(rgb(0x3366ff))),
        )
    });
    let r = rect(&screen, "box");
    // The middle of the 16 px box (1 px after the row's edge).
    let inside = pixel(&screen, r.x + 9.0, r.y + r.height / 2.0);
    assert!(
        distance(inside, 0x3366ff) <= 4,
        "bg-transparent: {inside:06x}"
    );
}

#[test]
fn a_focused_tab_panel_draws_no_ring() {
    let (screen, ()) = kivact::test::render_with_handle(crate::options(), || {
        let options = vec![ListItem::new(1, "Account"), ListItem::new(2, "Password")];
        (
            (),
            page(
                kivact::ui::tabs("Tabs", options, signal(1))
                    .content(|_| text("Make changes to your account here.")),
            ),
        )
    });
    screen.press_tab(false);
    screen.press_tab(false);
    let panel = screen.get_by_role(Role::TabPanel, "Account");
    assert!(panel.is_focused(), "Tab reaches the panel");
    let r = panel.rect().expect("laid out");
    let outside = pixel(&screen, r.x + r.width / 2.0, r.y + r.height + 0.5);
    assert!(
        distance(outside, 0xffffff) <= 2,
        "outline-none: {outside:06x}"
    );
    assert_clean(&screen);
}

#[test]
fn a_button_label_keeps_its_line_inside_a_relaxed_parent() {
    let (screen, ()) = kivact::test::render_with_handle(crate::options(), || {
        (
            (),
            page(
                column(kivact::ui::button("Save").test_id("save"))
                    .style(style().font_size(12).line_height(1.625)),
            ),
        )
    });
    let button = rect(&screen, "save");
    let label = screen
        .within(screen.get_by_test_id("save"))
        .get_by_text("Save")
        .rect()
        .expect("laid out");
    assert_eq!(label.height, 16.0, "text-xs keeps a 16 px line");
    assert_eq!(
        label.y - button.y,
        (button.height - label.height) / 2.0,
        "the label sits in the middle"
    );
    assert_clean(&screen);
}

#[test]
fn a_focused_check_box_rings_its_box_not_its_row_and_a_pressable_badge_lights_on_hover() {
    let (screen, ()) = kivact::test::render_with_handle(crate::options(), || {
        (
            (),
            page((
                kivact::ui::checkbox("Remember me on this device", signal(false)).test_id("box"),
                basic::badge("Pressable").on_press(|| {}).test_id("badge"),
            )),
        )
    });
    // Tab gives the check box visible focus: its 16 px box rings, while the
    // end of its row stays on the page (no ring around the whole row).
    screen.press_tab(false);
    assert!(
        screen
            .get_by_role(Role::CheckBox, "Remember me on this device")
            .is_focused()
    );
    let r = rect(&screen, "box");
    let row_end = pixel(&screen, r.x + r.width + 0.5, r.y + r.height / 2.0);
    assert!(
        distance(row_end, 0xffffff) <= 2,
        "no ring around the row: {row_end:06x}"
    );
    let halo = pixel(&screen, r.x + 0.5, r.y + r.height / 2.0);
    assert!(distance(halo, 0xffffff) > 8, "the box's halo: {halo:06x}");
    // A pressable default badge is `bg-primary/80` while hovered.
    let b = rect(&screen, "badge");
    let at_rest = pixel(&screen, b.x + 3.0, b.y + b.height / 2.0);
    screen.get_by_test_id("badge").hover();
    let hovered = pixel(&screen, b.x + 3.0, b.y + b.height / 2.0);
    assert!(distance(at_rest, 0x171717) <= 2, "{at_rest:06x}");
    assert!(
        distance(hovered, 0x171717) > 8,
        "lighter while hovered: {hovered:06x}"
    );
    assert_clean(&screen);
}
