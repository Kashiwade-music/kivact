//! Role: Capture the Soft catalog pages in both appearances for visual review.
use kivact::style::ThemeMode;
use kivact::test::{RenderOptions, Screen, TestFonts, render_with};
use kivact::view::{MouseButton, Role};
use kivact_ui_soft_catalog::{catalog_at, fonts, pages};
use std::path::Path;
use std::sync::Arc;

#[test]
#[ignore = "captures screenshots for manual visual review"]
fn capture_catalog() {
    let output = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/soft-lookbook");
    std::fs::create_dir_all(&output).expect("lookbook output directory");
    let filter = std::env::var("KIVACT_SOFT_LOOKBOOK_PAGES").ok();
    for page in pages().iter().filter(|page| {
        filter
            .as_deref()
            .is_none_or(|filter| filter.split(',').any(|slug| slug.trim() == page.slug))
    }) {
        for (mode, name) in [(ThemeMode::Light, "light"), (ThemeMode::Dark, "dark")] {
            let screen = render_with(
                RenderOptions {
                    width: 1120.0,
                    height: 800.0,
                    fonts: TestFonts::Only(fonts().iter().copied().map(Arc::from).collect()),
                    ..RenderOptions::from_env()
                },
                move || catalog_at(mode, page.slug),
            );
            screen
                .screenshot()
                .save_png(output.join(format!("{}-{name}.png", page.slug)))
                .expect("catalog screenshot");
            if page.slug == "button" {
                capture_button_states(&screen, &output, name);
            }
        }
    }
}

fn capture_button_states(screen: &Screen, output: &Path, mode: &str) {
    let primary = screen.get_by_role(Role::Button, "Primary");
    let save = |state: &str| {
        screen
            .screenshot()
            .save_png(output.join(format!("button-{mode}-{state}.png")))
            .expect("button state screenshot");
    };
    primary.hover();
    save("hover");
    screen.mouse_down(MouseButton::Primary);
    save("pressed");
    screen.mouse_up(MouseButton::Primary);
    screen.mouse_leave();
    primary.focus();
    screen.press_tab(false);
    screen.press_tab(true);
    save("focus");
}
