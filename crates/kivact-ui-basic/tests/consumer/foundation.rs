//! Role: Theme tests: a theme switch restyles the mounted components and keeps their state.

use crate::support::{distance, pixel, pixel_in};
use kivact::prelude::*;
use kivact::view::Node;
use kivact_ui_basic as basic;

/// A page painted with the `background` token of the theme in effect.
fn page(child: impl IntoView) -> Node {
    column(child).style(
        style()
            .background(basic::BACKGROUND)
            .padding(16)
            .gap(12)
            .flex_grow(1)
            .align_items(Align::Start),
    )
}

#[test]
fn theme_switch_restyles_mounted_components_and_keeps_their_state() {
    let (screen, (dark, checked, name)) = kivact::test::render_with_handle(
        kivact::test::RenderOptions::with_theme(basic::theme(kivact::style::ThemeMode::Light))
            .reduced_motion(),
        || {
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
                            kivact::ui::text_input(name).label("Name"),
                        ))
                    },
                ),
            )
        },
    );
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
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    screen.assert_layout_clean();
}
