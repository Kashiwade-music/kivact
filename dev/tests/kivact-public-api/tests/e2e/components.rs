//! Role: These tests verify component builders and shared modifiers.
//!
//! Component builders that hold a root
//! `Node` get the standard modifiers through `Element`, writer-less values
//! bind as cheaply as constants, and the roles added for the official components
//! reach the screen.

use kivact::view::View;
use std::cell::Cell;
use std::rc::Rc;

use crate::support::mount;
use kivact::draw::{Draw, RegionId};
use kivact::geometry::Size;
use kivact::prelude::*;
use kivact::style::Align;
use kivact::test::render;
use kivact::view::A11yAction;

// ---- Element -------------------------------------------------------------

/// An external-style component: a builder struct around its root `Node`.
struct Card {
    root: Node,
}

#[track_caller]
fn card(title: impl IntoText) -> impl Element {
    let title = title.into_text_value();
    Card {
        root: node()
            .role(Role::Group)
            .name(title)
            .style(style().width(40).height(20))
            .child(text(title)),
    }
}

impl Element for Card {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for Card {
    fn into_view(self) -> View {
        self.root.into_view()
    }
}

/// The user's modifiers land on the component's root node and win over the
/// component's own settings (written before them).
#[test]
fn node_builder_modifiers_land_on_the_root_node() {
    let (screen, (opened, pressed)) = mount(|| {
        let opened = Rc::new(Cell::new(0));
        let pressed = Rc::new(Cell::new(0));
        let (o, p) = (opened.clone(), pressed.clone());
        let view = column(
            card("Profile")
                .with_node(move |node| node.role(Role::Dialog))
                .name("Profile card")
                .test_id("card")
                .style(style().height(30))
                .with_node(move |node| {
                    node.a11y_action(A11yAction::Expand, move |_| o.set(o.get() + 1))
                })
                .on_press(move || p.set(p.get() + 1)),
        )
        .style(style().align_items(Align::Start));
        ((opened, pressed), view)
    });
    let el = screen.get_by_test_id("card");
    assert_eq!(el.role(), Some(Role::Dialog));
    assert_eq!(el.name(), "Profile card");
    let r = el.rect().expect("rect");
    assert_eq!(
        (r.width, r.height),
        (40.0, 30.0),
        "user style layers on top"
    );
    assert_eq!(el.get_by_text("Profile").text(), "Profile");
    el.a11y(A11yAction::Expand);
    el.click();
    assert_eq!((opened.get(), pressed.get()), (1, 1));
    assert!(screen.get_by_role(Role::Dialog, "Profile card").exists());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// Every element function returns `Node`, so the arms of a `match` that
/// build different elements need no `.into_view()`.
#[test]
fn element_functions_share_one_type() {
    let mode = |m: u8| match m {
        0 => text("t").test_id("t"),
        1 => column(text("c")).test_id("c"),
        2 => image(kivact::view::Asset::from(Vec::<u8>::new())).test_id("i"),
        _ => node().test_id("n"),
    };
    let screen = render(move || (mode(0), mode(1), mode(3)));
    assert_eq!(screen.get_by_test_id("t").text(), "t");
    assert_eq!(screen.get_by_test_id("c").get_by_text("c").text(), "c");
    assert!(screen.get_by_test_id("n").exists());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

// ---- writer-less values ---------------------------------------------------

// ---- roles ----------------------------------------------------------------

const COMPONENT_ROLES: [Role; 11] = [
    Role::RadioGroup,
    Role::RadioButton,
    Role::Switch,
    Role::TabPanel,
    Role::Splitter,
    Role::Table,
    Role::Row,
    Role::Cell,
    Role::ColumnHeader,
    Role::Tree,
    Role::TreeItem,
];

/// Each role added for the official components goes to the host and comes back
/// in queries and in the semantic snapshot.
#[test]
fn s9_roles_reach_the_screen() {
    let screen = render(|| {
        let nodes: Vec<View> = COMPONENT_ROLES
            .iter()
            .map(|&r| node().role(r).name(format!("{r:?}")).into_view())
            .collect();
        column(nodes)
    });
    let mut seen = Vec::new();
    let mut stack = vec![screen.snapshot()];
    while let Some(n) = stack.pop() {
        if let Some(r) = n.role
            && COMPONENT_ROLES.contains(&r)
        {
            assert_eq!(n.name, format!("{r:?}"));
            seen.push(r);
        }
        stack.extend(n.children);
    }
    assert_eq!(seen.len(), COMPONENT_ROLES.len(), "{seen:?}");
    for r in COMPONENT_ROLES {
        assert_eq!(
            screen.get_by_role(r, &format!("{r:?}")).role(),
            Some(r),
            "{r:?}"
        );
    }
}

// ---- What assistive technology is offered ----------------------------------

/// Actions follow the semantics policy: a disabled node and the scene
/// regions of a disabled node offer none, a text node is named by its
/// content, and `SetValue` from assistive technology ends an IME
/// composition with the new value. A field disabled while focused loses
/// the focus to the next tab stop and takes no more keys.
#[test]
fn disabled_nodes_offer_no_actions_and_lose_the_focus() {
    let (screen, (value, off)) = mount(|| {
        let (value, off) = (signal(String::from("abc")), signal(false));
        let view = column((
            node()
                .role(Role::Button)
                .name("off")
                .disabled(true)
                .on_press(|| {})
                .style(style().size(20, 20)),
            node()
                .role(Role::Canvas)
                .name("chart")
                .disabled(true)
                .style(style().size(40, 20))
                .on_press(|| {})
                .region(RegionId(1), {
                    let attachment = kivact::custom::part(|state| state);
                    kivact::custom::Part::new(attachment.state(), move |node| {
                        node.part(attachment)
                            .role(Role::Button)
                            .name("bar")
                            .on_press(|| {})
                    })
                })
                .draw(|d: &mut Draw, size: Size| {
                    d.region(RegionId(1), size.to_rect());
                }),
            text("Visual").name("ignored"),
            kivact::view::node()
                .name("Name")
                .part(kivact::interaction::use_text_edit(value).control())
                .disabled(off),
            node()
                .role(Role::Button)
                .name("next")
                .focusable(true)
                .style(style().size(20, 20)),
        ));
        ((value, off), view)
    });
    assert!(
        screen
            .get_by_role(Role::Button, "off")
            .a11y_actions()
            .is_empty()
    );
    assert!(
        screen
            .get_by_role(Role::Button, "bar")
            .a11y_actions()
            .is_empty()
    );
    screen.get_by_role(Role::Text, "Visual");

    let field = screen.get_by_role(Role::TextBox, "Name");
    field.compose("かな");
    field.a11y_set_value("new");
    assert_eq!(value.get_untracked(), "new");
    field.compose("");
    field.type_text("x");
    assert_eq!(value.get_untracked(), "newx", "the composition ended");

    off.set(true);
    screen.run_until_idle();
    assert_eq!(screen.focused().map(|e| e.name()).as_deref(), Some("next"));
    screen.press_key(Key::Char('z'));
    assert_eq!(value.get_untracked(), "newx");
}
