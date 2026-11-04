//! Role: Foundation tests: theme, standard recipes, icons and primitives.

use crate::support::{assert_clean, distance, pixel};
use kivact::prelude::*;
use kivact::style::Align;
use kivact::test::Screen;
use kivact::ui::ControlSize;
use kivact_ui_basic as basic;

/// A page painted with the `background` token of the theme in effect.
fn page(child: impl IntoView) -> Node {
    column(child).style(
        style()
            .background(basic::Color::Background)
            .padding(16)
            .gap(12)
            .grow(1)
            .align_items(Align::Start),
    )
}

#[test]
fn theme_switch_restyles_mounted_components_and_keeps_their_state() {
    let (screen, (dark, checked, name)) =
        kivact::test::render_with_handle(crate::support::options(), || {
            let dark = signal(false);
            let checked = signal(true);
            let name = signal(String::from("Ada"));
            (
                (dark, checked, name),
                kivact::style::provide_theme(
                    move || {
                        basic::theme(if dark.get() {
                            kivact::style::ThemeMode::Dark
                        } else {
                            kivact::style::ThemeMode::Light
                        })
                    },
                    move || {
                        page((
                            kivact::ui::button("Save").test_id("save"),
                            kivact::ui::checkbox("Sync", checked),
                            kivact::ui::text_input("Name", name),
                        ))
                    },
                ),
            )
        });
    assert!(distance(pixel(&screen, 4.0, 4.0), 0xffffff) <= 2);
    assert!(distance(pixel_in(&screen, "save", 4.0), 0x171717) <= 2);
    assert!(
        basic::theme(kivact::style::ThemeMode::Light)
            == basic::theme(kivact::style::ThemeMode::Light),
        "the kit's themes share one recipe table and icon set, so an equal theme compares equal"
    );
    dark.set(true);
    screen.run_until_idle();
    assert!(distance(pixel(&screen, 4.0, 4.0), 0x0a0a0a) <= 2);
    assert!(
        distance(pixel_in(&screen, "save", 4.0), 0xe5e5e5) <= 2,
        "the dark primary fills the mounted button"
    );
    assert!(checked.get_untracked());
    assert_eq!(name.get_untracked(), "Ada");
    assert_eq!(
        screen.get_by_role(Role::CheckBox, "Sync").checked(),
        Some(CheckedState::Checked)
    );
    assert_eq!(
        screen.get_by_role(Role::TextBox, "Name").value().as_deref(),
        Some("Ada")
    );
    assert_clean(&screen);
}

/// The color 3 px in and 13 px down inside a check box's 16 px box, clear of the check mark.
fn check_box_fill(screen: &kivact::test::Screen, name: &str) -> u32 {
    let rect = screen
        .get_by_role(Role::CheckBox, name)
        .rect()
        .expect("laid out");
    let top = ((rect.height - 16.0) / 2.0).round();
    pixel(screen, rect.x + 4.0, rect.y + top + 13.0)
}

#[test]
fn checkbox_and_switch_toggle_with_pointer_and_keyboard() {
    let (screen, (checked, on)) =
        kivact::test::render_with_handle(crate::support::options(), || {
            let checked = signal(false);
            let on = signal(false);
            (
                (checked, on),
                page((
                    kivact::ui::checkbox("Accept terms", checked),
                    kivact::ui::switch("Airplane mode", on).test_id("switch"),
                    kivact::ui::checkbox("Partial", signal(false)).indeterminate(true),
                    kivact::ui::checkbox("Locked", signal(false)).disabled(true),
                )),
            )
        });
    assert!(distance(check_box_fill(&screen, "Accept terms"), 0xffffff) <= 2);
    let check = screen.get_by_role(Role::CheckBox, "Accept terms");
    check.click();
    assert!(checked.get_untracked());
    assert!(distance(check_box_fill(&screen, "Accept terms"), 0x171717) <= 2);
    check.press_key(Key::Space);
    assert!(!checked.get_untracked());
    assert_eq!(
        screen.get_by_role(Role::CheckBox, "Partial").checked(),
        Some(CheckedState::Indeterminate)
    );
    assert!(screen.get_by_role(Role::CheckBox, "Locked").is_disabled());
    let switch = screen.get_by_role(Role::Switch, "Airplane mode");
    let track = |screen: &kivact::test::Screen| pixel_in(screen, "switch", 6.0);
    assert!(distance(track(&screen), 0x171717) > 40);
    switch.click();
    assert!(on.get_untracked());
    assert!(
        distance(track(&screen), 0x171717) <= 2,
        "{:06x}",
        track(&screen)
    );
    switch.press_key(Key::Space);
    assert!(!on.get_untracked());
    assert_clean(&screen);
}

#[test]
fn text_input_edits_its_binding_and_rings_on_focus() {
    let (screen, (name, notes)) =
        kivact::test::render_with_handle(crate::support::options(), || {
            let name = signal(String::new());
            let notes = signal(String::new());
            (
                (name, notes),
                page((
                    kivact::ui::text_input("Name", name)
                        .placeholder("Your name")
                        .test_id("name"),
                    kivact::ui::text_input("Code", signal(String::new()))
                        .size(ControlSize::Small)
                        .test_id("code"),
                    kivact::ui::text_area("Notes", notes).test_id("notes"),
                )),
            )
        });
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
    assert_clean(&screen);
}

/// The color `inset` logical pixels inside the left edge of `id`, at mid height.
fn pixel_in(screen: &Screen, id: &str, inset: f64) -> u32 {
    let rect = screen.get_by_test_id(id).rect().expect("laid out");
    pixel(screen, rect.x + inset, rect.y + rect.height / 2.0)
}
