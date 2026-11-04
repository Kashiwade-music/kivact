//! Role: Tests for standard UI components and their semantics.
//!
//! The standard components of `kivact::ui` (button, checkbox, switch,
//! text_input) through the public API: their roles, names and states, the
//! keys, clicks and assistive-technology actions that operate them, how
//! variants, sizes, the theme's tokens and a theme's recipes change their
//! look (checked on the frame's pixels), that what the caller sets wins over
//! the recipe, and local painting without additional semantic nodes.

use crate::paint::{distance, pixel};
use crate::support::mount;
use kivact::prelude::*;
use kivact::style::{Align, Justify, Style};
use kivact::style::{Color, Direction, rgb};
use kivact::test::{RenderOptions, Screen};
use kivact::ui::Look;
use kivact::ui::{
    ButtonState, ButtonVariant, TextInputState, UiRecipes, button, checkbox, switch, text_input,
};
use kivact::view::A11yAction;

const RED: u32 = 0xff0000;
const GREEN: u32 = 0x00ff00;
const BLUE: u32 = 0x0000ff;
const YELLOW: u32 = 0xffff00;

fn theme_with(recipes: UiRecipes) -> Theme {
    Theme::light().with(recipes)
}

fn options(theme: Theme) -> RenderOptions {
    RenderOptions::from_env().theme(theme).reduced_motion()
}

/// The pixel `dx` right of and `dy` below the top left of `id`.
fn pixel_in(screen: &Screen, id: &str, dx: f64, dy: f64) -> u32 {
    let r = screen.get_by_test_id(id).rect().expect("laid out");
    let shot = screen.screenshot();
    pixel(&shot, (r.x + dx) as u32, (r.y + dy) as u32)
}

/// The pixel at the vertical middle of `id`, `dx` right of its left edge.
fn pixel_mid(screen: &Screen, id: &str, dx: f64) -> u32 {
    let h = screen.get_by_test_id(id).rect().expect("laid out").height;
    pixel_in(screen, id, dx, (h / 2.0).floor())
}

// ---------------------------------------------------------------------------
// Roles, names, states and operation.

#[test]
fn a_button_is_a_named_button_that_enter_space_click_and_invoke_press() {
    let (screen, presses) = mount(|| {
        let presses = signal(0);
        (
            presses,
            column((
                button("Save").on_press(move || presses.update(|n| *n += 1)),
                text(move || format!("pressed {}", presses.get())),
            )),
        )
    });
    let save = screen.get_by_role(Role::Button, "Save");
    assert_eq!(save.name(), "Save");
    assert!(!save.is_disabled());
    assert!(save.a11y_actions().contains(&A11yAction::Activate));
    save.click();
    assert_eq!(presses.get_untracked(), 1, "click");
    assert!(save.is_focused(), "a click focuses a button");
    save.press_key(Key::Enter);
    assert_eq!(presses.get_untracked(), 2, "Enter");
    save.press_key(Key::Space);
    assert_eq!(presses.get_untracked(), 3, "Space");
    save.a11y(A11yAction::Activate);
    assert_eq!(presses.get_untracked(), 4, "Activate");
    screen.get_by_text("pressed 4");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_disabled_button_is_disabled_and_ignores_every_way_of_pressing_it() {
    let (screen, (presses, off)) = mount(|| {
        let presses = signal(0);
        let off = signal(true);
        (
            (presses, off),
            button("Delete")
                .variant(ButtonVariant::Danger)
                .disabled(off)
                .on_press(move || presses.update(|n| *n += 1)),
        )
    });
    let delete = screen.get_by_role(Role::Button, "Delete");
    assert!(delete.is_disabled());
    delete.force_click();
    delete.a11y(A11yAction::Activate);
    screen.press_key(Key::Enter);
    screen.press_key(Key::Space);
    assert_eq!(presses.get_untracked(), 0);
    off.set(false);
    screen.run_until_idle();
    assert!(!delete.is_disabled());
    delete.click();
    assert_eq!(presses.get_untracked(), 1);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_checkbox_and_a_switch_publish_their_role_and_state_and_flip_on_space_and_click() {
    let (screen, (agreed, notify)) = mount(|| {
        let agreed = signal(false);
        let notify = signal(true);
        (
            (agreed, notify),
            column((checkbox("Agree", agreed), switch("Notify", notify))),
        )
    });
    let check = screen.get_by_role(Role::CheckBox, "Agree");
    let on = screen.get_by_role(Role::Switch, "Notify");
    assert_eq!(check.checked(), Some(CheckedState::Unchecked));
    assert_eq!(on.checked(), Some(CheckedState::Checked));
    check.click();
    on.click();
    assert!(agreed.get_untracked());
    assert!(!notify.get_untracked());
    assert_eq!(check.checked(), Some(CheckedState::Checked));
    assert_eq!(on.checked(), Some(CheckedState::Unchecked));
    check.press_key(Key::Space);
    on.press_key(Key::Space);
    assert!(!agreed.get_untracked());
    assert!(notify.get_untracked());
    check.press_key(Key::Enter);
    on.press_key(Key::Enter);
    assert!(!agreed.get_untracked(), "Enter does not flip a check box");
    assert!(notify.get_untracked(), "nor a switch");
    check.a11y(A11yAction::Toggle);
    on.a11y(A11yAction::Activate);
    assert!(agreed.get_untracked());
    assert!(!notify.get_untracked());
    // A change of the bound signal shows in the state.
    notify.set(true);
    screen.run_until_idle();
    assert_eq!(on.checked(), Some(CheckedState::Checked));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_text_input_is_a_named_text_box_bound_to_its_signal() {
    let (screen, (name, submitted)) = mount(|| {
        let name = signal("Ada".to_string());
        let submitted = signal(0);
        (
            (name, submitted),
            column((
                text_input("Name", name)
                    .placeholder("Your name")
                    .on_submit(move || submitted.update(|n| *n += 1)),
                text(move || format!("Hello, {}", name.get())),
            )),
        )
    });
    let input = screen.get_by_role(Role::TextBox, "Name");
    assert_eq!(input.value().as_deref(), Some("Ada"));
    input.press_key(Key::End);
    input.type_text(" Lovelace");
    assert_eq!(name.get_untracked(), "Ada Lovelace");
    screen.get_by_text("Hello, Ada Lovelace");
    input.press_key(Key::Enter);
    assert_eq!(submitted.get_untracked(), 1);
    name.set("Grace".to_string());
    screen.run_until_idle();
    assert_eq!(input.value().as_deref(), Some("Grace"));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

// ---------------------------------------------------------------------------
// Looks: variants, sizes, tokens, themes.

#[test]
fn the_tokens_of_the_theme_re_skin_every_component() {
    let orange = Theme::light();
    let colors = kivact::style::theme::Palette {
        accent: rgb(0xff8800),
        ..orange.colors().clone()
    };
    let orange = orange.with_colors(colors).with_control_height(40.0);
    let screen = kivact::test::render_with(options(orange), || {
        row((
            button("Go").test_id("go"),
            switch("On", signal(true)).test_id("switch"),
        ))
        .style(style().gap(24).padding(8).align_items(Align::Start))
    });
    assert!(distance(pixel_mid(&screen, "go", 4.0), 0xff8800) <= 2);
    assert_eq!(
        screen.get_by_test_id("go").rect().expect("laid out").height,
        40.0,
        "control_height"
    );
    // The track of the switch that is on is the accent.
    assert!(distance(pixel_mid(&screen, "switch", 10.0), 0xff8800) <= 2);
}

// ---------------------------------------------------------------------------
// Recipes.

fn flat_button(_: &Theme, state: &Look<ButtonState>) -> Style {
    let variant = state
        .variant::<ButtonVariant>()
        .copied()
        .unwrap_or_default();
    style()
        .size(200, 40)
        .background(match variant {
            ButtonVariant::Danger => rgb(BLUE),
            _ => rgb(RED),
        })
        .direction(Direction::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .color(Color::WHITE)
}

fn plain_input(_: &Theme, _: &Look<TextInputState>) -> Style {
    style().height(36).background(rgb(YELLOW))
}

/// The default recipes with the button replaced.
fn recipes_with_button(
    recipe: impl Fn(&Theme, &Look<ButtonState>) -> Style + Send + Sync + 'static,
) -> UiRecipes {
    UiRecipes::default().button(|_| recipe)
}

#[test]
fn a_recipe_in_the_theme_replaces_the_look_of_its_component_only() {
    let recipes = UiRecipes::default()
        .button(|_| flat_button)
        .text_input(|_| plain_input);
    let screen = kivact::test::render_with(options(theme_with(recipes)), || {
        column((
            button("Go").test_id("go"),
            button("Stop")
                .variant(ButtonVariant::Danger)
                .test_id("stop"),
            text_input("Name", signal(String::new())).test_id("name"),
            checkbox("Ready", signal(false)).test_id("ready"),
        ))
        .style(style().gap(8).padding(8).align_items(Align::Start))
    });
    let size = |id: &str| screen.get_by_test_id(id).rect().expect("laid out");
    assert_eq!((size("go").width, size("go").height), (200.0, 40.0));
    assert_eq!(pixel_mid(&screen, "go", 4.0), RED);
    assert_eq!(
        pixel_mid(&screen, "stop", 4.0),
        BLUE,
        "the variant reaches the recipe"
    );
    assert_eq!(pixel_mid(&screen, "name", 4.0), YELLOW);
    assert_eq!(size("name").height, 36.0);
    // The check box keeps the default recipe: its box is drawn.
    assert_eq!(
        screen.get_by_test_id("ready").children().len(),
        1,
        "the default check box paints on its root and has only its label"
    );
}

#[test]
fn what_the_caller_sets_comes_after_the_recipe_and_wins() {
    let recipes = recipes_with_button(flat_button);
    let screen = kivact::test::render_with(options(theme_with(recipes)), || {
        column((
            button("Plain").test_id("plain"),
            button("Mine")
                .style(style().background(rgb(GREEN)).width(100))
                .test_id("mine"),
            button("Themed")
                .style(move || style().height(50))
                .test_id("themed"),
        ))
        .style(style().gap(8).padding(8).align_items(Align::Start))
    });
    let size = |id: &str| screen.get_by_test_id(id).rect().expect("laid out");
    assert_eq!(pixel_mid(&screen, "plain", 4.0), RED);
    assert_eq!(
        pixel_mid(&screen, "mine", 4.0),
        GREEN,
        "the caller's background wins"
    );
    assert_eq!(size("mine").width, 100.0, "and the caller's width");
    assert_eq!(size("mine").height, 40.0, "the rest is the recipe's");
    assert_eq!(size("themed").height, 50.0);
    // A name set by the caller replaces the label as the accessible name.
    let (screen, ()) = mount(|| ((), button("Visible").name("Accessible")));
    screen.get_by_role(Role::Button, "Accessible");
}

// ---------------------------------------------------------------------------
// One-line labels.

#[test]
fn a_kits_theme_gives_the_standard_components_the_kits_look() {
    // Same components, three themes: the core's, a re-skinned core theme and
    // the kit's, whose theme installs its recipes (a taller button, the
    // kit's blue, a drawn switch of its own proportions).
    let build = || {
        row((
            button("Go").test_id("go"),
            switch("On", signal(true)).test_id("switch"),
        ))
        .style(style().gap(24).padding(8).align_items(Align::Start))
    };
    let core = kivact::test::render_with(options(Theme::light()), build);
    let kit = kivact::test::render_with(
        options(kivact_ui_pop::theme(kivact::style::ThemeMode::Light)),
        build,
    );
    let height =
        |screen: &Screen, id: &str| screen.get_by_test_id(id).rect().expect("laid out").height;
    assert_eq!(height(&core, "go"), 28.0);
    assert_eq!(height(&kit, "go"), 36.0, "the kit's control height");
    assert_ne!(pixel_mid(&core, "go", 4.0), pixel_mid(&kit, "go", 4.0));
    assert!(
        distance(pixel_mid(&kit, "go", 4.0), 0x2b8bd8) <= 8,
        "the kit's blue, not the core accent: {:06x}",
        pixel_mid(&kit, "go", 4.0)
    );
    // The kit's recipes are the same table a caller can read back.
    let theme = kivact_ui_pop::theme(kivact::style::ThemeMode::Light);
    let recipes = theme.get::<UiRecipes>();
    assert_ne!(*recipes, UiRecipes::default());
    assert!(kit.errors().is_empty(), "{:?}", kit.errors());
}
