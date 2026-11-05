//! Role: Capture the Soft catalog pages in both appearances for visual review.
#![deny(clippy::too_many_lines, clippy::disallowed_types)]
use kivact::style::ThemeMode;
use kivact::test::{RenderOptions, Screen, TestFonts, render_with};
use kivact::view::{Key, MouseButton, Role};
use kivact_ui_soft_catalog::{catalog_at, fonts, pages};
use std::fmt::Write;
use std::path::Path;
use std::time::Duration;

#[test]
#[ignore = "captures screenshots for manual visual review"]
fn capture_catalog() {
    let output = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/soft-lookbook");
    std::fs::create_dir_all(&output).expect("lookbook output directory");
    let filter = std::env::var("KIVACT_SOFT_LOOKBOOK_PAGES").ok();
    let mut diagnostics = Vec::new();
    let mut layout = String::new();
    for page in pages().iter().filter(|page| {
        filter
            .as_deref()
            .is_none_or(|filter| filter.split(',').any(|slug| slug.trim() == page.slug))
    }) {
        for (mode, name) in [(ThemeMode::Light, "light"), (ThemeMode::Dark, "dark")] {
            let screen = render_with(
                RenderOptions::from_env()
                    .window_size(1120.0, 800.0)
                    .fonts(TestFonts::Only(
                        fonts().iter().map(|&bytes| bytes.into()).collect(),
                    )),
                move || catalog_at(mode, page.slug),
            );
            screen
                .screenshot()
                .save_png(output.join(format!("{}-{name}.png", page.slug)))
                .expect("catalog screenshot");
            let findings = screen.get_by_test_id("catalog-page-content").check_layout();
            writeln!(
                layout,
                "{} ({name}) — {} findings",
                page.slug,
                findings.len()
            )
            .expect("write layout heading");
            for finding in findings {
                writeln!(layout, "  {finding}").expect("write layout finding");
            }
            if page.slug == "button" {
                capture_button_states(&screen, &output, name);
            }
            capture_overlay_states(&screen, &output, page.slug, name);
            capture_disclosure_states(&screen, &output, page.slug, name);
            capture_content_states(&screen, &output, page.slug, name);
            capture_table_states(&screen, &output, page.slug, name);
            capture_date_states(&screen, &output, page.slug, name);
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
    let report = if filter.is_some() {
        "layout-report-filtered.txt"
    } else {
        "layout-report.txt"
    };
    std::fs::write(output.join(report), layout).expect("save lookbook layout report");
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
        "date-range-input" => "Booking dates",
        "dialog" => "Open dialog",
        "drawer" => "Open drawer",
        "dropdown-menu" => "Actions",
        "layer" | "popover" => "Open popover",
        "lightbox" => "Open image",
        "mobile-nav" => "Open navigation",
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

fn capture_disclosure_states(screen: &Screen, output: &Path, slug: &str, mode: &str) {
    let trigger = match slug {
        "chat-tool-calls" => "2 tool calls",
        "code-block" => "Example source",
        _ => return,
    };
    screen
        .get_by_role(Role::Button, trigger)
        .press_key(Key::Enter);
    for (elapsed, state) in [(85, "closing"), (300, "closed")] {
        screen.advance(Duration::from_millis(elapsed));
        screen
            .screenshot()
            .save_png(output.join(format!("{slug}-{mode}-{state}.png")))
            .expect("disclosure state screenshot");
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

fn capture_content_states(screen: &Screen, output: &Path, slug: &str, name: &str) {
    if slug == "carousel" {
        screen.get_by_role(Role::Button, "Next slide").click();
        screen.advance(Duration::from_millis(800));
        screen
            .screenshot()
            .save_png(output.join(format!("carousel-{name}-scrolled.png")))
            .expect("carousel scroll screenshot");
    }
    if slug == "selector" {
        screen
            .get_by_role(Role::ComboBox, "Team")
            .press_key(Key::ArrowDown);
        screen.advance(Duration::from_millis(350));
        screen
            .screenshot()
            .save_png(output.join(format!("selector-{name}-highlighted.png")))
            .expect("selector highlight screenshot");
    }
    if slug == "tree-list" {
        screen
            .get_by_role(Role::TreeItem, "src")
            .press_key(Key::ArrowLeft);
        screen.advance(Duration::from_millis(120));
        screen
            .screenshot()
            .save_png(output.join(format!("tree-list-{name}-closing.png")))
            .expect("tree exit screenshot");
    }
}

fn capture_table_states(screen: &Screen, output: &Path, slug: &str, name: &str) {
    if slug == "table" {
        screen
            .get_by_role(Role::Button, "Show active projects")
            .click();
        screen
            .get_by_role(Role::Button, "Design details")
            .press_key(Key::Enter);
        screen
            .screenshot()
            .save_png(output.join(format!("table-{name}-filtered-expanded.png")))
            .expect("table state screenshot");
        screen
            .get_by_role(Role::ColumnHeader, "Workspace")
            .scroll(72.0, 0.0);
        screen.advance(Duration::from_millis(220));
        screen
            .screenshot()
            .save_png(output.join(format!("table-{name}-pinned.png")))
            .expect("pinned column screenshot");
        screen.get_by_role(Role::Button, "Filter Status").click();
        screen.advance(Duration::from_millis(220));
        screen
            .screenshot()
            .save_png(output.join(format!("table-{name}-typed-filter.png")))
            .expect("typed filter screenshot");
    }
}

fn capture_date_states(screen: &Screen, output: &Path, slug: &str, name: &str) {
    if slug == "date-input" {
        screen
            .get_by_role(Role::TextBox, "Touch due date")
            .press_key(Key::ArrowDown);
        screen.advance(Duration::from_millis(500));
        screen
            .screenshot()
            .save_png(output.join(format!("date-input-{name}-touch-calendar.png")))
            .expect("touch date screenshot");
        screen.get_by_role(Role::Button, "October 2026").click();
        screen.advance(Duration::from_millis(350));
        screen
            .screenshot()
            .save_png(output.join(format!("date-input-{name}-touch-wheels.png")))
            .expect("touch date wheels screenshot");
    }
}
