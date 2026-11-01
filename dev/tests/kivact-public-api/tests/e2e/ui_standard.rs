//! Role: Tests for standard UI components and their semantics.
//!
//! The standard components of `kivact::ui` (button, checkbox, switch,
//! text_input) through the public API: their roles, names and states, the
//! keys, clicks and assistive-technology actions that operate them, how
//! variants, sizes, the theme's tokens and a theme's recipes change their
//! look (checked on the frame's pixels), that what the caller sets wins over
//! the recipe, and local painting without additional semantic nodes.

use crate::paint::{distance, expect, pixel};
use crate::support::{mount, mount_with};
use kivact::binding;
use kivact::draw::{Draw, Rect, Size};
use kivact::prelude::*;
use kivact::primitive::NodeExt;
use kivact::style::{Align, Justify, Style};
use kivact::style::{Color, Direction, rgb};
use kivact::test::{RenderOptions, Screen};
use kivact::ui::Look;
use kivact::ui::{
    ButtonState, ButtonVariant, CheckState, ControlSize, TextInputState, UiRecipes, button,
    checkbox, switch, text_input,
};
use kivact::view::A11yAction;
use kivact::view::provide_theme;

const RED: u32 = 0xff0000;
const GREEN: u32 = 0x00ff00;
const BLUE: u32 = 0x0000ff;
const YELLOW: u32 = 0xffff00;

fn theme_with(recipes: UiRecipes) -> Theme {
    Theme::light().with(recipes)
}

fn options(theme: Theme) -> RenderOptions {
    RenderOptions {
        theme,
        reduced_motion: true,
        ..RenderOptions::from_env()
    }
}

/// The pixel `dx` right of and `dy` below the top left of `id`.
fn pixel_in(screen: &Screen, id: &str, dx: f32, dy: f32) -> u32 {
    let r = screen.get_by_test_id(id).rect().expect("laid out");
    let shot = screen.screenshot();
    pixel(&shot, (r.x + dx) as u32, (r.y + dy) as u32)
}

/// The pixel at the vertical middle of `id`, `dx` right of its left edge.
fn pixel_mid(screen: &Screen, id: &str, dx: f32) -> u32 {
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
    assert_eq!(check.is_checked(), Some(false));
    assert_eq!(on.is_checked(), Some(true));
    check.click();
    on.click();
    assert!(agreed.get_untracked());
    assert!(!notify.get_untracked());
    assert_eq!(check.is_checked(), Some(true));
    assert_eq!(on.is_checked(), Some(false));
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
    assert_eq!(on.is_checked(), Some(true));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_checkbox_can_be_mixed_and_a_disabled_toggle_ignores_input() {
    let (screen, (mixed, locked, value)) = mount(|| {
        let value = signal(false);
        let mixed = signal(true);
        let locked = signal(true);
        // Pressing a mixed check box writes true and the caller clears it.
        let binding = binding(value.into_value(), move |v: bool| {
            mixed.set(false);
            value.set(v);
        });
        (
            (mixed, locked, value),
            column((
                checkbox("All", binding).indeterminate(mixed),
                switch("Gated", value).disabled(locked),
            )),
        )
    });
    let all = screen.get_by_role(Role::CheckBox, "All");
    assert!(all.is_indeterminate());
    all.click();
    assert!(value.get_untracked());
    assert!(!mixed.get_untracked());
    assert!(!all.is_indeterminate());
    let gated = screen.get_by_role(Role::Switch, "Gated");
    assert!(gated.is_disabled());
    value.set(false);
    screen.run_until_idle();
    gated.force_click();
    gated.a11y(A11yAction::Toggle);
    assert!(!value.get_untracked());
    locked.set(false);
    screen.run_until_idle();
    gated.click();
    assert!(value.get_untracked());
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

#[test]
fn a_disabled_text_input_cannot_be_edited() {
    let (screen, name) = mount(|| {
        let name = signal(String::new());
        (name, text_input("Locked", name).disabled(true))
    });
    let input = screen.get_by_role(Role::TextBox, "Locked");
    assert!(input.is_disabled());
    input.force_click();
    screen.type_text("typed");
    assert_eq!(name.get_untracked(), "");
}

// ---------------------------------------------------------------------------
// Looks: variants, sizes, tokens, themes.

#[test]
fn variants_and_sizes_change_the_look() {
    let screen = kivact::test::render_with(options(Theme::light()), || {
        row((
            button("Primary").test_id("primary"),
            button("Secondary")
                .variant(ButtonVariant::Secondary)
                .test_id("secondary"),
            button("Ghost")
                .variant(ButtonVariant::Ghost)
                .test_id("ghost"),
            button("Danger")
                .variant(ButtonVariant::Danger)
                .test_id("danger"),
            button("Outline")
                .variant(ButtonVariant::Outline)
                .test_id("outline"),
            button("Link").variant(ButtonVariant::Link).test_id("link"),
            button("Small").size(ControlSize::Small).test_id("small"),
        ))
        .style(style().gap(8).padding(8).align_items(Align::Start))
    });
    // The fill is the token of the variant: Accent, Danger, Surface, none.
    expect(&screen.screenshot(), 0, 0, 0xffffff, 0, "the page is white");
    let fill = |id: &str| pixel_mid(&screen, id, 4.0);
    assert!(
        distance(fill("primary"), 0x2563eb) <= 2,
        "{:06x}",
        fill("primary")
    );
    assert!(
        distance(fill("danger"), 0xdc2626) <= 2,
        "{:06x}",
        fill("danger")
    );
    assert!(distance(fill("secondary"), 0xffffff) <= 2);
    assert!(distance(fill("ghost"), 0xffffff) <= 2);
    // The secondary button is outlined: its border pixel is the Border token.
    assert!(distance(pixel_mid(&screen, "secondary", 0.0), 0xd1d5db) <= 8);
    assert_eq!(
        pixel_mid(&screen, "ghost", 0.0),
        0xffffff,
        "a ghost button has no outline"
    );
    // Outline is the surface with the Border token; Link has neither fill nor border.
    assert!(distance(fill("outline"), 0xffffff) <= 2);
    assert!(distance(pixel_mid(&screen, "outline", 0.0), 0xd1d5db) <= 8);
    assert_eq!(
        pixel_mid(&screen, "link", 0.0),
        0xffffff,
        "a link has no outline"
    );
    // A medium button is the theme's control height, a small one is lower.
    let height = |id: &str| screen.get_by_test_id(id).rect().expect("laid out").height;
    assert_eq!(height("primary"), 28.0);
    assert!(height("small") < height("primary"), "{}", height("small"));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn hover_and_press_tint_the_button_with_the_theme_tokens() {
    let screen = kivact::test::render_with(options(Theme::light()), || {
        column((
            button("Go").test_id("go"),
            button("Quiet")
                .variant(ButtonVariant::Ghost)
                .test_id("quiet"),
        ))
        .style(style().gap(8).padding(8).align_items(Align::Start))
    });
    let rest = pixel_mid(&screen, "go", 4.0);
    let quiet_rest = pixel_mid(&screen, "quiet", 4.0);
    screen.get_by_test_id("go").hover();
    let hovered = pixel_mid(&screen, "go", 4.0);
    assert_ne!(hovered, rest, "hover tints the fill");
    screen.get_by_test_id("quiet").hover();
    let quiet_hovered = pixel_mid(&screen, "quiet", 4.0);
    assert_ne!(quiet_hovered, quiet_rest, "hover shows the ghost button");
    assert_eq!(
        pixel_mid(&screen, "go", 4.0),
        rest,
        "the first button is back at rest"
    );
}

#[test]
fn a_text_input_has_the_field_look_and_its_size_follows_the_size() {
    let screen = kivact::test::render_with(options(Theme::light()), || {
        column((
            text_input("Medium", signal(String::new())).test_id("medium"),
            text_input("Small", signal(String::new()))
                .size(ControlSize::Small)
                .test_id("small"),
        ))
        .style(style().gap(8).padding(8).align_items(Align::Start))
    });
    let rect = |id: &str| screen.get_by_test_id(id).rect().expect("laid out");
    assert_eq!(rect("medium").height, 28.0);
    assert!(rect("small").height < 28.0);
    // A white field with the Border token's outline.
    assert!(distance(pixel_mid(&screen, "medium", 4.0), 0xffffff) <= 2);
    assert!(distance(pixel_mid(&screen, "medium", 0.0), 0xd1d5db) <= 12);
}

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

#[test]
fn a_dark_theme_changes_the_look_of_the_same_components() {
    let build = || button("Go").test_id("go");
    let light = kivact::test::render_with(options(Theme::light()), build);
    let dark = kivact::test::render_with(options(Theme::dark()), build);
    assert_ne!(
        pixel_mid(&light, "go", 4.0),
        pixel_mid(&dark, "go", 4.0),
        "the accent token differs"
    );
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

fn left_strip(_: &Theme, _: &ButtonState, d: &mut Draw, size: Size) {
    d.fill_rect(Rect::new(0.0, 0.0, 60.0, size.height), rgb(GREEN));
}

fn flat_toggle(_: &Theme, _: &Look<CheckState>) -> Style {
    style().size(120, 30)
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

#[test]
fn a_recipes_drawing_and_the_callers_draw_and_draw_over_are_all_painted() {
    let recipes = recipes_with_button(painted_button);
    let screen = kivact::test::render_with(options(theme_with(recipes)), || {
        button("Go")
            // Behind the recipe's layer: seen where the recipe draws nothing.
            .draw(|d, size| d.fill_rect(Rect::new(0.0, 0.0, 130.0, size.height), rgb(YELLOW)))
            // Over everything.
            .draw_over(|d, size| d.fill_rect(Rect::new(150.0, 0.0, 50.0, size.height), rgb(BLUE)))
            .test_id("go")
    });
    let at = |dx: f32| pixel_mid(&screen, "go", dx);
    assert_eq!(
        at(10.0),
        GREEN,
        "the recipe's layer is above the caller's draw"
    );
    assert_eq!(
        at(125.0),
        YELLOW,
        "the caller's draw is behind the recipe's layer"
    );
    assert_eq!(at(140.0), RED, "the style's background elsewhere");
    assert_eq!(
        at(180.0),
        BLUE,
        "the caller's draw_over is above everything"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn local_paint_adds_no_semantic_node_and_a_press_on_it_reaches_the_button() {
    let drawing = recipes_with_button(painted_button);
    let plain = recipes_with_button(flat_button);
    let tree = |recipes: UiRecipes| {
        let (screen, presses) = mount_with(options(theme_with(recipes)), || {
            let presses = signal(0);
            (
                presses,
                button("Go")
                    .test_id("go")
                    .on_press(move || presses.update(|n| *n += 1)),
            )
        });
        (screen, presses)
    };
    let (with, with_presses) = tree(drawing);
    let (without, _) = tree(plain);
    assert_eq!(
        with.get_by_test_id("go").children().len(),
        1,
        "recipe painting adds no child beyond the content row"
    );
    assert_eq!(
        without.get_by_test_id("go").children().len(),
        1,
        "a recipe without one adds none: the button is its label and itself"
    );
    // The layer has no role: it is not found by role, and the nodes of the
    // accessibility tree that have one are the same with and without it.
    assert_eq!(with.all_by_role(Role::Button, "Go").len(), 1);
    let roles = |screen: &Screen| -> Vec<String> {
        screen
            .accessibility_tree()
            .lines()
            .filter(|line| !line.trim_start().starts_with("Generic"))
            .map(str::to_owned)
            .collect()
    };
    assert_eq!(roles(&with), roles(&without));
    let layer = with.get_by_test_id("go").children()[0];
    assert_eq!(layer.role(), None, "a plain box: no role");
    assert!(!layer.is_focused());
    assert!(layer.a11y_actions().is_empty());
    // A press on the layer (the left strip, away from the label) is a press
    // on the button.
    let go = with.get_by_role(Role::Button, "Go");
    let r = go.rect().expect("laid out");
    with.click_at(r.x + 10.0, r.y + r.height / 2.0);
    assert_eq!(with_presses.get_untracked(), 1);
    assert!(go.is_focused(), "and it focuses the button");
    assert!(with.errors().is_empty(), "{:?}", with.errors());
}

#[test]
fn the_default_button_has_no_extra_node() {
    let (screen, ()) = mount(|| ((), button("Go").test_id("go")));
    assert_eq!(screen.get_by_test_id("go").children().len(), 1);
}

fn square_toggle_draw(_: &Theme, look: &CheckState, d: &mut Draw, size: Size) {
    d.fill_rect(
        Rect::new(0.0, 0.0, 20.0, size.height),
        rgb(if look.checked.is_checked() {
            GREEN
        } else {
            RED
        }),
    );
}

#[test]
fn a_toggle_recipe_draws_from_the_state_and_follows_it() {
    let recipes = UiRecipes::default().checkbox(|_| painted_toggle);
    let (screen, on) = mount_with(options(theme_with(recipes)), || {
        let on = signal(false);
        (on, checkbox("Mark", on).test_id("mark"))
    });
    assert_eq!(pixel_mid(&screen, "mark", 5.0), RED);
    screen.get_by_role(Role::CheckBox, "Mark").click();
    assert!(on.get_untracked());
    assert_eq!(
        pixel_mid(&screen, "mark", 5.0),
        GREEN,
        "the drawing follows the state"
    );
}

#[test]
fn the_default_check_box_and_switch_draw_their_state() {
    let (screen, (check, switch_on)) = mount_with(options(Theme::light()), || {
        let check = signal(false);
        let switch_on = signal(false);
        (
            (check, switch_on),
            column((
                checkbox("Check", check).test_id("check"),
                switch("Switch", switch_on).test_id("switch"),
            ))
            .style(style().gap(8).padding(8).align_items(Align::Start)),
        )
    });
    // Off: a white box with a border, and a gray track.
    let before = (
        pixel_mid(&screen, "check", 9.0),
        pixel_mid(&screen, "switch", 28.0),
    );
    assert!(distance(before.0, 0xffffff) <= 2, "{:06x}", before.0);
    assert!(distance(before.1, 0xd1d5db) <= 2, "{:06x}", before.1);
    check.set(true);
    switch_on.set(true);
    screen.run_until_idle();
    // On: the accent fills both.
    let on = (
        pixel_mid(&screen, "check", 6.0),
        pixel_mid(&screen, "switch", 6.0),
    );
    assert!(distance(on.0, 0x2563eb) <= 2, "{:06x}", on.0);
    assert!(distance(on.1, 0x2563eb) <= 2, "{:06x}", on.1);
}

#[test]
fn switching_the_theme_at_run_time_restyles_paint_without_extra_nodes() {
    let bare = UiRecipes::default().checkbox(|_| flat_toggle);
    let (screen, theme) = mount(|| {
        let theme = signal(Theme::light());
        (
            theme,
            provide_theme(theme, || checkbox("Mark", signal(false)).test_id("mark")),
        )
    });
    let layers = || screen.get_by_test_id("mark").children().len();
    assert_eq!(layers(), 1, "the default check box paints on its root");
    theme.set(theme_with(bare));
    screen.run_until_idle();
    assert_eq!(layers(), 1, "a recipe without a drawing has no layer");
    assert_eq!(
        screen
            .get_by_test_id("mark")
            .rect()
            .expect("laid out")
            .width,
        120.0
    );
    theme.set(Theme::light());
    screen.run_until_idle();
    assert_eq!(layers(), 1, "restoring paint adds no child");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

// ---------------------------------------------------------------------------
// One-line labels.

#[test]
fn labels_stay_on_one_line_and_are_cut_by_an_ellipsis_in_a_narrow_control() {
    const LONG: &str = "A label that is much longer than its button";
    let (screen, ()) = mount(|| {
        (
            (),
            column((
                button(LONG).style(style().width(90)).test_id("long"),
                checkbox(LONG, signal(false))
                    .style(style().width(110))
                    .test_id("check"),
            ))
            .style(style().gap(8).padding(8).align_items(Align::Start)),
        )
    });
    // The accessible name is the whole label.
    let long = screen.get_by_role(Role::Button, LONG);
    let button_box = long.rect().expect("laid out");
    assert_eq!(button_box.width, 90.0);
    assert_eq!(button_box.height, 28.0, "one line high");
    for text_node in screen.all_by_text(LONG) {
        let r = text_node.rect().expect("laid out");
        assert!(r.height < 28.0, "one line: {r:?}");
    }
    let outer = |id: &str| screen.get_by_test_id(id).rect().expect("laid out");
    for text_node in screen.all_by_text(LONG) {
        let r = text_node.rect().expect("laid out");
        let owner = if r.y < outer("check").y {
            outer("long")
        } else {
            outer("check")
        };
        assert!(
            r.x + r.width <= owner.x + owner.width + 0.5,
            "the label {r:?} stays inside {owner:?}"
        );
    }
    assert_eq!(
        screen
            .get_by_test_id("check")
            .rect()
            .expect("laid out")
            .height,
        28.0
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn an_icon_comes_before_the_label_inside_the_button() {
    let (screen, ()) = mount(|| {
        (
            (),
            button("Add")
                .icon(node().style(style().size(12, 12)).test_id("icon"))
                .test_id("add"),
        )
    });
    let add = screen.get_by_test_id("add").rect().expect("laid out");
    let icon = screen.get_by_test_id("icon").rect().expect("laid out");
    let label = screen.get_by_text("Add").rect().expect("laid out");
    assert!(icon.x + icon.width <= label.x, "{icon:?} {label:?}");
    assert!(icon.x >= add.x && label.x + label.width <= add.x + add.width + 0.5);
}

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

fn painted_button(theme: &Theme, state: &Look<ButtonState>) -> Style {
    let (theme, state) = (theme.clone(), state.clone());
    flat_button(&theme, &state).paint(move |draw, size| left_strip(&theme, &state, draw, size))
}
fn painted_toggle(theme: &Theme, state: &Look<CheckState>) -> Style {
    let (theme, state) = (theme.clone(), state.clone());
    flat_toggle(&theme, &state)
        .paint(move |draw, size| square_toggle_draw(&theme, &state, draw, size))
}
