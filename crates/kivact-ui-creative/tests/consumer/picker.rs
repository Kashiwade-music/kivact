//! Role: The search picker: filtering, keys, pick and cancel, the preview, focus.

use super::*;
use kivact::view::node_ref;

type Items = Vec<ui::PickerItem<&'static str>>;

fn items() -> Items {
    vec![
        ui::PickerItem::new("solid", "Solid")
            .section("Generate")
            .description("Flat colour")
            .shortcut("Shift S"),
        ui::PickerItem::new("noise", "Noise")
            .section("Generate")
            .description("Fractal pattern")
            .icon("click"),
        ui::PickerItem::new("locked", "Locked")
            .section("Generate")
            .enabled(false),
        ui::PickerItem::new("blur", "Blur").section("Filter"),
        ui::PickerItem::new("glow", "Glow").section("Filter"),
    ]
}

/// What the application saw, in order.
#[derive(Clone, Copy)]
struct Log {
    events: Signal<Vec<String>>,
}

impl Log {
    fn new() -> Self {
        Self {
            events: signal(Vec::new()),
        }
    }

    fn push(&self, event: String) {
        self.events.update(|events| events.push(event));
    }

    fn take(&self) -> Vec<String> {
        let all = self.events.get_untracked();
        self.events.set(Vec::new());
        all
    }

    fn picker(&self, picker: ui::SearchPicker<&'static str>) -> ui::SearchPicker<&'static str> {
        let log = *self;
        picker
            .placeholder("Find")
            .on_pick(move |key: &'static str| log.push(format!("pick {key}")))
            .on_preview(move |key: Option<&'static str>| log.push(format!("preview {key:?}")))
            .on_cancel(move || log.push("cancel".to_owned()))
    }
}

fn mount_attached() -> (Screen, Log) {
    kivact::test::render_with_handle(crate::options(), || {
        let log = Log::new();
        let picker = log.picker(ui::search_picker(items()));
        (
            log,
            column((
                picker.attach(
                    kivact::ui::button("Add").variant(kivact::ui::ButtonVariant::Secondary),
                ),
                kivact::ui::button("Elsewhere").variant(kivact::ui::ButtonVariant::Secondary),
            )),
        )
    })
}

fn open(screen: &Screen) {
    screen.get_by_role(Role::Button, "Add").click();
    assert!(
        screen.get_by_role(Role::TextBox, "Find").is_focused(),
        "the focus is in the field"
    );
}

fn closed(screen: &Screen) -> bool {
    screen.query_by_role(Role::Dialog, "Find").is_none()
}

fn row<'a>(screen: &'a Screen, name: &str) -> kivact::test::Element<'a> {
    screen.get_by_role(Role::Option, name)
}

/// The labels of the rows the list shows, in the order of `items()`.
fn shown(screen: &Screen) -> Vec<&'static str> {
    ["Solid", "Noise", "Locked", "Blur", "Glow"]
        .into_iter()
        .filter(|name| screen.query_by_role(Role::Option, name).is_some())
        .collect()
}

#[test]
fn the_popup_is_a_dialog_with_a_field_that_controls_the_list() {
    let (screen, _) = mount_attached();
    let add = screen.get_by_role(Role::Button, "Add");
    assert_eq!(add.is_expanded(), Some(false));
    assert!(closed(&screen));
    open(&screen);
    assert_eq!(add.is_expanded(), Some(true));
    let dialog = screen.get_by_role(Role::Dialog, "Find");
    assert_eq!(add.controls().len(), 1, "the button controls the popup");
    assert!(dialog.exists());
    let field = screen.get_by_role(Role::TextBox, "Find");
    let list = screen.get_by_role(Role::ListBox, "Find");
    assert_eq!(field.controls().len(), 1);
    assert!(
        field.controls()[0].exists() && field.controls()[0].role() == Some(Role::ListBox),
        "the field controls the list"
    );
    assert!(field.is_focused());
    assert_eq!(shown(&screen), ["Solid", "Noise", "Locked", "Blur", "Glow"]);
    assert!(list.exists());
    assert!(
        screen
            .query_by_role(Role::Option, "Solid")
            .is_some_and(|r| r.is_selected()),
        "the first enabled row is active when it opens"
    );
    clean(&screen);
}

#[test]
fn the_default_placeholder_is_the_picker_message() {
    let (screen, _) = kivact::test::render_with_handle(crate::options(), || {
        (
            (),
            ui::search_picker(vec![ui::PickerItem::new(1, "One")])
                .attach(kivact::ui::button("Open").variant(kivact::ui::ButtonVariant::Secondary)),
        )
    });
    screen.get_by_role(Role::Button, "Open").click();
    screen.get_by_role(Role::Dialog, "Search");
    screen.get_by_role(Role::TextBox, "Search");
    clean(&screen);

    let screen = render_with(
        RenderOptions {
            messages: kivact::i18n::Messages::new()
                .with("creative.picker.placeholder", "検索")
                .with("creative.picker.empty", "該当なし"),
            theme: ui::theme(kivact::style::ThemeMode::Dark),
            ..RenderOptions::from_env()
        },
        || {
            ui::search_picker(vec![ui::PickerItem::new(1, "One")])
                .attach(kivact::ui::button("Open").variant(kivact::ui::ButtonVariant::Secondary))
        },
    );
    screen.get_by_role(Role::Button, "Open").click();
    screen.get_by_role(Role::TextBox, "検索").type_text("zzz");
    screen.get_by_text("該当なし");
    clean(&screen);
}

#[test]
fn the_search_reads_the_label_the_description_and_the_section() {
    let (screen, _) = mount_attached();
    open(&screen);

    screen.type_text("  nOI ");
    assert_eq!(shown(&screen), ["Noise"], "label, any case, trimmed");

    let field = screen.get_by_role(Role::TextBox, "Find");
    field.focus();
    screen.press_key_with(
        Key::Char('a'),
        Modifiers {
            ctrl: true,
            ..Modifiers::default()
        },
    );
    screen.type_text("flat");
    assert_eq!(shown(&screen), ["Solid"], "description");

    screen.press_key_with(
        Key::Char('a'),
        Modifiers {
            ctrl: true,
            ..Modifiers::default()
        },
    );
    screen.type_text("FILTER");
    assert_eq!(shown(&screen), ["Blur", "Glow"], "section");

    screen.press_key_with(
        Key::Char('a'),
        Modifiers {
            ctrl: true,
            ..Modifiers::default()
        },
    );
    screen.type_text("zzz");
    assert!(shown(&screen).is_empty());
    screen.get_by_text("No matches");
    clean(&screen);
}

#[test]
fn a_section_has_a_heading_and_a_group_while_it_has_rows() {
    let (screen, _) = mount_attached();
    open(&screen);
    screen.get_by_role(Role::Group, "Generate");
    screen.get_by_role(Role::Group, "Filter");
    screen.get_by_text("Generate");
    screen.type_text("glow");
    assert!(screen.query_by_role(Role::Group, "Generate").is_none());
    screen.get_by_role(Role::Group, "Filter");
    assert_eq!(shown(&screen), ["Glow"]);
    clean(&screen);
}

#[test]
fn arrows_move_over_enabled_rows_and_wrap_at_the_ends() {
    let (screen, _) = mount_attached();
    open(&screen);
    let active = |name: &str| row(&screen, name).is_selected();
    screen.press_key(Key::ArrowUp);
    assert!(active("Glow"), "Up from none: the last enabled");
    screen.press_key(Key::ArrowUp);
    assert!(active("Blur"));
    screen.press_key(Key::ArrowUp);
    assert!(active("Noise"), "the disabled row is skipped");
    screen.press_key(Key::ArrowUp);
    assert!(active("Solid"));
    screen.press_key(Key::ArrowUp);
    assert!(active("Glow"), "Up wraps to the end");
    screen.press_key(Key::ArrowDown);
    screen.press_key(Key::ArrowDown);
    screen.press_key(Key::ArrowDown);
    assert!(active("Blur"), "Down skips the disabled row");
    screen.press_key(Key::ArrowDown);
    screen.press_key(Key::ArrowDown);
    assert!(active("Solid"), "Down wraps to the start");
    assert!(!active("Blur"));
    assert!(screen.get_by_role(Role::TextBox, "Find").is_focused());
    clean(&screen);
}

#[test]
fn typing_and_clearing_the_query_activate_the_first_enabled_match() {
    let (screen, log) = mount_attached();
    open(&screen);
    screen.type_text("o");
    assert_eq!(shown(&screen), ["Solid", "Noise", "Locked", "Glow"]);
    assert!(row(&screen, "Solid").is_selected());
    screen.press_key(Key::Backspace);
    assert!(
        row(&screen, "Solid").is_selected(),
        "an empty query activates the first enabled row"
    );
    assert_eq!(log.take(), ["preview Some(\"solid\")"]);
    screen.type_text("lock");
    assert_eq!(shown(&screen), ["Locked"]);
    assert!(
        !row(&screen, "Locked").is_selected(),
        "a disabled row is never active"
    );
    clean(&screen);
}

#[test]
fn enter_and_a_click_pick_and_close_and_a_disabled_row_cannot_be_picked() {
    let (screen, log) = mount_attached();
    open(&screen);
    screen.press_key(Key::ArrowDown);
    screen.press_key(Key::Enter);
    assert!(closed(&screen));
    assert!(
        log.take().contains(&"pick noise".to_owned()),
        "Enter picks the active row"
    );

    open(&screen);
    row(&screen, "Blur").click();
    assert!(closed(&screen));
    assert!(
        log.take().contains(&"pick blur".to_owned()),
        "a click picks"
    );

    open(&screen);
    assert!(row(&screen, "Locked").is_disabled());
    row(&screen, "Locked").force_click();
    assert!(!closed(&screen), "a disabled row does not close the picker");
    row(&screen, "Solid").a11y(A11yAction::Activate);
    assert!(closed(&screen), "Activate picks");
    assert!(log.take().contains(&"pick solid".to_owned()));
    clean(&screen);
}

#[test]
fn escape_cancels_and_gives_the_focus_back_to_the_button() {
    let (screen, log) = mount_attached();
    open(&screen);
    screen.press_key(Key::ArrowDown);
    log.take();
    screen.press_key(Key::Escape);
    assert!(closed(&screen));
    assert_eq!(log.take(), ["cancel", "preview None"]);
    assert!(screen.get_by_role(Role::Button, "Add").is_focused());
    assert_eq!(
        screen.get_by_role(Role::Button, "Add").is_expanded(),
        Some(false)
    );

    // A second press on the button cancels too.
    open(&screen);
    log.take();
    screen.get_by_role(Role::Button, "Add").click();
    assert!(closed(&screen));
    assert_eq!(log.take(), ["cancel", "preview None"]);
    clean(&screen);
}

#[test]
fn an_outside_press_and_tab_cancel_and_leave_the_focus_where_it_went() {
    let (screen, log) = mount_attached();
    open(&screen);
    screen.press_key(Key::ArrowDown);
    log.take();
    screen.get_by_role(Role::Button, "Elsewhere").click();
    assert!(closed(&screen));
    assert_eq!(log.take(), ["cancel", "preview None"]);
    assert!(screen.get_by_role(Role::Button, "Elsewhere").is_focused());

    open(&screen);
    log.take();
    screen.press_tab(false);
    assert!(closed(&screen));
    assert_eq!(log.take(), ["cancel", "preview None"]);
    assert!(
        screen.query_by_role(Role::TextBox, "Find").is_none(),
        "the field is gone"
    );

    open(&screen);
    log.take();
    screen.press_tab(true);
    assert!(closed(&screen), "Shift+Tab back to the button cancels");
    assert_eq!(log.take(), ["cancel", "preview None"]);
    clean(&screen);
}

#[test]
fn the_preview_follows_the_active_row_and_ends_after_a_pick_or_a_cancel() {
    let (screen, log) = mount_attached();
    open(&screen);
    assert_eq!(log.take(), ["preview Some(\"solid\")"]);
    screen.press_key(Key::ArrowDown);
    screen.press_key(Key::ArrowDown);
    screen.press_key(Key::ArrowDown);
    screen.press_key(Key::ArrowDown);
    screen.press_key(Key::ArrowUp);
    assert_eq!(
        log.take(),
        [
            "preview Some(\"noise\")",
            "preview Some(\"blur\")",
            "preview Some(\"glow\")",
            "preview Some(\"solid\")",
            "preview Some(\"glow\")"
        ]
    );
    // Filtering the active row away clears the preview.
    screen.type_text("glo");
    assert_eq!(
        log.take().last().map(String::as_str),
        Some("preview Some(\"glow\")")
    );
    for _ in 0..3 {
        screen.press_key(Key::Backspace);
    }
    assert_eq!(
        log.take().last().map(String::as_str),
        Some("preview Some(\"solid\")")
    );
    log.take();
    screen.press_key(Key::Enter);
    assert_eq!(
        log.take(),
        ["pick solid", "preview None"],
        "on_pick, then one None"
    );

    open(&screen);
    screen.press_key(Key::ArrowDown);
    log.take();
    screen.press_key(Key::Escape);
    assert_eq!(log.take(), ["cancel", "preview None"]);

    // The pointer moves the active row, and so the preview.
    open(&screen);
    log.take();
    row(&screen, "Glow").hover();
    assert_eq!(log.take(), ["preview Some(\"glow\")"]);
    assert!(row(&screen, "Glow").is_selected());
    screen.press_key(Key::Escape);
    clean(&screen);
}

#[test]
fn the_field_names_the_active_row_as_its_active_descendant() {
    let (screen, _) = mount_attached();
    open(&screen);
    let field = screen.get_by_role(Role::TextBox, "Find");
    assert!(field.has_active_descendant(&row(&screen, "Solid")));
    assert!(field.has_active_descendant(&row(&screen, "Solid")));
    assert!(row(&screen, "Solid").is_selected());
    screen.press_key(Key::ArrowDown);
    assert!(field.has_active_descendant(&row(&screen, "Noise")));
    assert!(!field.has_active_descendant(&row(&screen, "Solid")));
    assert!(!row(&screen, "Solid").is_selected());
    assert_eq!(row(&screen, "Noise").description(), "Fractal pattern");
    assert!(field.is_focused(), "the rows take no focus");
    clean(&screen);
}

#[test]
fn a_pick_gives_the_focus_back_to_the_button() {
    let (screen, _) = mount_attached();
    open(&screen);
    screen.press_key(Key::ArrowDown);
    screen.press_key(Key::Enter);
    assert!(closed(&screen));
    assert!(screen.get_by_role(Role::Button, "Add").is_focused());
    open(&screen);
    row(&screen, "Glow").click();
    assert!(screen.get_by_role(Role::Button, "Add").is_focused());
    clean(&screen);
}

fn mount_at() -> (Screen, (Log, Signal<Option<Point>>)) {
    kivact::test::render_with_handle(crate::options(), || {
        let log = Log::new();
        let at = signal(None::<Point>);
        let viewer = node_ref();
        let picker = log.picker(ui::search_picker(items()));
        (
            (log, at),
            column((
                node()
                    .node_ref(viewer)
                    .role(Role::Group)
                    .name("Viewer")
                    .focusable(true)
                    .style(style().size(300, 200)),
                kivact::ui::button("Elsewhere").variant(kivact::ui::ButtonVariant::Secondary),
                picker.at(viewer, at),
            )),
        )
    })
}

#[test]
fn at_opens_at_the_point_and_a_pick_or_escape_restores_the_anchor() {
    let (screen, (log, at)) = mount_at();
    assert!(closed(&screen));
    at.set(Some(Point::new(120.0, 90.0)));
    screen.run_until_idle();
    let dialog = screen.get_by_role(Role::Dialog, "Find");
    let rect = dialog.rect().expect("the popup is laid out");
    assert_eq!((rect.x, rect.y), (120.0, 90.0), "its corner is the point");
    screen.get_by_role(Role::TextBox, "Find").focus();
    screen.press_key(Key::Enter);
    assert!(closed(&screen));
    assert_eq!(at.get_untracked(), None, "the point is cleared");
    assert!(screen.get_by_role(Role::Group, "Viewer").is_focused());
    assert!(log.take().contains(&"pick solid".to_owned()));

    at.set(Some(Point::new(40.0, 60.0)));
    screen.run_until_idle();
    assert!(screen.get_by_role(Role::TextBox, "Find").is_focused());
    screen.press_key(Key::Escape);
    assert!(closed(&screen));
    assert_eq!(at.get_untracked(), None);
    assert!(screen.get_by_role(Role::Group, "Viewer").is_focused());
    assert!(log.take().contains(&"cancel".to_owned()));

    // An outside press cancels and keeps the focus where it went.
    at.set(Some(Point::new(40.0, 60.0)));
    screen.run_until_idle();
    screen.get_by_role(Role::Button, "Elsewhere").click();
    assert!(closed(&screen));
    assert_eq!(at.get_untracked(), None);
    assert!(screen.get_by_role(Role::Button, "Elsewhere").is_focused());
    clean(&screen);
}

#[test]
fn closing_by_clearing_the_point_ends_the_preview() {
    let (screen, (log, at)) = mount_at();
    at.set(Some(Point::new(50.0, 50.0)));
    screen.run_until_idle();
    screen.press_key(Key::ArrowDown);
    log.take();
    at.set(None);
    screen.run_until_idle();
    assert!(closed(&screen));
    assert_eq!(log.take(), ["cancel", "preview None"]);
    clean(&screen);
}

fn mount_columns() -> (Screen, (Log, Signal<Items>, Signal<String>)) {
    kivact::test::render_with_handle(crate::options(), || {
        let log = Log::new();
        let effects = signal(vec![
            ui::PickerItem::new("blur", "Blur"),
            ui::PickerItem::new("glow", "Glow"),
        ]);
        let title = signal("Effects".to_owned());
        let picker = log.picker(
            ui::search_picker(vec![
                ui::PickerItem::new("solid", "Solid"),
                ui::PickerItem::new("noise", "Noise"),
            ])
            .column(title, effects)
            .column(
                "Unavailable",
                vec![ui::PickerItem::new("locked", "Locked").enabled(false)],
            ),
        );
        (
            (log, effects, title),
            picker.attach(kivact::ui::button("Add")),
        )
    })
}

#[test]
fn multiple_columns_keep_all_named_lists_visible_and_cycle_without_canceling() {
    let (screen, (log, _, _)) = mount_columns();
    open(&screen);
    for name in ["Find", "Effects", "Unavailable"] {
        screen.get_by_role(Role::Heading, name);
        screen.get_by_role(Role::ListBox, name);
    }
    for label in ["Solid", "Noise", "Blur", "Glow", "Locked"] {
        assert!(row(&screen, label).is_visible());
    }
    let field = screen.get_by_role(Role::TextBox, "Find");
    assert!(field.has_active_descendant(&row(&screen, "Solid")));
    log.take();
    screen.press_tab(false);
    assert!(field.has_active_descendant(&row(&screen, "Blur")));
    screen.press_key(Key::ArrowDown);
    assert!(field.has_active_descendant(&row(&screen, "Glow")));
    screen.press_key(Key::ArrowRight);
    assert!(field.has_active_descendant(&row(&screen, "Solid")));
    screen.press_key(Key::ArrowLeft);
    assert!(
        field.has_active_descendant(&row(&screen, "Blur")),
        "entering a column starts at its first enabled row"
    );
    screen.press_tab(true);
    assert!(field.has_active_descendant(&row(&screen, "Solid")));
    assert!(field.is_focused());
    assert!(!closed(&screen));
    assert!(!log.take().iter().any(|event| event == "cancel"));
    clean(&screen);
}

#[test]
fn column_search_and_reactive_items_keep_an_enabled_active_candidate() {
    let (screen, (log, effects, title)) = mount_columns();
    open(&screen);
    let field = screen.get_by_role(Role::TextBox, "Find");
    screen.type_text("gl");
    assert!(screen.query_by_role(Role::Option, "Solid").is_none());
    assert!(field.has_active_descendant(&row(&screen, "Glow")));
    screen.press_key(Key::Backspace);
    screen.press_key(Key::Backspace);
    assert!(
        field.has_active_descendant(&row(&screen, "Blur")),
        "query changes keep the current column when it has matches"
    );
    effects.set(vec![ui::PickerItem::new("sharpen", "Sharpen")]);
    title.set("Filters".to_owned());
    screen.run_until_idle();
    screen.get_by_role(Role::Heading, "Filters");
    screen.get_by_role(Role::ListBox, "Filters");
    assert!(field.has_active_descendant(&row(&screen, "Sharpen")));
    effects.set(Vec::new());
    screen.run_until_idle();
    assert!(
        field.has_active_descendant(&row(&screen, "Solid")),
        "an empty column moves to the next enabled column"
    );
    screen.type_text("missing");
    assert!(screen.all_by_role(Role::Option, "").is_empty());
    screen.get_by_text("No matches");
    assert!(log.take().iter().any(|event| event == "preview None"));
    screen.press_tab(false);
    assert!(field.is_focused());
    assert!(!closed(&screen));
    clean(&screen);
}

#[test]
fn inactive_column_pointer_preview_and_pick_share_the_combobox_contract() {
    let (screen, (log, _, _)) = mount_columns();
    open(&screen);
    log.take();
    row(&screen, "Glow").hover();
    let field = screen.get_by_role(Role::TextBox, "Find");
    assert!(field.has_active_descendant(&row(&screen, "Glow")));
    assert_eq!(log.take(), ["preview Some(\"glow\")"]);
    assert!(field.is_focused());
    row(&screen, "Glow").click();
    assert!(closed(&screen));
    assert_eq!(log.take(), ["pick glow", "preview None"]);
    open(&screen);
    log.take();
    row(&screen, "Blur").a11y(A11yAction::Activate);
    assert!(closed(&screen));
    assert_eq!(log.take(), ["pick blur", "preview None"]);
    clean(&screen);
}
