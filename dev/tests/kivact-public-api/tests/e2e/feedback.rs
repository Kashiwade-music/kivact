//! Role: Exercise feedback and card contracts through public components.
use kivact::prelude::*;
use kivact::test::{MirrorMode, RenderOptions, render_with_handle};
use kivact::ui::{self, AlertVariant, ControlSize};
use kivact::view::{Key, Role};

fn options() -> RenderOptions {
    RenderOptions::default().mirror(MirrorMode::Off)
}

#[test]
fn empty_state_announces_once() {
    let (screen, title) = render_with_handle(options(), || {
        let title = signal("No results");
        (
            title,
            ui::empty_state(move || title.get())
                .description("Try another search")
                .announce(),
        )
    });
    let notice = screen.get_by_role(Role::Text, "No results. Try another search");
    title.set("Still empty");
    screen.run_until_idle();
    assert!(notice.exists());
    assert!(
        screen
            .query_by_role(Role::Text, "Still empty. Try another search")
            .is_none()
    );
    assert!(screen.get_by_role(Role::Heading, "Still empty").exists());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn alert_dismiss_restores_focus() {
    let (screen, dismissed) = render_with_handle(options(), || {
        let dismissed = signal(0);
        (
            dismissed,
            column((
                ui::button("Previous control"),
                ui::alert("Connection lost")
                    .variant(AlertVariant::Banner)
                    .dismiss_label("Dismiss connection alert")
                    .on_dismiss(move || dismissed.update(|count| *count += 1)),
                ui::button("Next control"),
            )),
        )
    });
    let previous = screen.get_by_role(Role::Button, "Previous control");
    previous.focus();
    screen
        .get_by_role(Role::Button, "Dismiss connection alert")
        .focus();
    screen.press_key(Key::Enter);
    assert_eq!(dismissed.get(), 1);
    assert!(previous.is_focused());
    assert!(
        screen
            .query_by_role(Role::Alert, "Connection lost")
            .is_none()
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn card_on_press_activates_with_the_keyboard() {
    let (screen, presses) = render_with_handle(options(), || {
        let presses = signal(0);
        (
            presses,
            ui::card(text("Open project"))
                .name("Project")
                .on_press(move || presses.update(|count| *count += 1)),
        )
    });
    let card = screen.get_by_role(Role::Button, "Project");
    card.press_key(Key::Enter);
    card.press_key(Key::Space);
    card.a11y(kivact::view::A11yAction::Activate);
    assert_eq!(presses.get(), 3);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn selectable_card_writes_the_binding_and_disabled_blocks_edits() {
    let (screen, (selected, disabled)) = render_with_handle(options(), || {
        let selected = signal(false);
        let disabled = signal(false);
        (
            (selected, disabled),
            ui::card(text("Option"))
                .name("Choice")
                .selected(selected)
                .disabled(disabled),
        )
    });
    let card = screen.get_by_role(Role::CheckBox, "Choice");
    card.press_key(Key::Space);
    assert!(selected.get());
    card.press_key(Key::Enter);
    assert!(!selected.get());
    disabled.set(true);
    screen.run_until_idle();
    card.a11y(kivact::view::A11yAction::Activate);
    assert!(!selected.get());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn input_group_scopes_control_sizes_and_preserves_adornments() {
    let (screen, ()) = render_with_handle(options(), || {
        (
            (),
            ui::input_group_with("Address", || {
                (
                    ui::input_addon("https://"),
                    ui::text_input("Host", signal(String::new())),
                )
            })
            .size(ControlSize::Large)
            .start(text("Start"))
            .end(text("End"))
            .block_start(text("Above"))
            .block_end(text("Below")),
        )
    });
    assert!(screen.get_by_role(Role::Group, "Address").exists());
    assert!(screen.get_by_role(Role::Text, "https://").exists());
    assert!(screen.get_by_role(Role::Text, "Above").exists());
    assert!(screen.get_by_role(Role::Text, "Below").exists());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn alert_details_follow_the_open_binding() {
    let (screen, open) = render_with_handle(options(), || {
        let open = signal(false);
        (
            open,
            ui::alert("Warning")
                .content(|| text("Details"))
                .collapsible(true)
                .open(open),
        )
    });
    assert!(screen.query_by_role(Role::Text, "Details").is_none());
    screen.get_by_role(Role::Button, "Expand details").click();
    assert!(open.get());
    assert!(screen.get_by_role(Role::Text, "Details").exists());
    open.set(false);
    screen.run_until_idle();
    assert!(screen.query_by_role(Role::Text, "Details").is_none());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn input_group_addons_focus_controls_without_stealing_nested_button_focus() {
    let (screen, presses) = render_with_handle(options(), || {
        let presses = signal(0usize);
        (
            presses,
            ui::input_group(
                "Search group",
                (
                    ui::input_addon("Prefix"),
                    ui::text_input("Search", signal(String::new())),
                ),
            )
            .start(text("Search icon").test_id("search-addon"))
            .end(ui::button("Copy").on_press(move || presses.update(|n| *n += 1)))
            .block_end(text("Hint").test_id("search-hint")),
        )
    });
    let input = screen.get_by_role(Role::TextBox, "Search");
    screen.get_by_test_id("search-addon").click();
    assert!(input.is_focused());
    screen.get_by_role(Role::Button, "Copy").click();
    assert_eq!(presses.get_untracked(), 1);
    assert!(screen.get_by_role(Role::Button, "Copy").is_focused());
    screen.get_by_test_id("search-hint").click();
    assert!(input.is_focused());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
