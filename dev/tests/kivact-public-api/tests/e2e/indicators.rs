//! Role: Verify public indicator fallbacks, motion policy and typed shortcuts.
use kivact::platform::{self, CommandKey};
use kivact::prelude::*;
use kivact::test::{RenderOptions, render, render_with};
use kivact::ui;
use kivact::view::{Asset, Modifiers};
use std::sync::{Arc, Mutex};
use std::time::Duration;
#[test]
fn avatar_falls_back_to_initials_when_the_image_fails() {
    let (screen, source) = crate::support::mount(|| {
        let source = signal(Some(Asset::from(include_bytes!("../fixtures/rgba.png"))));
        (source, ui::avatar("Ada Lovelace").src(source))
    });
    screen.get_by_role(Role::Image, "Ada Lovelace");
    assert!(screen.query_by_text("AL").is_none());
    source.set(Some(Asset::from("missing-avatar.png")));
    screen.run_until_idle();
    screen.get_by_text("AL");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn skeleton_stops_with_reduced_motion() {
    let samples = Arc::new(Mutex::new(Vec::new()));
    let recorded = samples.clone();
    let recipes = ui::UiRecipes::default().skeleton(|previous| {
        move |theme, state| {
            recorded
                .lock()
                .expect("invariant: sample lock is healthy")
                .push(state.pulse);
            previous(theme, state)
        }
    });
    let screen = render_with(
        RenderOptions::default().theme(Theme::light().with(recipes)),
        || ui::skeleton().lines(2).shape(ui::SkeletonShape::Text),
    );
    screen.advance(Duration::from_millis(200));
    assert!(
        !samples
            .lock()
            .expect("invariant: sample lock is healthy")
            .is_empty()
    );
    screen.set_reduced_motion(true);
    screen.run_until_idle();
    samples
        .lock()
        .expect("invariant: sample lock is healthy")
        .clear();
    screen.advance(Duration::from_millis(200));
    screen.advance(Duration::from_millis(200));
    assert!(
        samples
            .lock()
            .expect("invariant: sample lock is healthy")
            .iter()
            .all(|sample| *sample == 1.0)
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
#[test]
fn kbd_uses_the_platform_command_key() {
    let screen = render(|| {
        column((
            ui::kbd_combo(vec![Key::Char('K')]).modifiers(move || Modifiers {
                meta: true,
                ..Modifiers::default()
            }),
            text(move || {
                if platform::capabilities().get().command_key == CommandKey::Command {
                    "command-platform"
                } else {
                    "control-platform"
                }
            }),
        ))
    });
    let name = if screen.query_by_text("command-platform").is_some() {
        "Command + K"
    } else {
        "Control + K"
    };
    screen.get_by_role(Role::Image, name);
    screen.get_by_text(if name == "Command + K" {
        "\u{2318}"
    } else {
        "Ctrl"
    });
    screen.get_by_text("K");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
