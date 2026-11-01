//! Role: Tests the state the standard components hand to their recipes.
//!
//! What a recipe can read on real windows: the size of a check box, a
//! switch, a button and a select, the box of a button's icons sized by the
//! button's size, whether a select's trigger shows its placeholder (its
//! list is at least as wide as the trigger), whether a text input is
//! (and a check box or switch) marked invalid, the orientation of a tab list, where a tooltip's arrow
//! points, and a button's icons and the state of the popup it opens.

use crate::support::mount_with;
use kivact::interaction::{TabListState, TabState};
use kivact::prelude::*;
use kivact::primitive::NodeExt;
use kivact::style::pct;
use kivact::style::{Align, Style, Theme};
use kivact::test::RenderOptions;
use kivact::ui::Look;
use kivact::ui::{
    ButtonState, CheckState, ControlSize, ListItem, SelectTriggerState, Side, TextInputState,
    TooltipArrowState, UiRecipes, button, checkbox, select, switch, tabs, text_input, tooltip,
};
use kivact::view::Orientation;

/// Render `f` with `recipes` installed in the light theme.
fn with_recipes<V: IntoView>(
    recipes: UiRecipes,
    f: impl FnOnce() -> V + 'static,
) -> kivact::test::Screen {
    let (screen, ()) = mount_with(
        RenderOptions {
            theme: Theme::light().with(recipes),
            ..RenderOptions::from_env()
        },
        move || ((), column(f()).style(style().align_items(Align::Start))),
    );
    screen
}

fn side(size: ControlSize) -> f32 {
    match size {
        ControlSize::ExtraSmall => 20.0,
        ControlSize::Small => 40.0,
        ControlSize::Medium => 60.0,
        ControlSize::Large => 80.0,
    }
}

#[test]
fn check_boxes_and_switches_hand_their_size_to_the_recipe() {
    fn sized_check(_: &Theme, state: &Look<CheckState>) -> Style {
        style().size(side(state.size), 20)
    }
    fn sized_switch(_: &Theme, state: &Look<CheckState>) -> Style {
        style().size(side(state.size), 20)
    }
    let screen = with_recipes(
        UiRecipes::default()
            .checkbox(|_| sized_check)
            .switch(|_| sized_switch),
        || {
            let on = signal(false);
            (
                checkbox("Small check", on).size(ControlSize::Small),
                checkbox("Check", on),
                switch("Large switch", on).size(ControlSize::Large),
                switch("Switch", on),
            )
        },
    );
    let width = |role, name| {
        screen
            .get_by_role(role, name)
            .rect()
            .expect("laid out")
            .width
    };
    assert_eq!(width(Role::CheckBox, "Small check"), 40.0);
    assert_eq!(width(Role::CheckBox, "Check"), 60.0);
    assert_eq!(width(Role::Switch, "Large switch"), 80.0);
    assert_eq!(width(Role::Switch, "Switch"), 60.0);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn the_default_look_draws_a_small_check_box_and_switch_smaller() {
    let screen = with_recipes(UiRecipes::default(), || {
        let on = signal(true);
        (
            checkbox("Small check", on).size(ControlSize::Small),
            checkbox("Check", on),
            switch("Small switch", on).size(ControlSize::Small),
            switch("Switch", on),
        )
    });
    let height = |role, name| {
        screen
            .get_by_role(role, name)
            .rect()
            .expect("laid out")
            .height
    };
    assert!(height(Role::CheckBox, "Small check") < height(Role::CheckBox, "Check"));
    assert!(height(Role::Switch, "Small switch") < height(Role::Switch, "Switch"));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn the_default_look_draws_each_button_size_lower_than_the_next() {
    let screen = with_recipes(UiRecipes::default(), || {
        (
            button("Extra small").size(ControlSize::ExtraSmall),
            button("Small").size(ControlSize::Small),
            button("Medium"),
            button("Large").size(ControlSize::Large),
            text_input("Extra small field", signal(String::new())).size(ControlSize::ExtraSmall),
            text_input("Small field", signal(String::new())).size(ControlSize::Small),
        )
    });
    let height = |role, name| {
        screen
            .get_by_role(role, name)
            .rect()
            .expect("laid out")
            .height
    };
    let buttons = ["Extra small", "Small", "Medium", "Large"].map(|n| height(Role::Button, n));
    assert!(
        buttons.windows(2).all(|pair| pair[0] < pair[1]),
        "button heights grow with the size: {buttons:?}"
    );
    assert!(
        height(Role::TextBox, "Extra small field") < height(Role::TextBox, "Small field"),
        "an extra small field is lower than a small one"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_button_hands_its_size_to_the_box_of_its_icons() {
    fn icon_box(_: &Theme, state: &Look<ButtonState>) -> Style {
        let side = side(state.size) / 4.0;
        style().size(side, side)
    }
    let screen = with_recipes(UiRecipes::default().button_icon(|_| icon_box), || {
        // An icon that fills the box it is given.
        let icon = |id: &'static str| node().style(style().size(pct(100), pct(100))).test_id(id);
        (
            button("Small").size(ControlSize::Small).icon(icon("small")),
            button("Medium")
                .icon(icon("lead"))
                .trailing_icon(icon("trail")),
        )
    });
    let width = |id: &str| screen.get_by_test_id(id).rect().expect("laid out").width;
    assert_eq!(width("small"), 10.0);
    assert_eq!(width("lead"), 15.0);
    assert_eq!(width("trail"), 15.0);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_select_trigger_knows_when_it_shows_the_placeholder_and_its_list_is_as_wide() {
    fn trigger(_: &Theme, state: &Look<SelectTriggerState>) -> Style {
        style().size(if state.placeholder { 200 } else { 300 }, 30)
    }
    let (screen, chosen) = mount_with(
        RenderOptions {
            theme: Theme::light()
                .with(UiRecipes::default().select(|table| table.trigger(|_| trigger))),
            ..RenderOptions::from_env()
        },
        || {
            let chosen = signal(None::<u32>);
            let options = vec![ListItem::new(1, "One"), ListItem::new(2, "Two")];
            let view = column(select("Number", options, chosen).placeholder("Pick"))
                .style(style().align_items(Align::Start));
            (chosen, view)
        },
    );
    let trigger = || screen.get_by_role(Role::ComboBox, "Number");
    let width = |e: kivact::test::Element<'_>| e.rect().expect("laid out").width;
    assert_eq!(width(trigger()), 200.0, "no value: the placeholder");
    chosen.set(Some(2));
    screen.run_until_idle();
    assert_eq!(width(trigger()), 300.0);
    trigger().click();
    let list = screen.get_by_role(Role::ListBox, "Number");
    assert!(width(list) >= 300.0, "the list is as wide as the trigger");
    screen.press_key(Key::Escape);
    chosen.set(Some(9));
    screen.run_until_idle();
    assert_eq!(width(trigger()), 200.0, "a key that is not an option");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_text_input_hands_its_invalid_flag_to_the_recipe() {
    fn field(_: &Theme, state: &Look<TextInputState>) -> Style {
        style().size(if state.invalid { 250 } else { 150 }, 30)
    }
    let (screen, bad) = mount_with(
        RenderOptions {
            theme: Theme::light().with(UiRecipes::default().text_input(|_| field)),
            ..RenderOptions::from_env()
        },
        || {
            let bad = signal(false);
            let value = signal(String::new());
            let view = column((
                text_input("Email", value).invalid(bad),
                text_input("Name", value),
            ))
            .style(style().align_items(Align::Start));
            (bad, view)
        },
    );
    let width = |name| {
        screen
            .get_by_role(Role::TextBox, name)
            .rect()
            .expect("laid out")
            .width
    };
    assert_eq!(width("Email"), 150.0);
    bad.set(true);
    screen.run_until_idle();
    assert_eq!(width("Email"), 250.0);
    assert_eq!(width("Name"), 150.0);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn tabs_hand_their_orientation_to_the_list_tab_and_panel_recipes() {
    fn list(_: &Theme, state: &Look<TabListState>) -> Style {
        if state.orientation == Orientation::Vertical {
            style()
                .direction(kivact::style::Direction::Column)
                .width(120)
        } else {
            style().direction(kivact::style::Direction::Row).height(30)
        }
    }
    fn tab(_: &Theme, state: &Look<TabState>) -> Style {
        if state.orientation == Orientation::Vertical {
            style().width(pct(100)).height(24)
        } else {
            style().width(70).height(pct(100))
        }
    }
    fn panel(_: &Theme, state: &Look<TabState>) -> Style {
        let wide = state.orientation == Orientation::Vertical;
        style().width(if wide { 200 } else { 100 }).height(40)
    }
    let screen = with_recipes(
        UiRecipes::default().tabs(|table| table.list(|_| list).tab(|_| tab).panel(|_| panel)),
        || {
            let options = vec![ListItem::new(1, "One"), ListItem::new(2, "Two")];
            let across = signal(1);
            let down = signal(1);
            column((
                tabs("Tabs", options.clone(), across)
                    .test_id("across")
                    .content(|key| text(format!("Across {key}"))),
                tabs("Tabs", options, down)
                    .orientation(Orientation::Vertical)
                    .test_id("down")
                    .content(|key| text(format!("Down {key}"))),
            ))
            .style(style().gap(10).align_items(Align::Start))
        },
    );
    let rect = |e: kivact::test::Element<'_>| e.rect().expect("laid out");
    let across = rect(screen.get_by_test_id("across"));
    let down = rect(screen.get_by_test_id("down"));
    assert_eq!(
        (across.width, across.height),
        (140.0, 30.0),
        "two tabs in a row"
    );
    assert_eq!(
        (down.width, down.height),
        (120.0, 48.0),
        "two full-width tabs in a column"
    );
    let tabs: Vec<_> = screen
        .all_by_role(Role::Tab, "Two")
        .into_iter()
        .map(rect)
        .collect();
    assert_eq!((tabs[0].width, tabs[0].height), (70.0, 30.0));
    assert_eq!((tabs[1].width, tabs[1].height), (120.0, 24.0));
    let panels: Vec<_> = screen
        .all_by_role(Role::TabPanel, "One")
        .into_iter()
        .map(rect)
        .collect();
    assert_eq!(panels[0].width, 100.0);
    assert!(
        panels[0].y >= across.y + across.height,
        "a horizontal list is above its panel"
    );
    assert_eq!(panels[1].width, 200.0);
    assert!(
        panels[1].x >= down.x + down.width,
        "a vertical list is beside its panel"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_select_hands_its_size_to_the_trigger_recipe() {
    fn trigger(_: &Theme, state: &Look<SelectTriggerState>) -> Style {
        style().size(120, side(state.size))
    }
    let screen = with_recipes(
        UiRecipes::default().select(|table| table.trigger(|_| trigger)),
        || {
            let options = vec![ListItem::new(1, "One")];
            (
                select("Small", options.clone(), signal(None::<u32>)).size(ControlSize::Small),
                select("Medium", options.clone(), signal(None::<u32>)),
                select("Large", options, signal(None::<u32>)).size(ControlSize::Large),
            )
        },
    );
    let height = |name| {
        screen
            .get_by_role(Role::ComboBox, name)
            .rect()
            .expect("laid out")
            .height
    };
    assert_eq!(height("Small"), 40.0);
    assert_eq!(height("Medium"), 60.0);
    assert_eq!(height("Large"), 80.0);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_tooltip_arrow_points_at_the_trigger_and_follows_a_flip() {
    fn surface(_: &Theme, _: &Look<kivact::interaction::PopupState>) -> Style {
        style()
            .size(100, 24)
            .background(kivact::style::Color::WHITE)
    }
    fn arrow(_: &Theme, state: &Look<TooltipArrowState>) -> Style {
        let base = style()
            .position(kivact::style::Position::Absolute)
            .size(10, 10)
            .left(state.offset - 5.0)
            .background(rgb(crate::paint::RED));
        match state.side {
            Side::Top => base.top(pct(100)),
            Side::Bottom => base.top(-10),
            Side::Left | Side::Right => base,
        }
    }
    let screen = with_recipes(
        UiRecipes::default().tooltip(|table| table.surface(|_| surface).arrow(|_| arrow)),
        || {
            column((
                // At the window's top edge: the tooltip asked above flips below.
                tooltip("Flipped").attach(button("High").style(style().size(160, 30))),
                tooltip("Above").attach(button("Low").style(style().size(160, 30))),
            ))
            .style(style().gap(200).padding_left(40))
        },
    );
    for (trigger, tip, above) in [("Low", "Above", true), ("High", "Flipped", false)] {
        let anchor = screen.get_by_role(Role::Button, trigger);
        anchor.focus();
        screen.advance(std::time::Duration::from_millis(500));
        let a = anchor.rect().expect("laid out");
        let s = screen
            .get_by_role(Role::Tooltip, tip)
            .rect()
            .expect("shown");
        assert_eq!(s.y + s.height <= a.y, above, "{tip}: {s:?} {a:?}");
        let y = if above {
            s.y + s.height + 2.0
        } else {
            s.y - 2.0
        };
        let shot = screen.screenshot();
        let centre = (a.x + a.width / 2.0) as u32;
        crate::paint::expect(&shot, centre, y as u32, crate::paint::RED, 8, tip);
        let away = crate::paint::pixel(&shot, (s.x + 2.0) as u32, y as u32);
        assert!(
            crate::paint::distance(away, crate::paint::RED) > 64,
            "{tip}: only the arrow under the trigger's centre"
        );
    }
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_button_knows_its_icons_and_whether_the_popup_it_opens_is_open() {
    fn sized(_: &Theme, state: &Look<ButtonState>) -> Style {
        let width = match state.expanded {
            None => 100,
            Some(false) => 140,
            Some(true) => 180,
        };
        let lead = if state.leading_icon { 10 } else { 0 };
        let trail = if state.trailing_icon { 20 } else { 0 };
        style().size(width + lead + trail, 30)
    }
    let (screen, open) = mount_with(
        RenderOptions {
            theme: Theme::light().with(UiRecipes::default().button(|_| sized)),
            ..RenderOptions::from_env()
        },
        || {
            let open = signal(false);
            let popup = kivact::interaction::use_popup(open);
            let view = column((
                button("Plain"),
                button("Lead").icon(node()),
                button("Trail").trailing_icon(node()),
                button("Opens").part(popup.trigger()),
            ))
            .style(style().align_items(Align::Start));
            (open, view)
        },
    );
    let width = |name| {
        screen
            .get_by_role(Role::Button, name)
            .rect()
            .expect("laid out")
            .width
    };
    assert_eq!(width("Plain"), 100.0);
    assert_eq!(width("Lead"), 110.0);
    assert_eq!(width("Trail"), 120.0);
    assert_eq!(width("Opens"), 140.0, "a closed popup trigger");
    open.set(true);
    screen.run_until_idle();
    assert_eq!(width("Opens"), 180.0, "an open popup trigger");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn check_boxes_and_switches_hand_their_invalid_flag_to_the_recipe() {
    fn check(_: &Theme, state: &Look<CheckState>) -> Style {
        style().size(if state.invalid { 90 } else { 50 }, 20)
    }
    fn toggle(_: &Theme, state: &Look<CheckState>) -> Style {
        style().size(if state.invalid { 90 } else { 50 }, 20)
    }
    let (screen, bad) = mount_with(
        RenderOptions {
            theme: Theme::light().with(UiRecipes::default().checkbox(|_| check).switch(|_| toggle)),
            ..RenderOptions::from_env()
        },
        || {
            let bad = signal(false);
            let on = signal(false);
            let view = column((
                checkbox("Terms", on).invalid(bad),
                switch("Alerts", on).invalid(bad),
                checkbox("Other", on),
            ))
            .style(style().align_items(Align::Start));
            (bad, view)
        },
    );
    let width = |role, name| {
        screen
            .get_by_role(role, name)
            .rect()
            .expect("laid out")
            .width
    };
    assert_eq!(width(Role::CheckBox, "Terms"), 50.0);
    assert!(!screen.get_by_role(Role::CheckBox, "Terms").invalid());
    bad.set(true);
    screen.run_until_idle();
    assert_eq!(width(Role::CheckBox, "Terms"), 90.0);
    assert_eq!(width(Role::Switch, "Alerts"), 90.0);
    assert_eq!(width(Role::CheckBox, "Other"), 50.0);
    assert!(
        screen.get_by_role(Role::CheckBox, "Terms").invalid(),
        "assistive technology sees the invalid value"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
