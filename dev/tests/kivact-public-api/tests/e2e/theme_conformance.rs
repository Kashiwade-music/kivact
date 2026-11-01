//! Role: Check standard semantics, layout, recipes and focus paint in all kit themes.
use crate::paint::distance;
use kivact::prelude::*;
use kivact::style::{Align, ColorToken, Style, ThemeMode};
use kivact::test::{RenderOptions, Screen, Screenshot, TestFonts, render_with_handle};
use kivact::ui::{self, ListItem, Look, Recipe, UiRecipes};
use std::sync::Arc;

static INTER: &[u8] = include_bytes!("../../../../../crates/arvane/tests/fonts/InterVariable.ttf");

fn choices() -> Vec<ListItem<&'static str>> {
    vec![
        ListItem::new("a", "Alpha"),
        ListItem::new("b", "Beta").disabled(true),
        ListItem::new("c", "Gamma"),
    ]
}

/// Sample the straight top edge, clear of rounded corners and any shadow.
fn focus_pixels(screen: &Screen, theme: &Theme) {
    let button = screen.get_by_role(Role::Button, "Save");
    let rect = button.rect().expect("button geometry");
    let width = theme.focus_ring();
    assert!(width > 0.0);
    let x = (rect.x + rect.width / 2.0).floor() as u32;
    let y = (rect.y - (width / 2.0).ceil()).floor() as u32;
    let outside_y = (rect.y - width.ceil() - 2.0).floor() as u32;
    let before = screen.screenshot();
    let background = before.pixel(x, y).expect("ring margin is in the frame");
    screen.press_tab(false);
    assert!(button.is_focused(), "first Tab enters the standard button");
    let after = screen.screenshot();
    let painted = after.pixel(x, y).expect("ring pixel");
    let focus = theme.colors().focus.resolve(theme);
    let channel = |shift: u32| {
        let base = ((background >> shift) & 255u32) as f32;
        let color = match shift {
            16 => focus.r,
            8 => focus.g,
            _ => focus.b,
        };
        (color * 255.0 * focus.a + base * (1.0 - focus.a)).round() as u32
    };
    let expected = channel(16) << 16 | channel(8) << 8 | channel(0);
    assert!(
        distance(painted, expected) <= 8,
        "actual theme focus pixel {painted:06x}, expected {expected:06x}"
    );
    assert!(
        distance(painted, background) > 2,
        "keyboard focus paints a visible ring"
    );
    assert_eq!(
        before.pixel(x, outside_y),
        after.pixel(x, outside_y),
        "ring respects the actual theme width"
    );
}

type Handles = (
    Signal<i32>,
    Signal<bool>,
    Signal<Option<&'static str>>,
    Signal<&'static str>,
    Signal<bool>,
);

fn options(theme: Theme) -> RenderOptions {
    RenderOptions {
        theme,
        fonts: TestFonts::Only(vec![Arc::from(INTER)]),
        reduced_motion: true,
        width: 720.0,
        height: 960.0,
        ..RenderOptions::from_env()
    }
}

/// One standard component of every kind the kits restyle.
fn contract_view() -> (Handles, impl IntoView) {
    let count = signal(0);
    let checked = signal(true);
    let selected = signal(Some("a"));
    let tab = signal("a");
    let open = signal(false);
    (
        (count, checked, selected, tab, open),
        column((
            ui::button("Save").on_press(move || count.update(|n| *n += 1)),
            row((
                ui::checkbox("Check", checked),
                ui::switch("Switch", signal(false)),
                ui::switch("Locked", signal(true)).disabled(true),
            ))
            .style(style().gap(20)),
            ui::radio_group("Mode", choices(), signal(Some("a"))),
            ui::select("Choice", choices(), selected).style(style().width(280)),
            ui::tabs(
                "Sections",
                vec![ListItem::new("a", "First"), ListItem::new("c", "Second")],
                tab,
            )
            .content(|key| text(format!("Content {key}")))
            .style(style().width(280)),
            ui::text_input("Editor", signal("Initial".to_owned())).style(style().width(280)),
            ui::field(
                "Field",
                ui::text_input("Old name", signal("Name".to_owned())),
            )
            .hint("Helpful hint")
            .style(style().width(280)),
            ui::slider("Level", signal(0.5)).style(style().width(280)),
            ui::progress(Some(0.5))
                .name("Progress")
                .style(style().width(280)),
            ui::disclosure("Details", open, || text("Disclosed content")),
            ui::list("Items", choices(), signal(Some("a"))).style(style().width(280)),
        ))
        .style(
            style()
                .width(640)
                .padding(24)
                .gap(12)
                .align_items(Align::Start)
                .background(ColorToken::Surface),
        ),
    )
}

/// The standard components the contract view shows, by accessible role and
/// name.
fn components() -> [(Role, &'static str); 12] {
    [
        (Role::Button, "Save"),
        (Role::CheckBox, "Check"),
        (Role::Switch, "Switch"),
        (Role::RadioGroup, "Mode"),
        (Role::ComboBox, "Choice"),
        (Role::TabList, "Sections"),
        (Role::TextBox, "Editor"),
        (Role::TextBox, "Field"),
        (Role::Slider, "Level"),
        (Role::ProgressBar, "Progress"),
        (Role::Button, "Details"),
        (Role::ListBox, "Items"),
    ]
}

/// The components of [`components`] whose recipe each kit's own table sets,
/// mirroring `build()` in the kit's `recipes.rs`: Save is `button`, Check
/// `checkbox`, Switch `switch`, Mode `radio_group`, Choice `select`,
/// Sections `tabs`, Editor and Field `text_input` (Field also `field` where
/// the kit sets it), Level `slider`, Progress `progress`, Details
/// `disclosure` and Items `list`. Add a name here when a kit starts to set
/// the recipe.
const KIT_RECIPES: [(&str, &[&str]); 4] = [
    (
        "Basic",
        &[
            "Save", "Check", "Switch", "Mode", "Choice", "Sections", "Editor", "Field", "Level",
            "Progress", "Details", "Items",
        ],
    ),
    (
        "Game",
        &[
            "Save", "Check", "Switch", "Mode", "Choice", "Sections", "Editor", "Field", "Level",
            "Progress", "Details", "Items",
        ],
    ),
    (
        "Pop",
        &[
            "Save", "Check", "Switch", "Mode", "Choice", "Sections", "Editor", "Field", "Level",
            "Progress", "Details", "Items",
        ],
    ),
    (
        "Creative",
        &[
            "Save", "Check", "Switch", "Mode", "Choice", "Sections", "Editor", "Field", "Level",
            "Progress", "Details", "Items",
        ],
    ),
];

/// Whether any pixel of `rect` differs visibly between the two frames.
fn painted_differently(kit: &Screenshot, plain: &Screenshot, rect: kivact::draw::Rect) -> bool {
    let (x, y) = (rect.x.floor() as u32, rect.y.floor() as u32);
    let (width, height) = (rect.width.ceil() as u32 + 1, rect.height.ceil() as u32 + 1);
    let kit = kit.crop(x, y, width, height);
    let plain = plain.crop(x, y, width, height);
    kit.pixels
        .iter()
        .zip(&plain.pixels)
        .any(|(a, b)| distance(*a, *b) > 8)
}

/// `recipe` as a table entry again.
fn same<S: 'static>(
    recipe: Recipe<S>,
) -> impl Fn(&Theme, &Look<S>) -> Style + Send + Sync + 'static {
    move |theme, look| recipe(theme, look)
}

/// The kit's recipes with the recipes behind the component `name` replaced
/// by the library's defaults, all the others kept.
fn recipes_without(kit: &UiRecipes, name: &str) -> UiRecipes {
    let d = UiRecipes::default();
    let kit = kit.clone();
    match name {
        "Save" => kit.button(|_| same(d.button_recipe().clone())),
        "Check" => kit.checkbox(|_| same(d.checkbox_recipe().clone())),
        "Switch" => kit.switch(|_| same(d.switch_recipe().clone())),
        "Mode" => kit.radio_group(|_| d.radio_group_recipes().clone()),
        "Choice" => kit.select(|_| d.select_recipes().clone()),
        "Sections" => kit.tabs(|_| d.tabs_recipes().clone()),
        "Editor" => kit.text_input(|_| same(d.text_input_recipe().clone())),
        "Field" => kit
            .text_input(|_| same(d.text_input_recipe().clone()))
            .field(|_| d.field_recipes().clone()),
        "Level" => kit.slider(|_| d.slider_recipes().clone()),
        "Progress" => kit.progress(|_| d.progress_recipes().clone()),
        "Details" => kit.disclosure(|_| d.disclosure_recipes().clone()),
        "Items" => kit.list(|_| d.list_recipes().clone()),
        _ => unreachable!("no recipe behind {name}"),
    }
}

/// Mount the view with the kit's theme, then once per component the kit
/// styles with the same theme but that component's recipes replaced by the
/// library's defaults: the component must paint differently, so a recipe
/// the kit forgot to install is caught. (Replacing the whole table at once
/// would let layout shifts of other components hide a missing recipe.)
fn kit_recipes_paint(kit: &str, mode: ThemeMode, theme: &Theme, screen: &Screen) {
    let names = KIT_RECIPES
        .iter()
        .find(|(name, _)| *name == kit)
        .expect("every kit has a recipe table")
        .1;
    let kit_shot = screen.screenshot();
    let kit_recipes = theme.get::<UiRecipes>();
    for (role, name) in components() {
        if !names.contains(&name) {
            continue;
        }
        let rect = screen
            .get_by_role(role, name)
            .rect()
            .expect("visible recipe geometry");
        let plain_theme = theme.clone().with(recipes_without(&kit_recipes, name));
        let (plain, _) = render_with_handle(options(plain_theme), contract_view);
        assert!(
            painted_differently(&kit_shot, &plain.screenshot(), rect),
            "{kit} {mode:?} {name}: kit recipe not applied"
        );
    }
}

#[test]
fn all_kit_themes_preserve_standard_component_contracts() {
    let themes = [
        ("Basic", kivact_ui_basic::theme as fn(ThemeMode) -> Theme),
        ("Game", kivact_ui_game::theme),
        ("Pop", kivact_ui_pop::theme),
        ("Creative", kivact_ui_creative::theme),
    ];
    for (kit, theme) in themes {
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            let theme = theme(mode);
            let (screen, (count, checked, selected, tab, open)) =
                render_with_handle(options(theme.clone()), contract_view);
            kit_recipes_paint(kit, mode, &theme, &screen);
            focus_pixels(&screen, &theme);
            screen.press_key(Key::Enter);
            assert_eq!(count.get_untracked(), 1, "{kit} {mode:?}");
            let check = screen.get_by_role(Role::CheckBox, "Check");
            assert_eq!(check.is_checked(), Some(true));
            check.click();
            assert!(!checked.get_untracked());
            assert_eq!(
                screen.get_by_role(Role::Switch, "Switch").is_checked(),
                Some(false)
            );
            assert!(screen.get_by_role(Role::Switch, "Locked").is_disabled());
            assert_eq!(
                screen.get_by_role(Role::Switch, "Locked").is_checked(),
                Some(true)
            );
            assert_eq!(
                screen.get_by_role(Role::RadioButton, "Alpha").is_checked(),
                Some(true)
            );
            assert!(screen.get_by_role(Role::RadioButton, "Beta").is_disabled());
            let combo = screen.get_by_role(Role::ComboBox, "Choice");
            combo.click();
            screen
                .get_by_role(Role::ListBox, "Choice")
                .get_by_role(Role::Option, "Gamma")
                .click();
            assert_eq!(selected.get_untracked(), Some("c"));
            assert_eq!(combo.is_expanded(), Some(false));
            combo.click();
            screen.press_key(Key::Escape);
            assert_eq!(combo.is_expanded(), Some(false));
            assert!(combo.is_focused());
            let first = screen.get_by_role(Role::Tab, "First");
            let panel = screen.get_by_role(Role::TabPanel, "First");
            assert_eq!(
                first
                    .controls()
                    .iter()
                    .map(|e| e.name())
                    .collect::<Vec<_>>(),
                ["First"]
            );
            assert_eq!(
                panel
                    .labelled_by()
                    .iter()
                    .map(|e| e.name())
                    .collect::<Vec<_>>(),
                ["First"]
            );
            panel.get_by_text("Content a");
            first.press_key(Key::ArrowRight);
            assert_eq!(tab.get_untracked(), "c");
            let second = screen.get_by_role(Role::Tab, "Second");
            assert!(second.is_selected());
            assert_eq!(
                second
                    .controls()
                    .iter()
                    .map(|e| e.name())
                    .collect::<Vec<_>>(),
                ["Second"]
            );
            let panel = screen.get_by_role(Role::TabPanel, "Second");
            assert_eq!(
                panel
                    .labelled_by()
                    .iter()
                    .map(|e| e.name())
                    .collect::<Vec<_>>(),
                ["Second"]
            );
            panel.get_by_text("Content c");
            assert!(screen.query_by_text("Content a").is_none());
            assert_eq!(
                screen
                    .get_by_role(Role::TextBox, "Editor")
                    .value()
                    .as_deref(),
                Some("Initial")
            );
            let field = screen.get_by_role(Role::TextBox, "Field");
            assert_eq!(
                field
                    .described_by()
                    .iter()
                    .map(|e| e.text())
                    .collect::<Vec<_>>(),
                ["Helpful hint"]
            );
            assert_eq!(
                screen.get_by_role(Role::Slider, "Level").value_range().2,
                Some(0.5)
            );
            assert_eq!(
                screen
                    .get_by_role(Role::ProgressBar, "Progress")
                    .value_range()
                    .2,
                Some(0.5)
            );
            let disclosure = screen.get_by_role(Role::Button, "Details");
            assert_eq!(disclosure.is_expanded(), Some(false));
            disclosure.click();
            assert!(open.get_untracked());
            screen.get_by_text("Disclosed content");
            let list = screen.get_by_role(Role::ListBox, "Items");
            assert!(list.get_by_role(Role::Option, "Alpha").is_selected());
            for (role, name) in components() {
                let rect = screen
                    .get_by_role(role, name)
                    .rect()
                    .expect("visible recipe geometry");
                assert!(
                    rect.width > 0.0 && rect.height > 0.0,
                    "{kit} {mode:?} {name}"
                );
            }
            screen.assert_layout();
            assert!(
                screen.errors().is_empty(),
                "{kit} {mode:?}: {:?}",
                screen.errors()
            );
        }
    }
}
