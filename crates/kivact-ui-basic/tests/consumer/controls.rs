//! Role: Standard component wiring: a code block's copy, pagination, field, text input,
//! navigation menu, line tabs and sheet, each driven through its public parts.

use crate::support::{distance, pixel, pixel_in};
use kivact::geometry::Edge;

use kivact::prelude::*;
use kivact::test::LayoutCheck;
use kivact::ui::{DialogKind, PageItem};
use kivact::view::Node;
use kivact_ui_basic as basic;
use std::time::Duration;

/// A page painted with the `background` token of the theme in effect.
fn page(child: impl IntoView) -> Node {
    column(child).style(
        style()
            .background(basic::BACKGROUND)
            .color(basic::FOREGROUND)
            .padding(16)
            .gap(16)
            .flex_grow(1)
            .align_items(Align::Start),
    )
}

/// The 23 px line of the default size.
const LINE: f64 = 23.0;

#[test]
fn code_block_reports_copy_requests_and_shows_copied_for_two_seconds() {
    let (screen, copies) = kivact::test::render_with_handle(
        kivact::test::RenderOptions::with_theme(basic::theme(kivact::style::ThemeMode::Light))
            .reduced_motion(),
        || {
            let copies = signal(0usize);
            (
                copies,
                page(
                    kivact::ui::code_block("let a = 1;\n\nlet b = 2;".to_string())
                        .title("main.rs")
                        .line_numbers(true)
                        .on_copy(move || copies.update(|n| *n += 1))
                        .footer(text("Rust"))
                        .test_id("code")
                        .style(style().width(400)),
                ),
            )
        },
    );
    screen.get_by_text("main.rs");
    screen.get_by_text("Rust");
    // One number per line, the blank second line as high as the others.
    let numbers = screen.get_by_text("1\n2\n3").rect().expect("laid out");
    let code = screen.get_by_role(Role::TextBox, "main.rs");
    let rect = code.rect().expect("laid out");
    assert_eq!(rect.height, 3.0 * LINE);
    assert_eq!(numbers.y, rect.y);
    assert_eq!(numbers.height, rect.height);
    assert_eq!(code.value().as_deref(), Some("let a = 1;\n\nlet b = 2;"));
    screen.get_by_role(Role::Button, "Copy").click();
    assert_eq!(copies.get_untracked(), 1);
    screen.get_by_role(Role::Button, "Copied");
    screen.get_by_role(Role::Status, "Copied");
    screen.advance(Duration::from_millis(1900));
    screen.get_by_role(Role::Button, "Copied");
    screen.advance(Duration::from_millis(200));
    screen
        .get_by_role(Role::Button, "Copy")
        .press_key(Key::Enter);
    assert_eq!(copies.get_untracked(), 2);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    screen.assert_layout_clean();
}

#[test]
fn pagination_pages_follows_its_binding_and_disables_the_ends() {
    let (screen, page_index) = kivact::test::render_with_handle(
        kivact::test::RenderOptions::with_theme(basic::theme(kivact::style::ThemeMode::Light))
            .window_size(900.0, 3200.0)
            .reduced_motion(),
        || {
            let index = signal(0usize);
            (
                (index),
                page(
                    kivact::ui::pagination(10usize, index).item(|cell| match cell {
                        PageItem::Page(n) => text((n + 1).to_string()).into_view(),
                        PageItem::Previous => text("Previous").into_view(),
                        PageItem::Next => text("Next").into_view(),
                        _ => text("…").name("More pages").into_view(),
                    }),
                ),
            )
        },
    );
    let previous = || screen.get_by_role(Role::Button, "Previous page");
    let next = || screen.get_by_role(Role::Button, "Next page");
    assert!(previous().is_disabled());
    assert!(!next().is_disabled());
    assert!(screen.get_by_role(Role::Button, "1").is_selected());
    // First, current + 1, last, with one gap between.
    for shown in ["1", "2", "10"] {
        screen.get_by_role(Role::Button, shown);
    }
    assert!(screen.query_by_role(Role::Button, "5").is_none());
    assert_eq!(screen.all_by_text("…").len(), 1);
    next().click();
    assert_eq!(page_index.get_untracked(), 1);
    assert!(!previous().is_disabled());
    assert!(screen.get_by_role(Role::Button, "2").is_selected());
    screen.get_by_role(Role::Button, "10").click();
    assert_eq!(page_index.get_untracked(), 9);
    assert!(next().is_disabled());
    next().force_click();
    assert_eq!(
        page_index.get_untracked(),
        9,
        "a disabled link does not move"
    );
    previous().click();
    assert_eq!(page_index.get_untracked(), 8);
    // Two gaps in the middle; the current page and its neighbours show.
    page_index.set(5);
    screen.run_until_idle();
    for shown in ["1", "5", "6", "7", "10"] {
        screen.get_by_role(Role::Button, shown);
    }
    assert_eq!(screen.all_by_text("…").len(), 2);
    assert!(screen.get_by_role(Role::Button, "6").is_selected());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    screen.assert_layout_clean();
}

#[test]
fn field_shows_label_description_and_error() {
    let (screen, error) = kivact::test::render_with_handle(
        kivact::test::RenderOptions::with_theme(basic::theme(kivact::style::ThemeMode::Light))
            .reduced_motion(),
        || {
            let name = signal(String::new());
            let error = signal(None::<String>);
            (
                error,
                page(
                    kivact::ui::field("Username", kivact::ui::text_input(name).label("Username"))
                        .hint("Choose a unique username.")
                        .error(error),
                ),
            )
        },
    );
    let group = screen.get_by_role(Role::TextBox, "Username");
    assert!(group.exists());
    screen.get_by_text("Username");
    screen.get_by_text("Choose a unique username.");
    assert!(screen.query_by_role(Role::Alert, "Required").is_none());
    error.set(Some("Required".into()));
    screen.run_until_idle();
    screen.get_by_role(Role::Alert, "Required");
    screen.assert_layout();
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    screen.assert_layout_clean();
}

#[test]
fn text_input_edits_its_binding_and_rings_on_focus() {
    let (screen, (name, notes)) = kivact::test::render_with_handle(
        kivact::test::RenderOptions::with_theme(basic::theme(kivact::style::ThemeMode::Light))
            .reduced_motion(),
        || {
            let name = signal(String::new());
            let notes = signal(String::new());
            (
                (name, notes),
                page((
                    kivact::ui::text_input(name)
                        .label("Name")
                        .placeholder("Your name")
                        .test_id("name"),
                    kivact::ui::text_input(signal(String::new()))
                        .label("Code")
                        .size(ControlSize::Small)
                        .test_id("code"),
                    kivact::ui::text_area(notes).label("Notes").test_id("notes"),
                )),
            )
        },
    );
    let rect = |id: &str| screen.get_by_test_id(id).rect().expect("laid out");
    assert_eq!(rect("name").height, 32.0);
    assert_eq!(rect("code").height, 28.0);
    // Three lines (the engine's least for a multiline input) of 16 px,
    // `py-2` and the 1 px border: 2 px over shadcn's `min-h-16`, whose
    // `field-sizing: content` starts at one line.
    assert_eq!(rect("notes").height, 66.0);

    assert!(distance(pixel_in(&screen, "name", 0.0), 0xe5e5e5) <= 4);
    let field = screen.get_by_role(Role::TextBox, "Name");
    field.click();
    field.type_text("Ada");
    assert_eq!(name.get_untracked(), "Ada");
    assert!(
        distance(pixel_in(&screen, "name", 0.0), 0xa1a1a1) <= 4,
        "the focused border is the ring color: {:06x}",
        pixel_in(&screen, "name", 0.0)
    );
    let area = screen.get_by_role(Role::TextBox, "Notes");
    area.click();
    area.type_text("Line");
    assert_eq!(notes.get_untracked(), "Line");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    screen.assert_layout_clean();
}

#[test]
fn navigation_menu_shows_one_panel_at_a_time() {
    let screen = kivact::test::render_with_handle(
        kivact::test::RenderOptions::with_theme(basic::theme(kivact::style::ThemeMode::Light))
            .window_size(900.0, 700.0)
            .reduced_motion(),
        || {
            (
                (),
                page(
                    kivact::ui::navigation_menu("Navigation")
                        .link("alpha", "Alpha", || text("Alpha panel"))
                        .link("beta", "Beta", || text("Beta panel"))
                        .close_delay(std::time::Duration::ZERO),
                ),
            )
        },
    )
    .0;
    let alpha = screen.get_by_role(Role::Button, "Alpha");
    let beta = screen.get_by_role(Role::Button, "Beta");
    let shown = |text: &str| screen.query_by_text(text).is_some();
    // Hovering a trigger shows its panel, and the trigger reports it.
    alpha.hover();
    assert!(shown("Alpha panel"));
    assert_eq!(alpha.is_expanded(), Some(true));
    // Escape on the focused trigger closes a hover-opened panel.
    alpha.focus();
    screen.press_key(Key::Escape);
    assert!(!shown("Alpha panel"));
    assert_eq!(alpha.is_expanded(), Some(false));
    // Hovering the next trigger replaces the panel: one is open at a time.
    screen.mouse_move(850.0, 650.0);
    beta.hover();
    assert!(shown("Beta panel"));
    alpha.hover();
    assert!(shown("Alpha panel"));
    assert!(!shown("Beta panel"));
    assert_eq!(alpha.is_expanded(), Some(true));
    assert_eq!(beta.is_expanded(), Some(false));
    // ArrowDown on a trigger pins its panel.
    screen.mouse_move(850.0, 650.0);
    assert!(!shown("Alpha panel"));
    alpha.focus();
    screen.press_key(Key::ArrowRight);
    assert!(beta.is_focused());
    screen.press_key(Key::ArrowDown);
    assert!(shown("Beta panel"));
    assert!(!shown("Alpha panel"));
    assert_eq!(alpha.is_expanded(), Some(false));
    assert_eq!(beta.is_expanded(), Some(true));
    screen.press_key(Key::Escape);
    assert!(!shown("Beta panel"));
    assert_eq!(beta.is_expanded(), Some(false));
    // A click on a hover-opened panel pins it: it stays after the pointer leaves.
    alpha.hover();
    alpha.click();
    screen.mouse_move(850.0, 650.0);
    assert!(shown("Alpha panel"));
    assert_eq!(alpha.is_expanded(), Some(true));
    screen.press_key(Key::Escape);
    assert!(!shown("Alpha panel") && !shown("Beta panel"));
    assert_eq!(alpha.is_expanded(), Some(false));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    screen.assert_layout_clean();
}

#[test]
fn line_tabs_select_move_and_mark_the_selected_tab_with_an_underline() {
    let (screen, current) = kivact::test::render_with_handle(
        kivact::test::RenderOptions::with_theme(basic::theme(kivact::style::ThemeMode::Light))
            .window_size(900.0, 700.0)
            .reduced_motion(),
        || {
            let current = signal("overview");
            (
                current,
                page(
                    tabs(
                        "Tabs",
                        vec![
                            ListItem::new("overview", "Overview"),
                            ListItem::new("analytics", "Analytics"),
                            ListItem::new("reports", "Reports").disabled(true),
                            ListItem::new("settings", "Settings"),
                        ],
                        current,
                    )
                    .variant(basic::TabsVariant::Line)
                    .content(|key| text(format!("Panel {key}"))),
                ),
            )
        },
    );
    screen.get_by_role(Role::TabList, "Tabs");
    let overview = screen.get_by_role(Role::Tab, "Overview");
    let analytics = screen.get_by_role(Role::Tab, "Analytics");
    assert!(overview.is_selected());
    assert!(!analytics.is_selected());
    assert!(screen.get_by_role(Role::Tab, "Reports").is_disabled());
    screen.get_by_text("Panel overview");
    // The underline sits under the selected tab only.
    let under = |tab: &kivact::test::Locator<'_>| {
        let rect = tab.rect().expect("laid out");
        pixel(
            &screen,
            rect.x + rect.width / 2.0,
            rect.y + rect.height + 3.5,
        )
    };
    assert!(distance(under(&overview), 0x0a0a0a) <= 4, "selected");
    assert!(distance(under(&analytics), 0xffffff) <= 4, "not selected");
    // A line tab has no filled background, selected or not.
    let rect = overview.rect().expect("laid out");
    assert!(
        distance(
            pixel(&screen, rect.x + 3.0, rect.y + rect.height / 2.0),
            0xffffff
        ) <= 2
    );

    analytics.click();
    assert_eq!(current.get_untracked(), "analytics");
    screen.get_by_text("Panel analytics");
    assert!(distance(under(&analytics), 0x0a0a0a) <= 4);
    assert!(distance(under(&overview), 0xffffff) <= 4);
    // Arrows skip the disabled tab and the selection follows the focus.
    analytics.focus();
    analytics.press_key(Key::ArrowRight);
    assert_eq!(current.get_untracked(), "settings");
    screen.press_key(Key::ArrowLeft);
    assert_eq!(current.get_untracked(), "analytics");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    screen.assert_layout_clean();
}

#[test]
fn sheet_attaches_to_each_side_and_closes_on_escape() {
    for (side, name) in [
        (Edge::Right, "Right"),
        (Edge::Left, "Left"),
        (Edge::Top, "Top"),
        (Edge::Bottom, "Bottom"),
    ] {
        let (screen, open) = kivact::test::render_with_handle(
            kivact::test::RenderOptions::with_theme(basic::theme(kivact::style::ThemeMode::Light))
                .reduced_motion(),
            move || {
                let open = signal(false);
                (
                    open,
                    page((
                        kivact::ui::button("Open").on_press(move || open.set(true)),
                        kivact::ui::dialog(name, open)
                            .mode(DialogKind::Drawer(side))
                            .description("Make changes here.")
                            .content(|| text("The body"))
                            .footer(|| kivact::ui::button("Save changes")),
                    )),
                )
            },
        );
        let opener = screen.get_by_role(Role::Button, "Open");
        opener.click();
        let dialog = screen.get_by_role(Role::Dialog, name);
        let rect = panel(&dialog);
        match side {
            Edge::Right => {
                assert_eq!(rect.x + rect.width, 800.0, "{name}");
                assert_eq!((rect.y, rect.height), (0.0, 600.0), "{name}");
                assert_eq!(rect.width, 384.0, "{name}: sm:max-w-sm");
            }
            Edge::Left => {
                assert_eq!((rect.x, rect.y, rect.height), (0.0, 0.0, 600.0), "{name}");
                assert_eq!(rect.width, 384.0, "{name}");
            }
            Edge::Top => {
                assert_eq!((rect.x, rect.y, rect.width), (0.0, 0.0, 800.0), "{name}");
                assert!(rect.height < 600.0);
            }
            Edge::Bottom => {
                assert_eq!((rect.x, rect.width), (0.0, 800.0), "{name}");
                assert_eq!(rect.y + rect.height, 600.0, "{name}");
            }
        }
        screen.get_by_text("The body");
        screen.get_by_role(Role::Button, "Save changes");
        screen.get_by_role(Role::Button, "Close");
        assert!(screen.errors().is_empty(), "{:?}", screen.errors());
        screen.assert_layout_clean();
        screen.assert_layout_with(&layout_check());
        screen.press_key(Key::Escape);
        assert!(!open.get_untracked(), "{name}: Escape closes it");
        assert!(opener.is_focused(), "{name}: focus returns to the opener");
    }
}

/// The panel of a sheet: its first child.
fn panel(dialog: &kivact::test::Locator<'_>) -> kivact::geometry::Rect {
    dialog
        .children()
        .first()
        .and_then(|panel| panel.rect())
        .expect("the dialog has a laid out panel")
}

/// The layout check with the vertical centering of button labels relaxed: the
/// bundled test font's line box sits about 2 px off the centre of a 12 px
/// label, which the real catalog fonts do not.
fn layout_check() -> LayoutCheck {
    LayoutCheck {
        center_tolerance: Some(3.0),
        ..LayoutCheck::default()
    }
}
