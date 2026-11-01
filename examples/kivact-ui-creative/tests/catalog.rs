//! Role: All source chapters remain reachable in the complete, font-matched shell.
//! Optional artifacts are produced by the same public real-window driver
//! that consumers use; no library reaches into an implementation crate.
#![warn(redundant_imports)]

use kivact::test::{LayoutCheck, RenderOptions, Screen, TestFonts, render_with};
use kivact_ui_creative_catalog::{catalog, fonts};
use std::sync::Arc;
use std::time::Duration;

#[path = "catalog/creative.rs"]
mod creative;
#[path = "catalog/toolbar.rs"]
mod toolbar;

#[test]
fn creative_source_editors_scroll_in_the_original_shell() {
    let screen = mount();
    evidence(&screen, "creative-initial");
    chapters(
        &screen,
        "creative",
        &[
            "buttons",
            "numeric",
            "selection",
            "indicators",
            "transport",
            "knobs",
            "meters",
            "mixer",
            "properties",
            "docking",
            "outliner",
            "media",
            "layers",
            "timeline",
            "animation",
            "nodes",
            "grading",
            "picker",
            "viewport",
            "menus",
            "dialogs",
            "jobs",
        ],
    );
}

fn mount() -> Screen {
    let gpu = std::env::var("KIVACT_CATALOG_GPU").is_ok_and(|value| value == "1");
    render_with(
        RenderOptions {
            width: 1861.,
            height: 969.,
            scale: 1.,
            reduced_motion: true,
            window: kivact::test::WindowMode::Real,
            gpu,
            fonts: TestFonts::Only(fonts().iter().map(|&bytes| Arc::from(bytes)).collect()),
            ..RenderOptions::from_env()
        },
        catalog,
    )
}

/// Preserve the catalog layout exceptions. The "Z" axis labels of the inspector stick out of
/// their box by 1px on purpose and are ignored by test id.
fn layout_check() -> LayoutCheck {
    LayoutCheck::default().ignore_test_id("axis-label-z")
}

fn chapters(screen: &Screen, prefix: &str, chapters: &[&str]) {
    for (index, chapter) in chapters.iter().enumerate() {
        let nav = screen.get_by_test_id(&format!("{prefix}-nav-{chapter}"));
        if prefix == "creative" {
            let scroll = screen.get_by_test_id("creative-nav-scroll");
            let viewport = scroll.rect().expect("sidebar viewport");
            let item = nav.rect().expect("sidebar item");
            if item.y + item.height > viewport.y + viewport.height {
                scroll.scroll(
                    0.,
                    item.y + item.height - viewport.y - viewport.height + 12.,
                );
            }
        }
        nav.click();
        screen.advance(Duration::from_millis(400));
        assert!(nav.is_selected(), "{prefix}/{chapter} remains selected");
        let section_id = match prefix {
            "basic" => format!("basic-section-{chapter}"),
            "game" => format!("game-section-{index}"),
            "creative" => format!("creative-section-{}", index + 1),
            _ => unreachable!("known catalog scene"),
        };
        let section = screen
            .get_by_test_id(&section_id)
            .rect()
            .expect("chapter geometry");
        // Selecting the sidebar alone is insufficient: the chapter must actually
        // move into the viewport. The final sections clamp at the scroll limit.
        let final_id = match prefix {
            "basic" => format!(
                "basic-section-{}",
                chapters.last().expect("catalogue has chapters")
            ),
            "game" => format!("game-section-{}", chapters.len() - 1),
            "creative" => format!("creative-section-{}", chapters.len()),
            _ => unreachable!("known catalog scene"),
        };
        let end = screen
            .get_by_test_id(&final_id)
            .rect()
            .expect("final chapter geometry");
        let anchor = match prefix {
            "basic" => 76.0,
            "game" => 72.0,
            _ => 100.0,
        };
        let at_scroll_end = section.y > anchor + 8.0 && end.y + end.height <= 971.0;
        if at_scroll_end {
            assert!(
                (0.0..=600.0).contains(&section.y),
                "{prefix}/{chapter} visible at scroll limit: {section:?}"
            );
        } else {
            assert!(
                (section.y - anchor).abs() <= 8.0,
                "{prefix}/{chapter} scrolls to {anchor}: {section:?}"
            );
        }
        assert!(
            screen.errors().is_empty(),
            "{prefix}/{chapter}: {:?}",
            screen.errors()
        );
        screen.assert_layout_with(&crate::layout_check());
        evidence(screen, &format!("{prefix}-{chapter}"));
    }
}

/// Opt-in leaves ordinary successful tests free of filesystem IO.
fn evidence(screen: &Screen, name: &str) {
    if let Some(directory) = std::env::var_os("KIVACT_CATALOG_EVIDENCE") {
        let directory = std::path::PathBuf::from(directory);
        std::fs::create_dir_all(&directory).expect("create evidence directory");
        screen
            .screenshot()
            .save_png(directory.join(format!("{name}.png")))
            .expect("save evidence screenshot");
        std::fs::write(
            directory.join(format!("{name}-a11y.txt")),
            screen.accessibility_tree(),
        )
        .expect("save published accessibility tree");
        std::fs::write(
            directory.join(format!("{name}-retained.txt")),
            screen.debug(),
        )
        .expect("save retained semantic tree");
    }
}

#[path = "catalog/audio.rs"]
mod audio;
