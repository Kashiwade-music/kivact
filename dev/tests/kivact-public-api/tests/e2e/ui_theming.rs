//! Role: Tests the theme's parts of the standard components on real windows.
//!
//! What a theme gives the standard components and what an application
//! changes in it: the content slots of tabs, menus and selects and their
//! default content, the theme's icon set (resolution, a theme switch, the
//! diagnostic of a missing name, the size of an icon in a smaller box), recipes wrapped over a kit's, a library's
//! tokens re-read on a theme switch without a remount, the default focus
//! ring and disabled look of focusable elements and how a style overrides
//! them, and a kit's typed button variant.

use crate::paint::{distance, pixel};
use crate::support::{mount, mount_with};
use kivact::control::icon;
use kivact::interaction::{ListItemState, MenuItemState, TabState};
use kivact::prelude::*;
use kivact::style::{Color, Icon, IconSet, Style, ThemeExtension, ThemeMode, current_theme, rgb};
use kivact::test::{RenderOptions, Screen};
use kivact::ui::Look;
use kivact::ui::{ButtonState, UiRecipes};
use kivact::view::provide_theme;

const RED: u32 = 0xff0000;
const GREEN: u32 = 0x00ff00;
const BLUE: u32 = 0x0000ff;
const WHITE: u32 = 0xffffff;

/// A filled square: its pixels are the icon's color.
static SQUARE: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><rect width="24" height="24" fill="#000"/></svg>"##;

fn with_theme(theme: Theme) -> RenderOptions {
    RenderOptions {
        theme,
        reduced_motion: true,
        ..RenderOptions::from_env()
    }
}

/// The pixel at `(dx, dy)` from the top left of the element `id`.
fn at(screen: &Screen, id: &str, dx: f32, dy: f32) -> u32 {
    let r = screen.get_by_test_id(id).rect().expect("laid out");
    pixel(&screen.screenshot(), (r.x + dx) as u32, (r.y + dy) as u32)
}

// ---------------------------------------------------------------------------
// Slots and their default content.

#[test]
fn a_tab_slot_draws_the_callers_content_and_keeps_the_tabs_behavior() {
    let (screen, selected) = mount(|| {
        let selected = signal("inbox");
        let unread = signal(3u32);
        let view = tabs(
            "Tabs",
            vec![
                ListItem::new("inbox", "Inbox"),
                ListItem::new("sent", "Sent"),
            ],
            selected,
        )
        .item(move |item, state: Value<TabState>| {
            row((
                text(move || item.get().label()),
                text(move || {
                    let mark = if state.get().selected { "*" } else { "" };
                    match item.get().key() {
                        &"inbox" => format!("{mark}{} new", unread.get()),
                        _ => mark.to_owned(),
                    }
                }),
            ))
            .style(style().gap(6))
        })
        .name("Mail");
        (selected, view)
    });
    screen.find_by_text("*3 new");
    let inbox = screen.get_by_role(Role::Tab, "Inbox");
    inbox.focus();
    screen.press_key(Key::ArrowRight);
    assert_eq!(selected.get_untracked(), "sent");
    screen.find_by_text("3 new");
    assert!(screen.get_by_role(Role::Tab, "Sent").is_selected());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_radio_group_slot_draws_the_callers_content_and_keeps_the_radios_behavior() {
    use kivact::ui::CheckState;
    let (screen, selected) = mount(|| {
        let selected = signal(Some("week"));
        let view = radio_group(
            "Period",
            vec![
                ListItem::new("week", "Week"),
                ListItem::new("month", "Month").description("31"),
            ],
            selected,
        )
        .item(move |item, state: Value<CheckState>| {
            text(move || {
                let mark = if state.get().checked.is_checked() {
                    "*"
                } else {
                    ""
                };
                format!("{mark}{}", item.get().label())
            })
        });
        (selected, view)
    });
    screen.find_by_text("*Week");
    assert!(
        screen.query_by_text("31").is_none(),
        "the slot replaces the description too"
    );
    screen.get_by_role(Role::RadioButton, "Week").focus();
    screen.press_key(Key::ArrowDown);
    assert_eq!(selected.get_untracked(), Some("month"));
    screen.find_by_text("*Month");
    assert_eq!(
        screen.get_by_role(Role::RadioButton, "Month").is_checked(),
        Some(true)
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_menu_slot_replaces_item_rows_and_the_default_shows_headings_and_descriptions() {
    let (screen, chosen) = mount(|| {
        let chosen = signal(None::<&str>);
        let entries = vec![
            MenuEntry::group(
                "Align",
                vec![
                    MenuEntry::radio("left", "Left", true),
                    MenuEntry::radio("right", "Right", false),
                ],
            ),
            MenuEntry::separator(),
            MenuEntry::item("export", "Export").description("As a PNG image"),
        ];
        let custom = vec![
            MenuEntry::item("new", "New"),
            MenuEntry::item("open", "Open"),
        ];
        let view = column((
            menu("Arrange", entries, move |key| chosen.set(Some(key))),
            menu("File", custom, move |key| chosen.set(Some(key))).item(
                |entry, state: Value<MenuItemState>| {
                    text(move || {
                        let mark = if state.get().input.focused { "> " } else { "" };
                        format!("{mark}[{}]", entry.get().label())
                    })
                },
            ),
        ));
        (chosen, view)
    });
    screen.get_by_role(Role::Button, "Arrange").click();
    // The group's heading and the item's description are drawn, and the
    // keys move through the rows below the heading.
    screen.find_by_text("Align");
    screen.find_by_text("As a PNG image");
    assert_eq!(
        screen.get_by_role(Role::MenuItemRadio, "Left").is_checked(),
        Some(true)
    );
    screen.press_key(Key::Escape);
    screen.get_by_role(Role::Button, "File").click();
    screen.find_by_text("> [New]");
    screen.press_key(Key::ArrowDown);
    screen.find_by_text("> [Open]");
    screen.press_key(Key::Enter);
    assert_eq!(chosen.get_untracked(), Some("open"));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_select_option_slot_draws_each_row_and_selection_still_works() {
    let (screen, chosen) = mount(|| {
        let chosen = signal(None::<&str>);
        let options = vec![
            ListItem::new("apple", "Apple"),
            ListItem::new("pear", "Pear"),
        ];
        let view = select("Fruit", options, chosen).option(|item, state: Value<ListItemState>| {
            text(move || {
                let mark = if state.get().selected { "* " } else { "" };
                format!("{mark}<{}>", item.get().label())
            })
        });
        (chosen, view)
    });
    screen.get_by_role(Role::ComboBox, "Fruit").click();
    screen.find_by_text("<Pear>").click();
    assert_eq!(chosen.get_untracked(), Some("pear"));
    screen.get_by_role(Role::ComboBox, "Fruit").click();
    screen.find_by_text("* <Pear>");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

// ---------------------------------------------------------------------------
// The icon set.

#[test]
fn a_named_icon_draws_the_themes_image_and_follows_a_theme_switch() {
    let with_square = Theme::light().with_icon("brand", SQUARE);
    let without = Theme::light();
    let (screen, theme) = mount(move || {
        let theme = signal(with_square.clone());
        let view = provide_theme(theme, || {
            column((
                icon(Icon::named("brand"))
                    .test_id("brand")
                    .style(style().color(rgb(RED))),
                // An image icon draws its own asset under any theme.
                icon(Icon::from(kivact::Asset::from(SQUARE)))
                    .test_id("image")
                    .style(style().color(rgb(BLUE))),
            ))
            .style(style().padding(8).gap(8))
        });
        (theme, view)
    });
    assert!(distance(at(&screen, "brand", 8.0, 8.0), RED) <= 8);
    assert!(distance(at(&screen, "image", 8.0, 8.0), BLUE) <= 8);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    // A theme whose set lacks the name draws nothing and reports it once.
    theme.set(without);
    screen.run_until_idle();
    assert!(distance(at(&screen, "brand", 8.0, 8.0), WHITE) <= 8);
    assert!(distance(at(&screen, "image", 8.0, 8.0), BLUE) <= 8);
    let errors = screen.errors();
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(
        format!("{:?}", errors[0]).contains("\"brand\" is not in the theme's icon set"),
        "{errors:?}"
    );
}

#[test]
fn an_icon_is_16_px_and_shrinks_to_fit_a_smaller_box() {
    let (screen, ()) = mount(|| {
        let view = row((
            icon(Icon::from(kivact::Asset::from(SQUARE))).test_id("free"),
            node()
                .style(style().size(12, 12))
                .child(icon(Icon::from(kivact::Asset::from(SQUARE))).test_id("boxed")),
            icon(Icon::from(kivact::Asset::from(SQUARE)))
                .test_id("sized")
                .style(style().size(24, 24)),
        ))
        .style(
            style()
                .padding(8)
                .gap(8)
                .align_items(kivact::style::Align::Start),
        );
        ((), view)
    });
    let size = |id: &str| {
        let r = screen.get_by_test_id(id).rect().expect("laid out");
        (r.width, r.height)
    };
    assert_eq!(size("free"), (16.0, 16.0));
    assert_eq!(size("boxed"), (12.0, 12.0), "fits its 12 px box");
    assert_eq!(size("sized"), (24.0, 24.0), "a style sets its size");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn the_standard_marks_come_from_the_themes_icon_set() {
    // A theme that draws every mark as a filled square.
    let theme = Theme::light().with_icons(
        IconSet::new()
            .with("check", SQUARE)
            .with("dot", SQUARE)
            .with("menu-radio", SQUARE)
            .with("chevron-down", SQUARE)
            .with("chevron-right", SQUARE)
            .with("x", SQUARE),
    );
    let (screen, ()) = mount_with(with_theme(theme), || {
        let entries = vec![
            MenuEntry::item("grid", "Show grid").checked(true),
            MenuEntry::radio("snap", "Snap", true),
        ];
        (
            (),
            column((
                menu("View", entries, |_: &str| ()),
                radio_group("Choice", vec![ListItem::new("a", "A")], signal(Some("a")))
                    .test_id("radio"),
            ))
            .style(
                style()
                    .padding(8)
                    .gap(8)
                    .align_items(kivact::style::Align::Start),
            ),
        )
    });
    // The chosen radio item's indicator holds the theme's `dot`: a square
    // in the accent fills the indicator's middle.
    let item = screen.get_by_role(Role::RadioButton, "A");
    let indicator = item.children()[0].rect().expect("laid out");
    let middle = pixel(
        &screen.screenshot(),
        (indicator.x + indicator.width / 2.0) as u32,
        (indicator.y + indicator.height / 2.0) as u32,
    );
    assert!(distance(middle, 0x2563eb) <= 16, "{middle:06x}");
    screen.get_by_role(Role::Button, "View").click();
    let check = screen
        .get_by_role(Role::MenuItemCheckBox, "Show grid")
        .children()[0]
        .rect()
        .expect("laid out");
    let mark = pixel(
        &screen.screenshot(),
        (check.x + 8.0) as u32,
        (check.y + check.height / 2.0) as u32,
    );
    assert!(
        distance(mark, 0x111827) <= 16,
        "the check is a square: {mark:06x}"
    );
    // A chosen radio item of a menu holds the theme's `menu-radio`.
    let radio = screen.get_by_role(Role::MenuItemRadio, "Snap").children()[0]
        .rect()
        .expect("laid out");
    let mark = pixel(
        &screen.screenshot(),
        (radio.x + 2.0) as u32,
        (radio.y + radio.height / 2.0) as u32,
    );
    assert!(
        distance(mark, 0x111827) <= 16,
        "the radio mark is a square: {mark:06x}"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

// ---------------------------------------------------------------------------
// Recipes over a kit's and tokens over a theme switch.

#[test]
fn an_application_wraps_one_recipe_of_a_kit_and_keeps_the_others() {
    let kit = kivact_ui_pop::theme(ThemeMode::Light);
    let recipes = UiRecipes::clone(&kit.get::<UiRecipes>());
    let red_buttons = recipes.button(|prev| {
        move |theme: &Theme, state: &Look<ButtonState>| prev(theme, state).background(rgb(RED))
    });
    let changed = kit.clone().with(red_buttons);
    let draw = |theme: Theme| {
        let (screen, ()) = mount_with(with_theme(theme), || {
            (
                (),
                column((
                    button("Go").test_id("go"),
                    checkbox("Ready", signal(true)).test_id("ready"),
                ))
                .style(
                    style()
                        .padding(8)
                        .gap(8)
                        .align_items(kivact::style::Align::Start),
                ),
            )
        });
        screen
    };
    let (plain, wrapped) = (draw(kit), draw(changed));
    assert!(distance(at(&wrapped, "go", 4.0, 12.0), RED) <= 8);
    assert!(distance(at(&plain, "go", 4.0, 12.0), RED) > 64);
    // The wrapped recipe keeps the kit's size, and the check box keeps the
    // kit's recipe.
    let size = |screen: &Screen| screen.get_by_test_id("go").rect().expect("laid out");
    assert_eq!(size(&plain).height, size(&wrapped).height);
    for dx in [4.0, 8.0, 12.0] {
        assert_eq!(
            at(&plain, "ready", dx, 10.0),
            at(&wrapped, "ready", dx, 10.0)
        );
    }
    assert!(wrapped.errors().is_empty(), "{:?}", wrapped.errors());
}

#[derive(Clone, Debug, PartialEq)]
struct Brand {
    fill: Color,
}

impl ThemeExtension for Brand {
    fn fallback(_: &Theme) -> Self {
        Brand { fill: rgb(GREEN) }
    }
}

#[test]
fn a_librarys_tokens_are_read_again_on_a_theme_switch_without_a_remount() {
    let red = Theme::light().with(Brand { fill: rgb(RED) });
    let blue = Theme::light().with(Brand { fill: rgb(BLUE) });
    // A recipe that reads the tokens, as a kit's does.
    let recipes = UiRecipes::default().button(|prev| {
        move |theme: &Theme, state: &Look<ButtonState>| {
            prev(theme, state).background(theme.get::<Brand>().fill)
        }
    });
    let (red, blue) = (red.with(recipes.clone()), blue.with(recipes));
    let (screen, (theme, setups)) = mount(move || {
        let theme = signal(red.clone());
        let setups = signal(0);
        let view = provide_theme(theme, move || {
            setups.update(|n| *n += 1);
            column((
                button("Go").test_id("go"),
                node().test_id("box").style(move || {
                    let fill = current_theme().get::<Brand>().fill;
                    style().size(20, 20).background(fill)
                }),
            ))
            .style(
                style()
                    .padding(8)
                    .gap(8)
                    .align_items(kivact::style::Align::Start),
            )
        });
        ((theme, setups), view)
    });
    assert!(distance(at(&screen, "go", 4.0, 12.0), RED) <= 8);
    assert!(distance(at(&screen, "box", 10.0, 10.0), RED) <= 8);
    theme.set(blue);
    screen.run_until_idle();
    assert!(distance(at(&screen, "go", 4.0, 12.0), BLUE) <= 8);
    assert!(distance(at(&screen, "box", 10.0, 10.0), BLUE) <= 8);
    // A theme without the tokens reads their fallback.
    theme.set(Theme::light());
    screen.run_until_idle();
    assert!(distance(at(&screen, "box", 10.0, 10.0), GREEN) <= 8);
    assert_eq!(
        setups.get_untracked(),
        1,
        "the switch restyles, it does not remount"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

// ---------------------------------------------------------------------------
// The default focus ring and disabled look.

/// A focusable 40 by 20 button on white, styled by `extra`.
fn stop(id: &'static str, extra: Style) -> Node {
    node()
        .role(Role::Button)
        .name(id)
        .focusable(true)
        .test_id(id)
        .style(style().size(40, 20).background(rgb(WHITE)).merge(&extra))
}

#[test]
fn a_focusable_element_rings_in_the_focus_token_only_while_the_focus_is_visible() {
    let theme = Theme::light();
    let colors = kivact::style::theme::Palette {
        focus: rgb(RED),
        ..theme.colors().clone()
    };
    let theme = theme.with_colors(colors).with_focus_ring(3.0);
    let (screen, ()) = mount_with(with_theme(theme), || {
        (
            (),
            column((
                stop("plain", style()),
                stop("none", style().focus_visible(style().ring_none())),
                stop("own", style().focus_visible(style().ring(3, rgb(BLUE)))),
                // A standard component's recipe sets no focus look either.
                button("Standard").test_id("standard"),
            ))
            .style(style().padding(16).gap(16)),
        )
    });
    let outside = |screen: &Screen, id: &str| at(screen, id, 20.0, -2.0);
    // A press focuses without a visible focus: no ring.
    screen.get_by_test_id("plain").click();
    assert!(screen.get_by_test_id("plain").is_focused());
    assert!(distance(outside(&screen, "plain"), WHITE) <= 8);
    // Tab moves the focus visibly: the theme's ring, then none, then the
    // style's own.
    screen.press_tab(false);
    assert!(screen.get_by_test_id("none").is_focused());
    assert!(distance(outside(&screen, "none"), WHITE) <= 8);
    screen.press_tab(true);
    assert!(distance(outside(&screen, "plain"), RED) <= 8);
    screen.press_tab(false);
    screen.press_tab(false);
    assert!(screen.get_by_test_id("own").is_focused());
    assert!(distance(outside(&screen, "own"), BLUE) <= 8);
    screen.press_tab(false);
    assert!(screen.get_by_test_id("standard").is_focused());
    assert!(distance(outside(&screen, "standard"), RED) <= 8);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_disabled_focusable_element_is_dimmed_unresponsive_and_out_of_the_tab_order() {
    let theme = Theme::light().with_disabled_opacity(0.5);
    let (screen, ()) = mount_with(with_theme(theme), || {
        let red = style()
            .background(rgb(RED))
            .hover(style().background(rgb(BLUE)));
        (
            (),
            column((
                stop("before", style()),
                stop("dim", red.clone()).disabled(true),
                stop("opaque", red.clone().disabled(style().opacity(1.0))).disabled(true),
                stop("enabled", red),
            ))
            .style(style().padding(16).gap(16)),
        )
    });
    // Half red over white, and opaque red where the style says so.
    assert!(distance(at(&screen, "dim", 20.0, 10.0), 0xff8080) <= 12);
    assert!(distance(at(&screen, "opaque", 20.0, 10.0), RED) <= 8);
    // Its semantics say disabled, and the hover look does not apply.
    assert!(screen.get_by_test_id("dim").is_disabled());
    screen.get_by_test_id("opaque").hover();
    assert!(distance(at(&screen, "opaque", 20.0, 10.0), RED) <= 8);
    screen.get_by_test_id("enabled").hover();
    assert!(distance(at(&screen, "enabled", 20.0, 10.0), BLUE) <= 8);
    // Tab skips both disabled elements.
    screen.get_by_test_id("before").focus();
    screen.press_tab(false);
    assert!(screen.get_by_test_id("enabled").is_focused());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_disabled_roving_item_is_dimmed_like_a_disabled_focus_stop() {
    // An item of a roving list is a focus stop only while it holds the
    // roving focus; a disabled one never does, and still takes the theme's
    // disabled opacity without a disabled state style of its own.
    use kivact::interaction::{ListSelection, use_list};
    let theme = Theme::light().with_disabled_opacity(0.5);
    let (screen, ()) = mount_with(with_theme(theme), || {
        let items = vec![
            ListItem::new("on", "On"),
            ListItem::new("off", "Off").disabled(true),
        ];
        let chosen = signal(Some("on"));
        let list = use_list(items.clone(), ListSelection::Single(chosen.into()));
        let rows = items.into_iter().map(|item| {
            let key = *item.key();
            node()
                .part(list.item(key))
                .test_id(key)
                .style(style().size(40, 20).background(rgb(RED)))
        });
        (
            (),
            node()
                .part(list.list())
                .child(rows.collect::<Vec<_>>())
                .style(style().padding(16).gap(16)),
        )
    });
    assert!(screen.get_by_test_id("off").is_disabled());
    assert!(distance(at(&screen, "on", 20.0, 10.0), RED) <= 8);
    assert!(distance(at(&screen, "off", 20.0, 10.0), 0xff8080) <= 12);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

// ---------------------------------------------------------------------------
// A kit's typed button variant.

#[derive(Clone, Copy, Debug, PartialEq)]
enum Tone {
    Green,
}

impl kivact::ui::Variant for Tone {}

#[test]
fn a_kits_variant_reaches_its_recipe_and_other_themes_ignore_it() {
    let recipes = UiRecipes::default().button(|prev| {
        move |theme: &Theme, state: &Look<ButtonState>| match state.variant::<Tone>() {
            Some(Tone::Green) => prev(theme, state).background(rgb(GREEN)),
            None => prev(theme, state),
        }
    });
    let content = || {
        (
            (),
            column((
                button("Go").variant(Tone::Green).test_id("toned"),
                button("Stop").test_id("plain"),
            ))
            .style(
                style()
                    .padding(8)
                    .gap(8)
                    .align_items(kivact::style::Align::Start),
            ),
        )
    };
    let (kit, ()) = mount_with(with_theme(Theme::light().with(recipes)), content);
    assert!(distance(at(&kit, "toned", 4.0, 12.0), GREEN) <= 8);
    assert!(distance(at(&kit, "plain", 4.0, 12.0), 0x2563eb) <= 16);
    // The core's recipe does not know the type: the plain Primary look.
    let (core, ()) = mount_with(with_theme(Theme::light()), content);
    assert_eq!(at(&core, "toned", 4.0, 12.0), at(&core, "plain", 4.0, 12.0));
    // The pop kit's own tones are such an extension.
    let (pop, ()) = mount_with(with_theme(kivact_ui_pop::theme(ThemeMode::Light)), || {
        (
            (),
            column((
                button("Cheer")
                    .variant(kivact_ui_pop::ButtonTone::Pink)
                    .test_id("pink"),
                button("Go").test_id("blue"),
            ))
            .style(
                style()
                    .padding(8)
                    .gap(8)
                    .align_items(kivact::style::Align::Start),
            ),
        )
    });
    assert!(distance(at(&pop, "pink", 4.0, 18.0), 0xf2559a) <= 24);
    assert!(distance(at(&pop, "blue", 4.0, 18.0), 0x2b8bd8) <= 24);
    assert_eq!(
        pop.get_by_role(Role::Button, "Cheer").a11y_actions(),
        pop.get_by_role(Role::Button, "Go").a11y_actions()
    );
    assert!(kit.errors().is_empty() && core.errors().is_empty() && pop.errors().is_empty());
}

#[test]
fn a_button_takes_its_button_variant_and_a_kits_variant_through_one_method() {
    let recipes = UiRecipes::default().button(|prev| {
        move |theme: &Theme, state: &Look<ButtonState>| match state.variant::<Tone>() {
            Some(Tone::Green) => prev(theme, state).background(rgb(GREEN)),
            None => prev(theme, state),
        }
    });
    let (screen, ()) = mount_with(with_theme(Theme::light().with(recipes)), || {
        (
            (),
            column((
                button("a").variant(ButtonVariant::Danger).test_id("danger"),
                // A later value of the same type replaces the earlier one.
                button("b")
                    .variant(ButtonVariant::Danger)
                    .variant(ButtonVariant::Ghost)
                    .test_id("replaced"),
                button("c").variant(ButtonVariant::Ghost).test_id("ghost"),
                // The two types are kept side by side, in either order.
                button("d")
                    .variant(Tone::Green)
                    .variant(ButtonVariant::Danger)
                    .test_id("both"),
                button("e").test_id("plain"),
            ))
            .style(
                style()
                    .padding(8)
                    .gap(8)
                    .align_items(kivact::style::Align::Start),
            ),
        )
    });
    assert!(
        distance(
            at(&screen, "danger", 4.0, 12.0),
            at(&screen, "ghost", 4.0, 12.0)
        ) > 16
    );
    assert!(
        distance(
            at(&screen, "danger", 4.0, 12.0),
            at(&screen, "plain", 4.0, 12.0)
        ) > 16
    );
    assert_eq!(
        at(&screen, "replaced", 4.0, 12.0),
        at(&screen, "ghost", 4.0, 12.0)
    );
    assert!(distance(at(&screen, "both", 4.0, 12.0), GREEN) <= 8);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// A kit's segmented look of a radio group, beside the plain one.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Segmented;

impl kivact::ui::Variant for Segmented {}

#[test]
fn one_theme_draws_a_plain_and_a_segmented_radio_group_by_their_variant() {
    use kivact::ui::{CheckState, ListItem, radio_group};
    use kivact::view::Orientation;
    let recipes = UiRecipes::default().radio_group(|table| {
        table
            .group(|prev| {
                move |theme: &Theme, look: &Look<Orientation>| match look.variant::<Segmented>() {
                    Some(Segmented) => prev(theme, look).gap(0).background(rgb(GREEN)),
                    None => prev(theme, look),
                }
            })
            .item(|prev| {
                move |theme: &Theme, look: &Look<CheckState>| match look.variant::<Segmented>() {
                    Some(Segmented) => {
                        style()
                            .size(80, 30)
                            .background(if look.checked.is_checked() {
                                rgb(BLUE)
                            } else {
                                rgb(GREEN)
                            })
                    }
                    None => prev(theme, look),
                }
            })
            .indicator(|prev| {
                move |theme: &Theme, look: &Look<CheckState>| match look.variant::<Segmented>() {
                    Some(Segmented) => style().size(0, 0),
                    None => prev(theme, look),
                }
            })
    });
    let options = || {
        vec![
            ListItem::new("day", "Day"),
            ListItem::new("week", "Week"),
            ListItem::new("month", "Month"),
        ]
    };
    let (screen, (plain, segmented)) =
        mount_with(with_theme(Theme::light().with(recipes)), move || {
            let (plain, segmented) = (signal(Some("day")), signal(Some("day")));
            (
                (plain, segmented),
                column((
                    radio_group("Plain", options(), plain),
                    radio_group("Segmented", options(), segmented)
                        .orientation(Orientation::Horizontal)
                        .variant(Segmented)
                        .test_id("segmented"),
                ))
                .style(
                    style()
                        .padding(8)
                        .gap(8)
                        .align_items(kivact::style::Align::Start),
                ),
            )
        });
    let plain_group = screen.within(screen.get_by_role(Role::RadioGroup, "Plain"));
    let segmented_group = screen.within(screen.get_by_role(Role::RadioGroup, "Segmented"));
    let rect = |e: kivact::test::Element<'_>| e.rect().expect("laid out");
    // The segmented items are the kit's 80 by 30 segments in a row; the
    // plain ones keep the default look.
    let week = rect(segmented_group.get_by_role(Role::RadioButton, "Week"));
    let day = rect(segmented_group.get_by_role(Role::RadioButton, "Day"));
    assert_eq!((week.width, week.height), (80.0, 30.0));
    assert_eq!(week.y, day.y, "a row of segments");
    assert_ne!(
        rect(plain_group.get_by_role(Role::RadioButton, "Week")).width,
        80.0
    );
    let probe = |r: kivact::draw::Rect| {
        pixel(
            &screen.screenshot(),
            (r.x + 40.0) as u32,
            (r.y + 15.0) as u32,
        )
    };
    assert!(distance(probe(day), BLUE) <= 8);
    assert!(distance(probe(week), GREEN) <= 8);
    // The behavior is the same: the arrow chooses the next segment.
    segmented_group
        .get_by_role(Role::RadioButton, "Day")
        .focus();
    screen.press_key(Key::ArrowRight);
    assert_eq!(segmented.get_untracked(), Some("week"));
    assert_eq!(plain.get_untracked(), Some("day"));
    assert!(distance(probe(week), BLUE) <= 8);
    assert!(distance(probe(day), GREEN) <= 8);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
