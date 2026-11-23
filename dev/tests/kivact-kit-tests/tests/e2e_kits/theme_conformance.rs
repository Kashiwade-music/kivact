//! Role: Check standard semantics, layout, recipes, focus paint and the
//! theme switch of the five kits.
use kivact::interaction::Tone;
use kivact::prelude::*;
use kivact::test::{RenderOptions, Screen, Screenshot, render_with_handle};
use kivact::ui::theme::{
    ButtonTheme, CheckboxTheme, DisclosureTheme, FieldTheme, ListTheme, ProgressTheme,
    RadioGroupTheme, SelectTheme, SliderTheme, SwitchTheme, TabsTheme, TextInputTheme,
};
use kivact::ui::{self, RadioGroupVariant};
use kivact::view::Orientation;
use kivact_public_api_tests::paint::distance;
use std::time::Duration;

static INTER: &[u8] = include_bytes!("../../../../../crates/arvane/tests/fonts/InterVariable.ttf");

/// A kit theme in a mode.
type KitTheme = fn(ThemeMode) -> Theme;

/// The five kit themes, by name.
fn kits() -> [(&'static str, KitTheme); 5] {
    [
        ("Basic", kivact_ui_basic::theme),
        ("Game", kivact_ui_game::theme),
        ("Pop", kivact_ui_pop::theme),
        ("Creative", kivact_ui_creative::theme),
        ("Soft", kivact_ui_soft::theme),
    ]
}

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
    let [red, green, blue, alpha] = theme.get(&color::FOCUS_VAR).components(theme);
    let channel = |shift: u32, color: f64| {
        let base = ((background >> shift) & 255u32) as f64;
        (color * 255.0 * alpha + base * (1.0 - alpha)).round() as u32
    };
    let expected = channel(16, red) << 16 | channel(8, green) << 8 | channel(0, blue);
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
        .theme(theme.with_fonts([INTER]))
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
            ui::select(choices(), selected)
                .label("Choice")
                .style(style().width(280)),
            ui::tabs(
                "Sections",
                vec![ListItem::new("a", "First"), ListItem::new("c", "Second")],
                tab,
            )
            .content(|key| text(format!("Content {key}")))
            .style(style().width(280)),
            ui::text_input(signal("Initial".to_owned()))
                .label("Editor")
                .style(style().width(280)),
            ui::field(
                "Field",
                ui::text_input(signal("Name".to_owned())).label("Old name"),
            )
            .hint("Helpful hint")
            .style(style().width(280)),
            ui::slider(signal(0.5))
                .label("Level")
                .style(style().width(280)),
            ui::progress(Some(0.5))
                .name("Progress")
                .style(style().width(280)),
            ui::disclosure("Details", open).content(|| text("Disclosed content")),
            ui::list("Items", choices(), signal(Some("a"))).style(style().width(280)),
        ))
        .style(
            style()
                .width(640)
                .padding(24)
                .gap(12)
                .align_items(Align::Start)
                .background(color::SURFACE),
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

/// The components of [`components`] whose recipe each kit sets, mirroring
/// each kit's theme: Save is `button`, Check `checkbox`, Switch
/// `switch`, Mode `radio_group`, Choice `select`, Sections `tabs`, Editor and
/// Field `text_input` (Field also `field`), Level `slider`, Progress
/// `progress`, Details `disclosure` and Items `list`. Add a name here when a
/// kit starts to set the recipe.
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

/// `theme` with the recipes of the component `name` replaced by the
/// library defaults (its `<Name>Theme::default()`), all the others kept.
fn without_recipe(theme: &Theme, name: &str) -> Theme {
    let theme = theme.clone();
    match name {
        "Save" => theme.with(ButtonTheme::default()),
        "Check" => theme.with(CheckboxTheme::default()),
        "Switch" => theme.with(SwitchTheme::default()),
        "Mode" => theme.with(RadioGroupTheme::default()),
        "Choice" => theme.with(SelectTheme::default()),
        "Sections" => theme.with(TabsTheme::default()),
        "Editor" => theme.with(TextInputTheme::default()),
        "Field" => theme
            .with(TextInputTheme::default())
            .with(FieldTheme::default()),
        "Level" => theme.with(SliderTheme::default()),
        "Progress" => theme.with(ProgressTheme::default()),
        "Details" => theme.with(DisclosureTheme::default()),
        "Items" => theme.with(ListTheme::default()),
        _ => unreachable!("no recipe behind {name}"),
    }
}

/// Mount the view with the kit theme, then once per component the kit
/// styles with the same theme but that component recipes replaced by the
/// library defaults: the component must paint differently, so a recipe
/// the kit forgot to install is caught. (Replacing the whole table at once
/// would let layout shifts of other components hide a missing recipe.)
fn kit_recipes_paint(kit: &str, mode: ThemeMode, theme: &Theme, screen: &Screen) {
    let names = KIT_RECIPES
        .iter()
        .find(|(name, _)| *name == kit)
        .expect("every kit has a recipe table")
        .1;
    let kit_shot = screen.screenshot();
    for (role, name) in components() {
        if !names.contains(&name) {
            continue;
        }
        let rect = screen
            .get_by_role(role, name)
            .rect()
            .expect("visible recipe geometry");
        let (plain, _) = render_with_handle(options(without_recipe(theme, name)), contract_view);
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
    for (kit, theme) in kits() {
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            let theme = theme(mode);
            let (screen, (count, checked, selected, tab, open)) =
                render_with_handle(options(theme.clone()), contract_view);
            kit_recipes_paint(kit, mode, &theme, &screen);
            focus_pixels(&screen, &theme);
            screen.press_key(Key::Enter);
            assert_eq!(count.get_untracked(), 1, "{kit} {mode:?}");
            check_theme_toggles(&screen, checked);
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
            check_theme_tabs(&screen, tab);
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
            check_theme_layout(&screen, kit, mode);
        }
    }
}

/// The standard components with state a user changes, in one view.
fn stateful_view(
    editor: Signal<String>,
    checked: Signal<bool>,
    tab: Signal<&'static str>,
    level: Signal<f64>,
) -> impl IntoView {
    column((
        ui::text_input(editor)
            .label("Editor")
            .style(style().width(280)),
        ui::checkbox("Check", checked),
        ui::tabs(
            "Sections",
            vec![ListItem::new("a", "First"), ListItem::new("c", "Second")],
            tab,
        )
        .content(|key| text(format!("Content {key}")))
        .style(style().width(280)),
        ui::slider(level).label("Level").style(style().width(280)),
    ))
    .style(
        style()
            .width(640)
            .padding(24)
            .gap(12)
            .align_items(Align::Start)
            .background(color::SURFACE),
    )
}

/// The kit theme switches from `start` to the other mode while the user has
/// changed the state of the components: the state stays and the pixels change.
fn restyle_on_switch_keeps_state(kit: &str, theme: KitTheme, start: ThemeMode) {
    let to_dark = start == ThemeMode::Light;
    let (screen, (dark, editor, checked, tab, level)) =
        render_with_handle(options(theme(start)), move || {
            let dark = signal(start == ThemeMode::Dark);
            let editor = signal("Initial".to_owned());
            let checked = signal(true);
            let tab = signal("a");
            let level = signal(0.5);
            (
                (dark, editor, checked, tab, level),
                kivact::style::provide_theme(
                    move || {
                        theme(if dark.get() {
                            ThemeMode::Dark
                        } else {
                            ThemeMode::Light
                        })
                    },
                    move || stateful_view(editor, checked, tab, level),
                ),
            )
        });
    let input = screen.get_by_role(Role::TextBox, "Editor");
    let before_shot = screen.screenshot();
    let rect = input.rect().expect("laid out");
    input.press_key(Key::End);
    input.type_text("!");
    screen.get_by_role(Role::CheckBox, "Check").click();
    screen
        .get_by_role(Role::Tab, "First")
        .press_key(Key::ArrowRight);
    screen
        .get_by_role(Role::Slider, "Level")
        .press_key(Key::ArrowRight);
    let state = || {
        (
            editor.get_untracked(),
            checked.get_untracked(),
            tab.get_untracked(),
            level.get_untracked(),
        )
    };
    let before = state();
    assert_eq!(before.0, "Initial!", "{kit} {start:?}: typed text");
    assert!(!before.1, "{kit} {start:?}: the check box was checked");
    assert_eq!(before.2, "c", "{kit} {start:?}: the tab was selected");
    assert!(before.3 > 0.5, "{kit} {start:?}: the slider moved");
    dark.set(to_dark);
    screen.run_until_idle();
    assert_eq!(state(), before, "{kit} {start:?}: the state is kept");
    let after_shot = screen.screenshot();
    assert!(
        painted_differently(&before_shot, &after_shot, rect),
        "{kit} {start:?}: the theme switch restyles the text input"
    );
    assert!(
        screen.errors().is_empty(),
        "{kit} {start:?}: {:?}",
        screen.errors()
    );
}

#[test]
fn all_kit_themes_restyle_on_switch_and_keep_component_state() {
    for (kit, theme) in kits() {
        for start in [ThemeMode::Light, ThemeMode::Dark] {
            restyle_on_switch_keeps_state(kit, theme, start);
        }
    }
}

#[test]
fn all_kit_themes_are_one_shared_theme_per_mode() {
    for (kit, theme) in kits() {
        let light = theme(ThemeMode::Light);
        assert!(
            light == theme(ThemeMode::Light),
            "{kit}: the same mode gives an equal theme"
        );
        assert!(light != theme(ThemeMode::Dark), "{kit}: the modes differ");
    }
    for mode in [ThemeMode::Light, ThemeMode::Dark] {
        assert!(
            kivact_ui_game::theme_with(mode, kivact_ui_game::ThemeOptions::default())
                == kivact_ui_game::theme(mode),
            "Game {mode:?}: the default options give the theme"
        );
        assert!(
            kivact_ui_creative::theme_with(mode, kivact_ui_creative::ThemeOptions::default())
                == kivact_ui_creative::theme(mode),
            "Creative {mode:?}: the default options give the theme"
        );
    }
}

/// A toast in `theme` is a live region, the pointer on it holds it, and once
/// the pointer has left it stays for the core default time and then goes.
fn toast_lifetime(kit: &str, theme: Theme) {
    let (screen, toaster) = render_with_handle(options(theme), || {
        let toaster = kivact::interaction::use_toaster();
        (toaster, kivact::ui::toast_region(toaster))
    });
    // The core default: the duration a new toast gets, read from the public API.
    let default = Toast::new("Default")
        .duration
        .expect("a new toast lasts the default time");
    toaster.push(
        Toast::new("Event has been created")
            .tone(Tone::Info)
            .body("Sunday, December 03, 2023 at 9:00 AM"),
    );
    screen.run_until_idle();
    screen.get_by_role(Role::Status, "Event has been created");
    screen
        .get_by_text("Sunday, December 03, 2023 at 9:00 AM")
        .hover();
    screen.advance(default + Duration::from_secs(1));
    assert!(
        screen
            .query_by_role(Role::Status, "Event has been created")
            .is_some(),
        "{kit}: held while the pointer is on it"
    );
    screen.mouse_leave();
    screen.advance(default - Duration::from_millis(1));
    assert!(
        screen
            .query_by_role(Role::Status, "Event has been created")
            .is_some(),
        "{kit}: stays until the default time has passed"
    );
    screen.advance(Duration::from_millis(1));
    assert!(
        screen
            .query_by_role(Role::Status, "Event has been created")
            .is_none(),
        "{kit}: gone after the default time"
    );
    assert!(screen.errors().is_empty(), "{kit}: {:?}", screen.errors());
}

#[test]
fn all_kit_themes_keep_the_toast_lifetime() {
    for (kit, theme) in kits() {
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            toast_lifetime(kit, theme(mode));
        }
    }
}

#[test]
fn creative_segments_survive_a_theme_change() {
    let (screen, (theme, selected)) =
        render_with_handle(options(kivact_ui_creative::theme(ThemeMode::Dark)), || {
            let theme = signal(kivact_ui_creative::theme(ThemeMode::Dark));
            let selected = signal(Some("first"));
            (
                (theme, selected),
                kivact::style::provide_theme(theme, move || {
                    ui::radio_group(
                        "Order",
                        vec![
                            ListItem::new("first", "First"),
                            ListItem::new("second", "Second"),
                        ],
                        selected,
                    )
                    .with_node(move |node| node.orientation(Orientation::Horizontal))
                    .variant(RadioGroupVariant::Segmented)
                }),
            )
        });
    screen
        .get_by_role(Role::RadioButton, "First")
        .press_key(Key::ArrowRight);
    let before = screen.snapshot();
    theme.set(kivact_ui_creative::theme_with(
        ThemeMode::Dark,
        kivact_ui_creative::ThemeOptions {
            accent: kivact_ui_creative::Accent::Aqua,
            density: kivact_ui_creative::Density::Compact,
        },
    ));
    screen.run_until_idle();
    assert_eq!(screen.snapshot(), before);
    screen
        .get_by_role(Role::RadioButton, "Second")
        .press_key(Key::ArrowRight);
    assert_eq!(
        selected.get_untracked(),
        Some("first"),
        "segments are a standard radio group, which wraps"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

fn check_theme_tabs(screen: &Screen, tab: Signal<&'static str>) {
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
}

fn check_theme_toggles(screen: &Screen, checked: Signal<bool>) {
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
}

fn check_theme_layout(screen: &Screen, kit: &str, mode: ThemeMode) {
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
