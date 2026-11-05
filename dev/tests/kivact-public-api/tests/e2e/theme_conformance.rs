//! Role: Check standard semantics, layout, recipes and focus paint in all kit themes.
use crate::paint::distance;
use kivact::interaction::ListItem;
use kivact::prelude::*;
use kivact::style::{Align, ColorToken, Style, ThemeMode};
use kivact::test::{RenderOptions, Screen, Screenshot, TestFonts, render_with_handle};
use kivact::ui::{self, Look, Recipe, UiRecipes};

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
    let offset = theme.focus_ring_offset();
    assert!(width > 0.0);
    let x = (rect.x + rect.width / 2.0).floor() as u32;
    let y = (rect.y - offset - (width / 2.0).ceil()).floor() as u32;
    let outside_y = (rect.y - offset - width.ceil() - 2.0).floor() as u32;
    let before = screen.screenshot();
    let background = before.pixel(x, y).expect("ring margin is in the frame");
    screen.press_tab(false);
    assert!(button.is_focused(), "first Tab enters the standard button");
    let after = screen.screenshot();
    let painted = after.pixel(x, y).expect("ring pixel");
    let focus = theme.colors().focus.resolve(theme);
    let channel = |shift: u32| {
        let base = ((background >> shift) & 255u32) as f64;
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
    if offset >= 2.0 {
        let gap_y = (rect.y - offset / 2.0).floor() as u32;
        let gap_before = before.pixel(x, gap_y).expect("gap is in the frame");
        let gap_after = after.pixel(x, gap_y).expect("gap is in the frame");
        assert!(
            distance(gap_before, gap_after) <= 2,
            "the gap between the ring and the button stays clear"
        );
    }
}

type Handles = (
    Signal<i32>,
    Signal<bool>,
    Signal<Option<&'static str>>,
    Signal<&'static str>,
    Signal<bool>,
);

fn options(theme: Theme) -> RenderOptions {
    RenderOptions::from_env()
        .window_size(720.0, 960.0)
        .theme(theme)
        .fonts(TestFonts::Only(vec![kivact::view::Asset::from(INTER)]))
        .reduced_motion()
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
            ui::disclosure("Details", open).content(|| text("Disclosed content")),
            ui::list("Items", choices())
                .selection(signal(Some("a")))
                .style(style().width(280)),
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
const KIT_RECIPES: [(&str, &[&str]); 5] = [
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
    (
        "Soft",
        &[
            "Save", "Check", "Switch", "Mode", "Choice", "Sections", "Editor", "Field", "Level",
            "Progress", "Details", "Items",
        ],
    ),
];

/// Whether any pixel of `rect` differs visibly between the two frames.
fn painted_differently(kit: &Screenshot, plain: &Screenshot, rect: kivact::geometry::Rect) -> bool {
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
    let kit_recipes = theme.ext::<UiRecipes>();
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
fn standard_button_respects_the_theme_focus_ring_offset() {
    let theme = Theme::light().with_focus_ring_offset(3.0);
    let (screen, _) = render_with_handle(options(theme.clone()), contract_view);
    focus_pixels(&screen, &theme);
}

#[test]
fn all_kit_themes_preserve_standard_component_contracts() {
    let themes = [
        ("Basic", kivact_ui_basic::theme as fn(ThemeMode) -> Theme),
        ("Game", kivact_ui_game::theme),
        ("Pop", kivact_ui_pop::theme),
        ("Creative", kivact_ui_creative::theme),
        ("Soft", kivact_ui_soft::theme),
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
            assert_eq!(check.checked(), Some(CheckedState::Checked));
            check.click();
            assert!(!checked.get_untracked());
            assert_eq!(
                screen.get_by_role(Role::Switch, "Switch").checked(),
                Some(CheckedState::Unchecked)
            );
            assert!(screen.get_by_role(Role::Switch, "Locked").is_disabled());
            assert_eq!(
                screen.get_by_role(Role::Switch, "Locked").checked(),
                Some(CheckedState::Checked)
            );
            assert_eq!(
                screen.get_by_role(Role::RadioButton, "Alpha").checked(),
                Some(CheckedState::Checked)
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

/// README coverage follows the public recipe getters, including unchanged defaults.
fn filled_recipe_keys(kit: &UiRecipes) -> Vec<&'static str> {
    let defaults = UiRecipes::default();
    macro_rules! changed {
        ($getter:ident) => {
            !std::ptr::eq(&**kit.$getter(), &**defaults.$getter())
        };
    }
    let mut keys: Vec<_> = [
        ("button", changed!(button_recipe)),
        ("button_content", changed!(button_content_recipe)),
        ("button_busy", changed!(button_busy_recipe)),
        ("button_icon", changed!(button_icon_recipe)),
        ("checkbox", changed!(checkbox_recipe)),
        ("checkbox_indicator", changed!(checkbox_indicator_recipe)),
        ("checkbox_check", changed!(checkbox_check_recipe)),
        ("checkbox_mixed", changed!(checkbox_mixed_recipe)),
        ("switch", changed!(switch_recipe)),
        ("switch_track", changed!(switch_track_recipe)),
        ("switch_thumb", changed!(switch_thumb_recipe)),
        ("toggle", changed!(toggle_recipe)),
        ("text_input", changed!(text_input_recipe)),
        ("text_area", changed!(text_area_recipe)),
        ("select", kit.select_recipes() != defaults.select_recipes()),
        ("menu", kit.menu_recipes() != defaults.menu_recipes()),
        ("dialog", kit.dialog_recipes() != defaults.dialog_recipes()),
        (
            "tooltip",
            kit.tooltip_recipes() != defaults.tooltip_recipes(),
        ),
        ("tabs", kit.tabs_recipes() != defaults.tabs_recipes()),
        (
            "radio_group",
            kit.radio_group_recipes() != defaults.radio_group_recipes(),
        ),
        ("slider", kit.slider_recipes() != defaults.slider_recipes()),
        ("toast", kit.toast_recipes() != defaults.toast_recipes()),
        ("field", kit.field_recipes() != defaults.field_recipes()),
        (
            "splitter",
            kit.splitter_recipes() != defaults.splitter_recipes(),
        ),
        ("list", kit.list_recipes() != defaults.list_recipes()),
        ("tree", kit.tree_recipes() != defaults.tree_recipes()),
        (
            "data_table",
            kit.data_table_recipes() != defaults.data_table_recipes(),
        ),
        ("split", kit.split_recipes() != defaults.split_recipes()),
        (
            "progress",
            kit.progress_recipes() != defaults.progress_recipes(),
        ),
        (
            "disclosure",
            kit.disclosure_recipes() != defaults.disclosure_recipes(),
        ),
        (
            "command_palette",
            kit.command_palette_recipes() != defaults.command_palette_recipes(),
        ),
        (
            "tag_input",
            kit.tag_input_recipes() != defaults.tag_input_recipes(),
        ),
        (
            "number_input",
            kit.number_input_recipes() != defaults.number_input_recipes(),
        ),
    ]
    .into_iter()
    .filter_map(|(key, filled)| filled.then_some(key))
    .collect();
    keys.extend(filled_extended_recipe_keys(kit, &defaults));
    keys.extend(filled_feedback_recipe_keys(kit, &defaults));
    keys.extend(filled_content_recipe_keys(kit, &defaults));
    keys.sort_unstable();
    keys
}

/// Tables added by the standard component migration use the same identity contract.
fn filled_extended_recipe_keys(kit: &UiRecipes, defaults: &UiRecipes) -> Vec<&'static str> {
    [
        (
            "calendar",
            kit.calendar_recipes() != defaults.calendar_recipes(),
        ),
        (
            "date_input",
            kit.date_input_recipes() != defaults.date_input_recipes(),
        ),
        (
            "date_range_input",
            kit.date_range_input_recipes() != defaults.date_range_input_recipes(),
        ),
        (
            "time_input",
            kit.time_input_recipes() != defaults.time_input_recipes(),
        ),
        (
            "date_time_input",
            kit.date_time_input_recipes() != defaults.date_time_input_recipes(),
        ),
        (
            "carousel",
            kit.carousel_recipes() != defaults.carousel_recipes(),
        ),
        (
            "dropzone",
            kit.dropzone_recipes() != defaults.dropzone_recipes(),
        ),
        (
            "hover_card",
            kit.hover_card_recipes() != defaults.hover_card_recipes(),
        ),
        (
            "combobox",
            kit.combobox_recipes() != defaults.combobox_recipes(),
        ),
        (
            "toolbar",
            kit.toolbar_recipes() != defaults.toolbar_recipes(),
        ),
        (
            "toggle_group",
            kit.toggle_group_recipes() != defaults.toggle_group_recipes(),
        ),
        (
            "button_group",
            kit.button_group_recipes() != defaults.button_group_recipes(),
        ),
    ]
    .into_iter()
    .filter_map(|(key, filled)| filled.then_some(key))
    .collect()
}

/// Theme tables for passive content, feedback and navigation.
fn filled_feedback_recipe_keys(kit: &UiRecipes, defaults: &UiRecipes) -> Vec<&'static str> {
    [
        (
            "empty_state",
            kit.empty_state_recipes() != defaults.empty_state_recipes(),
        ),
        ("alert", kit.alert_recipes() != defaults.alert_recipes()),
        ("card", kit.card_recipes() != defaults.card_recipes()),
        (
            "input_group",
            kit.input_group_recipes() != defaults.input_group_recipes(),
        ),
        ("table", kit.table_recipes() != defaults.table_recipes()),
        (
            "item_list",
            kit.item_list_recipes() != defaults.item_list_recipes(),
        ),
        (
            "pagination",
            kit.pagination_recipes() != defaults.pagination_recipes(),
        ),
        (
            "breadcrumb",
            kit.breadcrumb_recipes() != defaults.breadcrumb_recipes(),
        ),
        (
            "stepper",
            kit.stepper_recipes() != defaults.stepper_recipes(),
        ),
        (
            "navigation_menu",
            kit.navigation_menu_recipes() != defaults.navigation_menu_recipes(),
        ),
    ]
    .into_iter()
    .filter_map(|(key, filled)| filled.then_some(key))
    .collect()
}

/// Scalar indicator entries and the message and content tables share default identity.
fn filled_content_recipe_keys(kit: &UiRecipes, defaults: &UiRecipes) -> Vec<&'static str> {
    macro_rules! changed {
        ($getter:ident) => {
            !std::ptr::eq(&**kit.$getter(), &**defaults.$getter())
        };
    }
    [
        ("badge", changed!(badge_recipe)),
        ("avatar", changed!(avatar_recipe)),
        ("avatar_status", changed!(avatar_status_recipe)),
        ("avatar_group", changed!(avatar_group_recipe)),
        ("skeleton", changed!(skeleton_recipe)),
        ("spinner", changed!(spinner_recipe)),
        ("kbd", changed!(kbd_recipe)),
        ("heading", changed!(heading_recipe)),
        ("stat", kit.stat_recipes() != defaults.stat_recipes()),
        (
            "description_list",
            kit.description_list_recipes() != defaults.description_list_recipes(),
        ),
        (
            "chat_message",
            kit.chat_message_recipes() != defaults.chat_message_recipes(),
        ),
        (
            "chat_thread",
            kit.chat_thread_recipes() != defaults.chat_thread_recipes(),
        ),
        (
            "chat_composer",
            kit.chat_composer_recipes() != defaults.chat_composer_recipes(),
        ),
        (
            "typing_indicator",
            kit.typing_indicator_recipes() != defaults.typing_indicator_recipes(),
        ),
        (
            "code_block",
            kit.code_block_recipes() != defaults.code_block_recipes(),
        ),
    ]
    .into_iter()
    .filter_map(|(key, filled)| filled.then_some(key))
    .collect()
}

#[test]
fn kit_readme_lists_the_filled_recipes() {
    let kits: [(&str, fn(ThemeMode) -> Theme, &str); 5] = [
        (
            "basic",
            kivact_ui_basic::theme,
            include_str!("../../../../../crates/kivact-ui-basic/README.md"),
        ),
        (
            "game",
            kivact_ui_game::theme,
            include_str!("../../../../../crates/kivact-ui-game/README.md"),
        ),
        (
            "pop",
            kivact_ui_pop::theme,
            include_str!("../../../../../crates/kivact-ui-pop/README.md"),
        ),
        (
            "creative",
            kivact_ui_creative::theme,
            include_str!("../../../../../crates/kivact-ui-creative/README.md"),
        ),
        (
            "soft",
            kivact_ui_soft::theme,
            include_str!("../../../../../crates/kivact-ui-soft/README.md"),
        ),
    ];
    for (name, theme, readme) in kits {
        let section = readme
            .split_once("<!-- kivact-recipes -->")
            .expect("kit README starts its recipe list")
            .1
            .split_once("<!-- /kivact-recipes -->")
            .expect("kit README ends its recipe list")
            .0;
        let mut listed: Vec<_> = section
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| {
                line.strip_prefix("- `")
                    .and_then(|key| key.strip_suffix('`'))
                    .expect("recipe entries use a bullet and a backticked key")
            })
            .collect();
        listed.sort_unstable();
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            assert_eq!(
                listed,
                filled_recipe_keys(&theme(mode).ext::<UiRecipes>()),
                "{name} {mode:?} README recipe coverage"
            );
        }
    }
}
