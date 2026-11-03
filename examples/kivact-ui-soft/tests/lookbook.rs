//! Role: Capture the Soft catalog pages in both appearances for visual review.
use kivact::style::ThemeMode;
use kivact::test::{RenderOptions, Screen, TestFonts, render_with};
use kivact::view::{Key, MouseButton, Role};
use kivact_ui_soft_catalog::{catalog_at, fonts, pages};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

#[test]
#[ignore = "captures screenshots for manual visual review"]
fn capture_catalog() {
    let output = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/soft-lookbook");
    std::fs::create_dir_all(&output).expect("lookbook output directory");
    let filter = std::env::var("KIVACT_SOFT_LOOKBOOK_PAGES").ok();
    let mut diagnostics = Vec::new();
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
            capture_overlay_states(&screen, &output, page.slug, name);
            if page.slug == "spinner" {
                screen.set_reduced_motion(true);
                screen.advance(Duration::from_millis(750));
                screen
                    .screenshot()
                    .save_png(output.join(format!("spinner-{name}-reduced.png")))
                    .expect("reduced-motion spinner screenshot");
            }
            let errors = screen.errors();
            if !errors.is_empty() {
                diagnostics.push(format!("{} ({name}): {errors:?}", page.slug));
            }
        }
    }
    assert!(diagnostics.is_empty(), "{}", diagnostics.join("\n"));
}

fn capture_overlay_states(screen: &Screen, output: &Path, slug: &str, mode: &str) {
    let save = |state: &str| {
        screen
            .screenshot()
            .save_png(output.join(format!("{slug}-{mode}-{state}.png")))
            .expect("overlay state screenshot");
    };
    let trigger = match slug {
        "alert-dialog" => "Delete file",
        "bottom-sheet" => "Open sheet",
        "bottom-sheet-switcher" => "Start review",
        "command-palette" => "Open commands",
        "dialog" => "Open dialog",
        "drawer" => "Open drawer",
        "dropdown-menu" => "Actions",
        "layer" | "popover" => "Open popover",
        "lightbox" => "Open image",
        "toast" => "Show notification",
        "hover-card" => "Avery Morgan",
        "tooltip" => "Save",
        _ => return,
    };
    let button = screen.get_by_role(Role::Button, trigger);
    match slug {
        "hover-card" => button.hover(),
        "tooltip" => button.focus(),
        _ => button.click(),
    }
    if matches!(slug, "bottom-sheet" | "dialog" | "drawer") {
        screen.advance(Duration::from_millis(34));
        save("entering");
    }
    screen.advance(Duration::from_millis(850));
    save("open");
    if matches!(slug, "bottom-sheet" | "dialog" | "drawer") {
        screen.press_key(Key::Escape);
        screen.advance(Duration::from_millis(34));
        save("exiting");
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
