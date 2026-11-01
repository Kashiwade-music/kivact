//! Role: Tests the semantic tone of a toast on real windows.
//!
//! The roles the tones are announced with, the icon a toast draws for its tone
//! from the theme's icon set, the icon that replaces it, and the recipe that
//! styles it.

use crate::support::mount_with;
use kivact::interaction::ToastState;
use kivact::prelude::*;
use kivact::style::{IconSet, Theme};
use kivact::test::{RenderOptions, Screen};
use kivact::ui::{Look, Toast, ToastRecipes, Tone, UiRecipes, toast_region, toaster};
use std::sync::{Arc, Mutex};

fn show(theme: Theme, toasts: Vec<Toast>) -> Screen {
    let (screen, ()) = mount_with(
        RenderOptions {
            theme,
            reduced_motion: true,
            ..RenderOptions::from_env()
        },
        move || {
            let toaster = toaster();
            toaster.max_visible(8);
            for toast in toasts {
                toaster.push(toast);
            }
            ((), toast_region(toaster))
        },
    );
    screen.run_until_idle();
    screen
}

fn title_x(screen: &Screen, title: &str) -> f32 {
    screen.get_by_text(title).rect().expect("laid out").x
}

#[test]
fn warning_and_danger_are_alerts_and_the_other_tones_statuses() {
    let tones = [
        ("Neutral", Tone::Neutral, Role::Status),
        ("Info", Tone::Info, Role::Status),
        ("Success", Tone::Success, Role::Status),
        ("Accent", Tone::Accent, Role::Status),
        ("Warning", Tone::Warning, Role::Alert),
        ("Danger", Tone::Danger, Role::Alert),
    ];
    let screen = show(
        Theme::light(),
        tones
            .iter()
            .map(|(title, tone, _)| Toast::new(*title).tone(*tone))
            .collect(),
    );
    for (title, _, role) in tones {
        screen.get_by_role(role, title);
    }
    assert!(Tone::Danger.is_urgent() && !Tone::Accent.is_urgent());
    assert_eq!(Toast::new("x").tone, Tone::Neutral);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_tone_draws_the_icon_the_theme_names_for_it_and_a_neutral_toast_has_none() {
    let screen = show(
        Theme::light(),
        vec![
            Toast::new("Plain"),
            Toast::new("Saved").tone(Tone::Success),
            Toast::new("Custom")
                .tone(Tone::Success)
                .icon(|| text("MARK").test_id("mark")),
        ],
    );
    let plain = title_x(&screen, "Plain");
    assert!(
        title_x(&screen, "Saved") > plain + 8.0,
        "the tone's icon takes room before the title"
    );
    assert_eq!(screen.all_by_test_id("mark").len(), 1);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_theme_without_the_names_draws_no_icon_and_reports_nothing() {
    let theme = Theme::light().with_icons(IconSet::standard().with(
        "x",
        &b"<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24'/>"[..],
    ));
    let bare = Theme::light().with_icons(IconSet::new().with(
        "x",
        &b"<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24'/>"[..],
    ));
    let with = show(
        theme,
        vec![Toast::new("Plain"), Toast::new("Saved").tone(Tone::Success)],
    );
    let without = show(
        bare,
        vec![Toast::new("Plain"), Toast::new("Saved").tone(Tone::Success)],
    );
    assert!(title_x(&with, "Saved") > title_x(&with, "Plain") + 8.0);
    assert_eq!(
        title_x(&without, "Saved"),
        title_x(&without, "Plain"),
        "no `toast-success` in the set: no icon"
    );
    assert!(without.errors().is_empty(), "{:?}", without.errors());
}

#[test]
fn the_icon_recipe_sees_the_tone() {
    let seen = Arc::new(Mutex::new(Vec::<Tone>::new()));
    let log = seen.clone();
    let recipes = UiRecipes::default().toast(move |table: ToastRecipes| {
        let log = log.clone();
        table.icon(move |prev| {
            move |theme: &Theme, look: &Look<ToastState>| {
                log.lock().unwrap().push(look.tone);
                prev(theme, look)
            }
        })
    });
    let screen = show(
        Theme::light().with(recipes),
        vec![Toast::new("Failed").tone(Tone::Danger), Toast::new("Plain")],
    );
    assert!(seen.lock().unwrap().contains(&Tone::Danger));
    assert!(
        !seen.lock().unwrap().contains(&Tone::Neutral),
        "none for a neutral toast"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
