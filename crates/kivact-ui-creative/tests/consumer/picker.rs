//! Role: The search picker: filtering, keys, pick and cancel, the preview, focus.

use kivact::prelude::*;
use kivact::test::Screen;
use kivact::view::A11yAction;
use kivact_ui_creative as ui;

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
            .on_activate(move |key: &'static str| log.push(format!("pick {key}")))
            .on_preview(move |key: Option<&'static str>| log.push(format!("preview {key:?}")))
            .on_cancel(move || log.push("cancel".to_owned()))
    }
}

fn mount_attached() -> (Screen, Log) {
    kivact::test::render_with_handle(crate::support::options(), || {
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

fn row<'a>(screen: &'a Screen, name: &str) -> kivact::test::Locator<'a> {
    screen.get_by_role(Role::Option, name)
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
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
