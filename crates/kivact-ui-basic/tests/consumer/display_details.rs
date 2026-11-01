//! Role: Pixel and rect checks of alert, bubble reactions, button group, combobox field.

use super::{assert_clean, distance, pixel};
use kivact::draw::Rect;
use kivact::prelude::*;
use kivact::style::Align;
use kivact::test::Screen;
use kivact::ui::ButtonVariant;
use kivact_ui_basic as basic;

/// `--destructive` of the light theme.
const DESTRUCTIVE: u32 = 0xe7000b;

/// A column on the theme's background, holding `child` at its natural size.
fn page(child: impl IntoView) -> Node {
    column(child).style(
        style()
            .background(basic::Color::Background)
            .color(basic::Color::Foreground)
            .padding(24)
            .gap(16)
            .align_items(Align::Start),
    )
}

fn rect(screen: &Screen, id: &str) -> Rect {
    screen.get_by_test_id(id).rect().expect("laid out")
}

/// The colors of the last frame inside `r`, row by row.
fn colors_in(screen: &Screen, r: Rect) -> Vec<u32> {
    let mut out = Vec::new();
    let mut y = r.y.ceil();
    while y < r.y + r.height {
        let mut x = r.x.ceil();
        while x < r.x + r.width {
            out.push(pixel(screen, x, y));
            x += 1.0;
        }
        y += 1.0;
    }
    out
}

/// The darkest channel sum among `colors`: the core of the glyphs.
fn ink(colors: &[u32]) -> u32 {
    colors
        .iter()
        .copied()
        .min_by_key(|c| (c >> 16 & 0xff) + (c >> 8 & 0xff) + (c & 0xff))
        .expect("some pixels")
}

#[test]
fn a_destructive_alert_draws_its_title_and_description_in_the_destructive_color() {
    let (screen, ()) = kivact::test::render_with_handle(crate::options(), || {
        (
            (),
            page(
                basic::alert("Unable to process your payment.")
                    .variant(basic::AlertVariant::Destructive)
                    .icon(kivact::control::icon("circle-alert"))
                    .description("Please verify your billing information.")
                    .style(style().width(420)),
            ),
        )
    });
    for line in [
        "Unable to process your payment.",
        "Please verify your billing information.",
    ] {
        let r = screen.get_by_text(line).rect().expect("laid out");
        let core = ink(&colors_in(&screen, r));
        assert!(
            distance(core, DESTRUCTIVE) <= 48,
            "{line:?} is drawn in {core:06x}, not the destructive {DESTRUCTIVE:06x}"
        );
    }
    assert_clean(&screen);
}

#[test]
fn an_alert_action_is_centered_on_the_alert_height() {
    let (screen, ()) = kivact::test::render_with_handle(crate::options(), || {
        (
            (),
            page(
                basic::alert("Update available")
                    .icon(kivact::control::icon("download"))
                    .description("A new version is ready to install. It takes a minute.")
                    .action(move || {
                        button("Undo")
                            .variant(ButtonVariant::Outline)
                            .size(kivact::ui::ControlSize::Small)
                            .test_id("undo")
                    })
                    .test_id("alert")
                    .style(style().width(420)),
            ),
        )
    });
    let alert = rect(&screen, "alert");
    let undo = rect(&screen, "undo");
    let middle = |r: Rect| r.y + r.height / 2.0;
    assert!(
        (middle(alert) - middle(undo)).abs() <= 0.5,
        "the action's middle {} is the alert's {}",
        middle(undo),
        middle(alert)
    );
    assert!(
        alert.x + alert.width - (undo.x + undo.width) <= 6.0,
        "the action stays at the right edge"
    );
    assert_clean(&screen);
}

#[test]
fn bubble_reactions_are_a_pill_that_keeps_off_the_next_bubble() {
    let (screen, ()) = kivact::test::render_with_handle(crate::options(), || {
        let pill = |id: &'static str| {
            move || {
                row((text("👀"), text("+2")))
                    .role(Role::Image)
                    .name("Reactions: eyes and 2 more")
                    .test_id(id)
            }
        };
        (
            (),
            page(
                basic::bubble_group((
                    basic::bubble("Shipped the release!")
                        .variant(basic::BubbleVariant::Muted)
                        .reactions(pill("below"))
                        .test_id("first"),
                    basic::bubble("Great work")
                        .variant(basic::BubbleVariant::Muted)
                        .reactions_side(basic::BubbleReactionsSide::Top)
                        .reactions(pill("above"))
                        .test_id("second"),
                    basic::bubble("Thanks").test_id("third"),
                ))
                .style(style().width(360)),
            ),
        )
    });
    let (first, second, third) = (
        rect(&screen, "first"),
        rect(&screen, "second"),
        rect(&screen, "third"),
    );
    // The pill is the caller's row and the kit's 2 px of padding around it.
    let grow = |r: Rect| Rect::new(r.x - 6.0, r.y - 2.0, r.width + 12.0, r.height + 4.0);
    let (below, above) = (grow(rect(&screen, "below")), grow(rect(&screen, "above")));
    // `translate-y-3/4`: three quarters of the pill hang out of the bubble.
    let hang = below.y + below.height - (first.y + first.height);
    assert!(
        (hang - below.height * 0.75).abs() <= 0.5,
        "{hang} of {} hangs below",
        below.height
    );
    // The 2 px ring around the pill clears the next bubble.
    assert!(
        below.y + below.height + 2.0 <= second.y,
        "the pill below the first ({:?}) clears the second ({second:?})",
        below
    );
    assert!(
        above.y - 2.0 >= first.y + first.height,
        "the pill above the second ({above:?}) clears the first ({first:?})"
    );
    assert!(second.y + second.height <= third.y);
    // A muted pill: its fill is not the page's background.
    let page_fill = pixel(&screen, 4.0, 4.0);
    let fill = pixel(&screen, below.x + 2.0, below.y + 2.0);
    assert!(distance(fill, page_fill) > 4, "the pill has its own fill");
    assert_eq!(screen.get_by_test_id("below").role(), Some(Role::Image));
    assert_clean(&screen);
}

/// The columns of the row `y` from `x0` to `x1` whose color is not the
/// buttons' fill (the color at `x0`).
fn border_columns(screen: &Screen, x0: f32, x1: f32, y: f32) -> Vec<(u32, u32)> {
    let fill = pixel(screen, x0, y);
    let mut out = Vec::new();
    let mut x = x0.floor();
    while x <= x1 {
        let c = pixel(screen, x, y);
        if distance(c, fill) > 2 {
            out.push((x as u32, c));
        }
        x += 1.0;
    }
    out
}

#[test]
fn button_group_neighbours_share_one_line_like_the_outer_edges() {
    // The dark `--input` is translucent: a line drawn twice is darker.
    for mode in [
        kivact::style::ThemeMode::Light,
        kivact::style::ThemeMode::Dark,
    ] {
        joined_lines(mode);
    }
}

fn joined_lines(mode: kivact::style::ThemeMode) {
    let (screen, ()) = kivact::test::render_with_handle(
        kivact::test::RenderOptions {
            theme: basic::theme(mode),
            ..crate::options()
        },
        || {
            (
                (),
                page(basic::button_group((
                    button("Archive")
                        .variant(ButtonVariant::Outline)
                        .test_id("archive"),
                    button("Report")
                        .variant(ButtonVariant::Outline)
                        .test_id("report"),
                    button("Snooze")
                        .variant(ButtonVariant::Outline)
                        .test_id("snooze"),
                ))),
            )
        },
    );
    let archive = rect(&screen, "archive");
    let report = rect(&screen, "report");
    let snooze = rect(&screen, "snooze");
    // Every edge sits on a whole pixel, so each line covers one column.
    for r in [archive, report, snooze] {
        assert_eq!(r.x, r.x.round(), "{r:?} starts on a pixel");
        assert_eq!(r.width, r.width.round(), "{r:?} is whole pixels wide");
    }
    // Above the labels: only the lines differ from the buttons' fill.
    let y = archive.y + 4.0;
    let outer = pixel(&screen, archive.x, y);
    for edge in [report.x, snooze.x] {
        let lines = border_columns(&screen, edge - 4.0, edge + 4.0, y);
        assert_eq!(lines.len(), 1, "{mode:?}: one line at {edge}: {lines:x?}");
        assert!(
            distance(lines[0].1, outer) <= 2,
            "{mode:?}: the shared line {:06x} is the outer edge's {outer:06x}",
            lines[0].1
        );
    }
    assert_clean(&screen);
}

#[test]
fn a_button_group_separator_is_drawn_between_filled_buttons() {
    let (screen, ()) = kivact::test::render_with_handle(crate::options(), || {
        (
            (),
            page(basic::button_group((
                button("Copy")
                    .variant(ButtonVariant::Secondary)
                    .test_id("copy"),
                basic::button_group_separator().test_id("line"),
                button("Paste")
                    .variant(ButtonVariant::Secondary)
                    .test_id("paste"),
            ))),
        )
    });
    let copy = rect(&screen, "copy");
    let line = rect(&screen, "line");
    let paste = rect(&screen, "paste");
    assert_eq!(line.width, 1.0, "a 1 px line");
    // `my-px`: the line runs the group's height but one pixel at each end.
    assert!((line.height - (copy.height - 2.0)).abs() <= 0.5, "{line:?}");
    assert!(copy.x + copy.width <= line.x && line.x + line.width <= paste.x);
    let fill = pixel(&screen, copy.x + 4.0, copy.y + copy.height / 2.0);
    let drawn = pixel(&screen, line.x, line.y + line.height / 2.0);
    // `--input` of the light theme.
    let input = 0xe5e5e5;
    assert!(
        distance(drawn, input) <= 2,
        "the line is `--input` {input:06x}, drawn {drawn:06x} beside {fill:06x}"
    );
    assert_clean(&screen);
}

#[test]
fn button_group_text_matches_the_height_and_middle_of_its_button() {
    let (screen, ()) = kivact::test::render_with_handle(crate::options(), || {
        (
            (),
            page(basic::button_group((
                basic::button_group_text("https://").test_id("prefix"),
                button("Go").variant(ButtonVariant::Outline).test_id("go"),
            ))),
        )
    });
    let prefix = rect(&screen, "prefix");
    let go = rect(&screen, "go");
    assert_eq!(
        prefix.height, go.height,
        "the cell is as high as the button"
    );
    assert_eq!(prefix.y, go.y);
    let label = screen.get_by_text("https://").rect().expect("laid out");
    let middle = |r: Rect| r.y + r.height / 2.0;
    assert!(
        (middle(label) - middle(go)).abs() <= 0.5,
        "the text sits on the button's middle"
    );
    assert_clean(&screen);
}

#[test]
fn a_focused_combobox_draws_one_field_without_an_inner_box() {
    let (screen, ()) = kivact::test::render_with_handle(crate::options(), || {
        let items = vec![
            ListItem::new("next", "Next.js"),
            ListItem::new("astro", "Astro"),
        ];
        (
            (),
            page(
                basic::combobox(items, signal(None::<&'static str>))
                    .placeholder("Select a framework")
                    .name("Framework")
                    .test_id("framework")
                    .style(style().width(260)),
            ),
        )
    });
    let field = screen.get_by_role(Role::TextBox, "Framework");
    field.click();
    screen.run_until_idle();
    let group = screen
        .within(screen.get_by_test_id("framework"))
        .get_by_role(Role::Group, "")
        .rect()
        .expect("laid out");
    assert_eq!(group.height, 32.0, "`h-8`: the field is 32 px high");
    let input = field.rect().expect("laid out");
    // At the text box's right edge, inside the group: the field's fill, no
    // border or ring of the text box's own.
    let fill = pixel(&screen, input.x + input.width / 2.0, input.y + 3.0);
    for (x, y) in [
        (input.x + input.width - 1.0, input.y + input.height / 2.0),
        (input.x + input.width, input.y + input.height / 2.0),
        (input.x + input.width - 1.0, input.y + 4.0),
    ] {
        let c = pixel(&screen, x, y);
        assert!(
            distance(c, fill) <= 2,
            "no inner box at ({x}, {y}): {c:06x} on {fill:06x}"
        );
    }
    assert_clean(&screen);
}
