//! Role: Verify overlay interaction through the Soft public API.
use kivact::interaction::Delays;
use kivact::interaction::MenuEntry;
use kivact::interaction::use_hover_surface;
use kivact::state::signal;
use kivact::style::Position;
use kivact::style::ThemeMode;
use kivact::style::style;
use kivact::test::RenderOptions;
use kivact::test::render_with;
use kivact::ui::button;
use kivact::ui::dialog;
use kivact::ui::menu;
use kivact::view::Key;
use kivact::view::NodeBuilder;
use kivact::view::Role;
use kivact::view::column;
use kivact::view::node_ref;
use kivact::view::row;
use kivact_ui_soft::{
    DialogPurpose, LightboxMedia, SheetHeight, ShowOn, bottom_sheet, lightbox, media_overlay,
};
use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

fn options() -> RenderOptions {
    RenderOptions::from_env()
        .window_size(800.0, 600.0)
        .theme(kivact_ui_soft::theme(ThemeMode::Light))
}

#[test]
fn modal_centers_on_the_window_and_sheet_stays_at_the_bottom() {
    let screen = render_with(options(), || {
        let modal = signal(false);
        let sheet = signal(false);
        column((
            dialog("Window modal", modal).attach(button("Open modal")),
            button("Open sheet").on_press(move || sheet.set(true)),
            bottom_sheet("Window sheet", sheet)
                .height(SheetHeight::Px(160.0))
                .content(|| button("Inside sheet")),
        ))
    });
    screen.get_by_role(Role::Button, "Open modal").click();
    screen.advance(Duration::from_millis(450));
    let modal = screen
        .get_by_role(Role::Dialog, "Window modal")
        .rect()
        .expect("laid out modal");
    assert!((modal.x + modal.width / 2.0 - 400.0).abs() <= 1.0);
    assert!((modal.y + modal.height / 2.0 - 300.0).abs() <= 1.0);
    screen.press_key(Key::Escape);
    screen.advance(Duration::from_millis(450));
    screen.get_by_role(Role::Button, "Open sheet").click();
    screen.advance(Duration::from_millis(450));
    let inside = screen
        .get_by_role(Role::Button, "Inside sheet")
        .rect()
        .expect("laid out sheet content");
    assert!(inside.y > 400.0, "{inside:?}");
    assert!(inside.width > 400.0, "{inside:?}");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn lightbox_exposes_zoom_and_replaces_a_failed_image_with_status() {
    static PHOTO: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" width="32" height="16"><rect width="32" height="16"/></svg>"#;
    let screen = render_with(options(), || {
        let open = signal(false);
        column((
            button("Open gallery").on_press(move || open.set(true)),
            lightbox(
                open,
                vec![
                    LightboxMedia::image(PHOTO, "Example photo"),
                    LightboxMedia::image("missing-soft-gallery-image.png", "Missing photo"),
                ],
            )
            .zoom(true),
        ))
    });
    screen.get_by_role(Role::Button, "Open gallery").click();
    screen.get_by_role(Role::Image, "Example photo");
    let zoom = screen.get_by_role(Role::Button, "Zoom");
    zoom.press_key(Key::Enter);
    assert!(zoom.is_selected());
    zoom.press_key(Key::Enter);
    assert!(!zoom.is_selected());
    screen.get_by_role(Role::Button, "Next").click();
    screen.get_by_text("Image unavailable");
    assert!(screen.query_by_role(Role::Image, "Missing photo").is_none());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn soft_submenus_delay_opening_and_remain_open_briefly_over_a_sibling() {
    let screen = render_with(options(), || {
        menu(
            "Actions",
            vec![
                MenuEntry::submenu("export", "Export", vec![MenuEntry::item("pdf", "PDF")]),
                MenuEntry::item("rename", "Rename"),
            ],
        )
        .on_activate(|_| {})
    });
    screen
        .get_by_role(Role::Button, "Actions")
        .press_key(Key::ArrowDown);
    let export = screen.get_by_role(Role::MenuItem, "Export");
    export.hover();
    screen.advance(Duration::from_millis(100));
    assert_eq!(export.is_expanded(), Some(false));
    screen.advance(Duration::from_millis(100));
    assert_eq!(export.is_expanded(), Some(true));
    screen.get_by_role(Role::MenuItem, "Rename").hover();
    screen.advance(Duration::from_millis(100));
    assert_eq!(export.is_expanded(), Some(true));
    screen.advance(Duration::from_millis(150));
    assert_eq!(export.is_expanded(), Some(false));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_diagonal_path_over_a_sibling_keeps_the_open_submenu() {
    let screen = render_with(options(), || {
        menu(
            "Actions",
            vec![
                MenuEntry::submenu(
                    "export",
                    "Export",
                    vec![
                        MenuEntry::item("pdf", "PDF"),
                        MenuEntry::item("png", "PNG"),
                        MenuEntry::item("svg", "SVG"),
                        MenuEntry::item("csv", "CSV"),
                        MenuEntry::item("txt", "Text file"),
                    ],
                ),
                MenuEntry::item("rename", "Rename"),
            ],
        )
        .on_activate(|_| {})
    });
    screen
        .get_by_role(Role::Button, "Actions")
        .press_key(Key::ArrowDown);
    let export = screen.get_by_role(Role::MenuItem, "Export");
    let start = export.rect().unwrap();
    let rename = screen.get_by_role(Role::MenuItem, "Rename");
    let sibling = rename.rect().unwrap();
    screen.mouse_move(start.x + start.width * 0.35, start.y + start.height * 0.72);
    screen.advance(Duration::from_millis(200));
    assert_eq!(export.is_expanded(), Some(true));

    screen.mouse_move(
        sibling.x + sibling.width * 0.68,
        sibling.y + sibling.height * 0.15,
    );
    screen.advance(Duration::from_millis(120));
    screen.mouse_move(
        sibling.x + sibling.width * 0.83,
        sibling.y + sibling.height * 0.28,
    );
    screen.advance(Duration::from_millis(120));
    assert_eq!(export.is_expanded(), Some(true));
    assert!(!rename.is_focused());

    screen.get_by_role(Role::MenuItem, "PNG").hover();
    assert_eq!(export.is_expanded(), Some(true));
    rename.hover();
    screen.advance(Duration::from_millis(250));
    assert_eq!(export.is_expanded(), Some(false));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn hover_surface_renews_its_close_delay_across_a_diagonal_gap() {
    let screen = render_with(options(), || {
        let anchor = node_ref();
        let surface = node_ref();
        let delayed = use_hover_surface(
            anchor,
            surface,
            true,
            Delays {
                open: Duration::ZERO,
                close: Duration::from_millis(200),
            },
        );
        let shown = delayed.is_open();
        column((
            button("Trigger").node_ref(anchor).style(
                style()
                    .position(Position::Absolute)
                    .left(110)
                    .top(20)
                    .size(180, 48),
            ),
            button("Flyout")
                .node_ref(surface)
                .hidden(move || !shown.get())
                .style(
                    style()
                        .position(Position::Absolute)
                        .left(140)
                        .top(160)
                        .size(240, 70),
                ),
        ))
        .style(style().position(Position::Relative).size(450, 320))
    });
    let trigger = screen.get_by_role(Role::Button, "Trigger");
    trigger.hover();
    let flyout = screen.get_by_role(Role::Button, "Flyout");
    let origin = trigger.rect().unwrap();
    let panel = flyout.rect().unwrap();
    let origin_x = origin.x + origin.width * 0.5;
    let origin_y = origin.y + origin.height * 0.5;
    let target_x = panel.x + panel.width * 0.5;
    for fraction in [0.35, 0.65] {
        screen.mouse_move(
            origin_x + (target_x - origin_x) * fraction,
            origin_y + (panel.y - origin_y) * fraction,
        );
        screen.advance(Duration::from_millis(120));
        screen.get_by_role(Role::Button, "Flyout");
    }
    screen.advance(Duration::from_millis(150));
    assert!(screen.query_by_role(Role::Button, "Flyout").is_none());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn closed_media_overlay_passes_pointer_and_keyboard_to_base() {
    let presses = Rc::new(Cell::new(0));
    let base_presses = presses.clone();
    let screen = render_with(options(), move || {
        media_overlay(|| button("Overlay action"))
            .open(false)
            .attach(
                button("Base")
                    .style(style().size(120, 60))
                    .on_press(move || {
                        base_presses.set(base_presses.get() + 1);
                    }),
            )
    });
    assert!(
        screen
            .query_by_role(Role::Button, "Overlay action")
            .is_none()
    );
    screen.press_key(Key::Tab);
    assert_eq!(screen.focused().unwrap().name(), "Base");
    screen.get_by_role(Role::Button, "Base").click();
    assert_eq!(presses.get(), 1);
}

#[test]
fn focus_mode_reveals_the_overlay_even_with_a_noninteractive_base() {
    let screen = render_with(options(), || {
        media_overlay(|| button("Overlay action"))
            .show_on(ShowOn::Focus)
            .attach(kivact::view::node().style(style().size(120, 60)))
    });
    assert!(
        screen
            .query_by_role(Role::Button, "Overlay action")
            .is_none()
    );
    screen.press_key(Key::Tab);
    assert!(
        screen
            .query_by_role(Role::Button, "Overlay action")
            .is_some()
    );
    screen.press_key(Key::Tab);
    assert_eq!(screen.focused().unwrap().name(), "Overlay action");
}

#[test]
fn sheet_escape_honors_purpose_and_programmatic_close_restores_final_focus() {
    for purpose in [
        DialogPurpose::Info,
        DialogPurpose::Form,
        DialogPurpose::Required,
    ] {
        let screen = render_with(options(), move || {
            let open = signal(false);
            let final_focus = node_ref();
            column((
                row((
                    button("Open sheet").on_press(move || open.set(true)),
                    button("Final focus").node_ref(final_focus),
                )),
                bottom_sheet("Sheet", open)
                    .purpose(purpose)
                    .final_focus(final_focus)
                    .content(move || {
                        button("Programmatic close").on_press(move || open.set(false))
                    }),
            ))
        });
        screen.get_by_role(Role::Button, "Open sheet").click();
        let role = if purpose == DialogPurpose::Required {
            Role::AlertDialog
        } else {
            Role::Dialog
        };
        assert!(screen.query_by_role(role, "Sheet").is_some());
        assert_eq!(screen.focused().unwrap().name(), "Programmatic close");
        screen.press_key(Key::Escape);
        if purpose == DialogPurpose::Required {
            assert!(screen.query_by_role(role, "Sheet").is_some());
            screen.press_key(Key::Enter);
        }
        assert!(screen.query_by_role(role, "Sheet").is_none());
        assert_eq!(screen.focused().unwrap().name(), "Final focus");
        assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    }
}
