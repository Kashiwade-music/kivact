//! Role: These tests verify component builders and shared modifiers.
//!
//! Component builders that hold a root
//! `Node` get the standard modifiers through `NodeBuilder`, writer-less values
//! bind as cheaply as constants, and the roles added for the official components
//! reach the screen.

use kivact::primitive::NodeExt;
use kivact::view::View;
use std::cell::Cell;
use std::rc::Rc;

use crate::support::mount;
use kivact::draw::{Draw, RegionId, Size};
use kivact::prelude::*;
use kivact::style::Align;
use kivact::test::render;
use kivact::view::A11yAction;

// ---- NodeBuilder -------------------------------------------------------------

/// An external-style component: a builder struct around its root `Node`.
struct Card {
    root: Node,
}

#[track_caller]
fn card(title: impl IntoText) -> Card {
    let title = title.into_text_value();
    Card {
        root: node()
            .role(Role::Group)
            .name(title)
            .style(style().width(40).height(20))
            .child(text(title)),
    }
}

impl NodeBuilder for Card {
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
                .role(Role::Dialog)
                .name("Profile card")
                .test_id("card")
                .style(style().height(30))
                .on_a11y(A11yAction::Expand, move || o.set(o.get() + 1))
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

/// A helper bounded by `NodeBuilder` takes a component or a plain `Node`.
#[test]
fn node_builder_helpers_accept_a_node_too() {
    fn tagged<B: NodeBuilder>(b: B, id: &str) -> B {
        b.test_id(id).role(Role::Status)
    }
    let screen = render(|| column((tagged(card("a"), "card"), tagged(node(), "box"))));
    assert_eq!(screen.get_by_test_id("card").role(), Some(Role::Status));
    assert_eq!(screen.get_by_test_id("box").role(), Some(Role::Status));
}

/// Every element function returns `Node`, so the arms of a `match` that
/// build different elements need no `.into_view()`.
#[test]
fn element_functions_share_one_type() {
    let mode = |m: u8| match m {
        0 => text("t").test_id("t"),
        1 => column(text("c")).test_id("c"),
        2 => image(kivact::Asset::from(Vec::<u8>::new())).test_id("i"),
        _ => node().test_id("n"),
    };
    let screen = render(move || (mode(0), mode(1), mode(3)));
    assert_eq!(screen.get_by_test_id("t").text(), "t");
    assert_eq!(screen.get_by_test_id("c").get_by_text("c").text(), "c");
    assert!(screen.get_by_test_id("n").exists());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// A setting only one kind of node shows, given to another kind, is
/// reported once at mount, located at the node, and dropped: a text keeps
/// its content and gets no child.
#[test]
fn a_setting_for_another_kind_of_node_is_reported_and_dropped() {
    let screen = render(|| {
        column((
            text("label")
                .test_id("label")
                .child(text("inner"))
                .draw(|_, _| {}),
            node().test_id("box").fit(kivact::control::ImageFit::Cover),
        ))
    });
    assert_eq!(screen.get_by_test_id("label").text(), "label");
    assert!(screen.query_by_text("inner").is_none());
    assert!(screen.get_by_test_id("box").exists());
    let errors = screen.errors();
    assert_eq!(errors.len(), 2, "{errors:?}");
    assert!(
        errors
            .iter()
            .all(|e| *e.kind() == kivact::diagnostics::ErrorKind::Unsupported
                && e.location()
                    .is_some_and(|l| l.file().ends_with("components.rs"))),
        "{errors:?}"
    );
    assert!(
        errors[0]
            .to_string()
            .contains("`.child(..)`, `.draw(..)` on a text node"),
        "{errors:?}"
    );
    assert!(
        errors[1].to_string().contains("`.fit(..)` on a box node"),
        "{errors:?}"
    );
}

/// An overlay and a virtual list take the node modifiers; the draw layers
/// and size hint that only a box node shows are reported once at mount,
/// located at the builder, and dropped, the rest of the modifiers land.
#[test]
fn an_overlay_and_a_virtual_list_report_the_modifiers_they_cannot_show() {
    let screen = render(|| {
        let anchor = node_ref();
        let rows = signal(vec![1u32, 2]);
        (
            node().node_ref(anchor),
            kivact::view::overlay(anchor, kivact::view::Placement::Below, text("menu"))
                .role(Role::Menu)
                .name("Actions")
                .test_id("overlay")
                .draw(|_, _| {})
                .intrinsic_size(kivact::draw::IntrinsicSize::default()),
            virtual_list(
                rows,
                |row| *row,
                |_, row| text(move || row.get().to_string()),
            )
            .name("Rows")
            .test_id("rows")
            .draw_over(|_, _| {}),
        )
    });
    assert_eq!(screen.get_by_test_id("overlay").role(), Some(Role::Menu));
    assert_eq!(screen.get_by_test_id("rows").name(), "Rows");
    let errors = screen.errors();
    assert_eq!(errors.len(), 2, "{errors:?}");
    assert!(
        errors
            .iter()
            .all(|e| *e.kind() == kivact::diagnostics::ErrorKind::Unsupported
                && e.location()
                    .is_some_and(|l| l.file().ends_with("components.rs"))),
        "{errors:?}"
    );
    assert!(
        errors[0]
            .to_string()
            .contains("`.draw(..)`, `.intrinsic_size(..)` on an overlay"),
        "{errors:?}"
    );
    assert!(
        errors[1]
            .to_string()
            .contains("`.draw_over(..)` on a virtual list"),
        "{errors:?}"
    );
}

// ---- writer-less values ---------------------------------------------------

/// What mounting `f` cost: (ops, set_props, creates, live reactive nodes).
fn cost(f: fn() -> View) -> (u64, u64, u64, usize) {
    let screen = render(f);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    let s = screen.stats();
    (s.ops, s.set_props, s.creates, s.live_reactive)
}

/// `into_value` of owned and borrowed constants sends the same ops as a
/// plain constant and creates no graph node or effect. The owner holds the
/// value in the constant arena.
#[test]
fn writer_less_values_bind_like_constants() {
    let plain = cost(|| node().name("x").child(text("x")).into_view());
    let constant = cost(|| {
        node()
            .name(("x".to_string()).into_value())
            .child(text(("x".to_string()).into_value()))
            .into_view()
    });
    let into_value = cost(|| {
        let c = "x".into_value();
        node().name(c).child(text(c)).into_view()
    });
    let signal_cost = cost(|| {
        let s = signal("x".to_string());
        node().name(s).child(text(s)).into_view()
    });
    let ops = |c: (u64, u64, u64, usize)| (c.0, c.1, c.2);
    assert_eq!(ops(constant), ops(plain));
    assert_eq!(ops(into_value), ops(plain));
    assert_eq!(constant.3, plain.3, "constants are not graph nodes");
    assert_eq!(into_value.3, plain.3, "a constant is not a graph node");
    // A signal is dynamic: its source plus one render effect per prop.
    assert_eq!(signal_cost.3, plain.3 + 3);
}

/// `disabled` and `selected` feed semantics and the state styles; a value
/// that never changes is set once instead of getting an effect.
#[test]
fn constant_states_need_no_effect() {
    let base = cost(|| node().into_view());
    let plain = cost(|| node().disabled(true).selected(false).into_view());
    let constant = cost(|| node().disabled(true.into_value()).into_view());
    assert_eq!(plain.3, base.3);
    assert_eq!(constant.3, base.3);
    let screen = render(|| {
        node()
            .disabled(true.into_value())
            .selected(true)
            .test_id("b")
    });
    let el = screen.get_by_test_id("b");
    assert!(el.is_disabled() && el.is_selected());
}

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
                .region(
                    RegionId(1),
                    kivact::primitive::part(|state| state)
                        .role(Role::Button)
                        .name("bar")
                        .on_press(|| {}),
                )
                .draw(|d: &mut Draw, size: Size| {
                    d.region(RegionId(1), size.to_rect());
                }),
            text("Visual").name("ignored"),
            kivact::control::node()
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
    field.a11y_with_value(A11yAction::SetValue, "new");
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
