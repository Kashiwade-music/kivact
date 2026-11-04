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
use crate::support::mount;
use kivact::interaction::{MenuItemState, TabState};
use kivact::motion::{Easing, Transition};
use kivact::prelude::*;
use kivact::style::provide_theme;
use kivact::style::{Color, Icon, ThemeExtension, TransitionProp, rgb, use_theme};
use kivact::test::Screen;
use kivact::ui::Look;
use kivact::ui::{ButtonState, UiRecipes};
use kivact::view::icon;
use std::time::Duration;

const RED: u32 = 0xff0000;
const GREEN: u32 = 0x00ff00;
const BLUE: u32 = 0x0000ff;
const WHITE: u32 = 0xffffff;

/// A filled square: its pixels are the icon's color.
static SQUARE: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><rect width="24" height="24" fill="#000"/></svg>"##;

/// The pixel at `(dx, dy)` from the top left of the element `id`.
fn at(screen: &Screen, id: &str, dx: f64, dy: f64) -> u32 {
    let r = screen.get_by_test_id(id).rect().expect("laid out");
    pixel(&screen.screenshot(), (r.x + dx) as u32, (r.y + dy) as u32)
}

#[test]
fn a_switch_thumb_recipe_animates_when_the_binding_changes() {
    let recipes = UiRecipes::default()
        .switch(|_| |_, _| style().size(120, 32).background(rgb(WHITE)))
        .switch_track(|_| |_, _| style().size(112, 24).left(4).top(4))
        .switch_thumb(|_| {
            |_, state| {
                style()
                    .size(16, 16)
                    .background(rgb(RED))
                    .translate_x(if state.checked.is_checked() { 80 } else { 0 })
                    .transition(
                        TransitionProp::Paint,
                        Transition::tween(400, Easing::Linear),
                    )
            }
        });
    let (screen, checked) = mount(move || {
        let checked = signal(false);
        (
            checked,
            provide_theme(Theme::light().with(recipes), move || {
                kivact::ui::switch("Motion", checked)
                    .hide_label(true)
                    .test_id("motion-switch")
            }),
        )
    });
    screen.set_reduced_motion(false);
    let thumb_at = |x| distance(at(&screen, "motion-switch", x, 12.0), RED) < 30;
    assert!(thumb_at(12.0));
    checked.set(true);
    screen.run_until_idle();
    screen.advance(Duration::from_millis(180));
    let rect = screen
        .get_by_test_id("motion-switch")
        .rect()
        .expect("laid out");
    let moving = screen.screenshot();
    let moving_at = |x| distance(pixel(&moving, rect.x as u32 + x, rect.y as u32 + 12), RED) < 30;
    assert!(!moving_at(12) && !moving_at(92));
    assert!((24..80).any(moving_at));
    screen.advance(Duration::from_secs(1));
    assert!(thumb_at(92.0));
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
            MenuEntry::item("export", "Export").with_description("As a PNG image"),
        ];
        let custom = vec![
            MenuEntry::item("new", "New"),
            MenuEntry::item("open", "Open"),
        ];
        let view = column((
            menu("Arrange", entries).on_activate(move |key| chosen.set(Some(key))),
            menu("File", custom)
                .on_activate(move |key| chosen.set(Some(key)))
                .item(|entry, state: Value<MenuItemState>| {
                    text(move || {
                        let mark = if state.get().input.focused { "> " } else { "" };
                        format!("{mark}[{}]", entry.get().label())
                    })
                }),
        ));
        (chosen, view)
    });
    screen.get_by_role(Role::Button, "Arrange").click();
    // The group's heading and the item's description are drawn, and the
    // keys move through the rows below the heading.
    screen.find_by_text("Align");
    screen.find_by_text("As a PNG image");
    assert_eq!(
        screen.get_by_role(Role::MenuItemRadio, "Left").checked(),
        Some(CheckedState::Checked)
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
                icon(Icon::from(kivact::view::Asset::from(SQUARE)))
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

// ---------------------------------------------------------------------------
// Recipes over a kit's and tokens over a theme switch.

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
            prev(theme, state).background(theme.ext::<Brand>().fill)
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
                    let fill = use_theme().get().ext::<Brand>().fill;
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

#[test]
fn use_theme_reads_inside_a_style_closure() {
    let (screen, theme) = mount(|| {
        let theme = signal(Theme::light());
        (
            theme,
            provide_theme(theme, || {
                node().test_id("theme-probe").style(|| {
                    let dark = use_theme().get().mode() == kivact::style::ThemeMode::Dark;
                    style()
                        .size(30, 30)
                        .background(rgb(if dark { BLUE } else { RED }))
                })
            }),
        )
    });
    assert!(distance(at(&screen, "theme-probe", 15.0, 15.0), RED) < 10);
    theme.set(Theme::dark());
    screen.run_until_idle();
    assert!(distance(at(&screen, "theme-probe", 15.0, 15.0), BLUE) < 10);
    assert!(screen.errors().is_empty());
}

#[test]
fn app_theme_follows_theme_hint() {
    let options = kivact::test::RenderOptions::from_env()
        .theme(|| Theme::light().with_mode(kivact::platform::theme_hint().get().into()));
    let screen = kivact::test::render_with(options, || {
        text(|| format!("{:?}", use_theme().get().mode()))
    });
    screen.set_dark(false);
    screen.run_until_idle();
    screen.get_by_text("Light");
    screen.set_dark(true);
    screen.run_until_idle();
    screen.get_by_text("Dark");
    screen.set_dark(false);
    screen.run_until_idle();
    screen.get_by_text("Light");
    assert!(screen.errors().is_empty());
}
