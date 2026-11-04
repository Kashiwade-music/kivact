//! Role: Tests for command palette behavior and filtering.
//!
//! The unstyled `CommandPalette`: opening, the active row, keys, filtering,
//! running, groups, the empty list, scrolling and the App's texts.

use kivact::view::View;
use std::cell::RefCell;
use std::rc::Rc;

use crate::support::mount;
use kivact::i18n::message;
use kivact::interaction::{CommandPalette, use_command_palette};
use kivact::prelude::*;
use kivact::test::{Element, Screen};
use kivact::interaction::Command;
use kivact::view::{Placement, overlay};
use kivact::state::callback;

type Cmd = Command<&'static str>;

/// Every row label the tests use.
const NAMES: [&str; 20] = [
    "New file",
    "Open file",
    "Save",
    "Toggle theme",
    "Zoom in",
    "Quit",
    "A",
    "B",
    "C",
    "D",
    "E",
    "F",
    "G",
    "H",
    "I",
    "J",
    "K",
    "L",
    "Alpha",
    "First of all",
];

fn base() -> Vec<Cmd> {
    vec![
        Cmd::new("new", "New file").group("File").hint("Ctrl N"),
        Cmd::new("open", "Open file")
            .group("File")
            .keywords(["load", "browse"]),
        Cmd::new("save", "Save").group("File").disabled(true),
        Cmd::new("theme", "Toggle theme")
            .group("View")
            .keywords(["dark"]),
        Cmd::new("zoom", "Zoom in").group("View"),
        Cmd::new("quit", "Quit"),
    ]
}

fn opener(open: Signal<bool>) -> Node {
    node()
        .role(Role::Button)
        .name("Open palette")
        .focusable(true)
        .style(style().size(120, 28))
        .child(text("Open palette"))
        .on_press(move || open.set(true))
}

fn palette_with_height(p: CommandPalette<&'static str>, height: f64) -> View {
    p.show(move || {
        let rows = for_each(
            p.visible(),
            |g| (g.index, g.heading.is_some()),
            move |(index, headed), group| {
                let commands = memo(move || group.with(|g| g.commands.clone()));
                let rows = for_each(
                    commands,
                    |c| *c.key(),
                    move |key, command| {
                        node()
                            .part(p.item(key))
                            .style(style().height(28))
                            .child(text(move || command.with(|c| c.label())))
                    },
                );
                if headed {
                    column((
                        text(move || {
                            group.with(|g| g.heading.as_ref().map(|h| h.get()).unwrap_or_default())
                        }),
                        rows,
                    ))
                    .part(p.group(index))
                    .into_view()
                } else {
                    column(rows).into_view()
                }
            },
        );
        let empty = show(
            move || p.visible().get().is_empty(),
            || text(message("palette.empty", "No commands")),
        );
        overlay(
            node_ref(),
            Placement::Center,
            column((
                kivact::view::node()
                    .name("")
                    .part(kivact::interaction::use_text_edit(p.query()).control())
                    .part(p.input()),
                scroll(column((rows, empty)).part(p.list())).style(style().max_height(height)),
            ))
            .style(style().width(400)),
        )
        .part(p.content())
    })
}

struct Page {
    screen: Screen,
    open: Signal<bool>,
    ran: Rc<RefCell<Vec<&'static str>>>,
}

fn page_with(
    commands: Vec<Cmd>,
    tweak: impl Fn(CommandPalette<&'static str>) -> CommandPalette<&'static str> + 'static,
) -> Page {
    page_with_height(commands, tweak, 360.0)
}

fn page_with_height(
    commands: Vec<Cmd>,
    tweak: impl Fn(CommandPalette<&'static str>) -> CommandPalette<&'static str> + 'static,
    height: f64,
) -> Page {
    let ran = Rc::new(RefCell::new(Vec::new()));
    let log = ran.clone();
    let (screen, open) = mount(move || {
        let open = signal(false);
        let run = callback(move |key| log.borrow_mut().push(key));
        (
            open,
            column((
                opener(open),
                palette_with_height(
                    tweak(use_command_palette(open, commands, move |value| {
                        run.try_call(value)
                    })),
                    height,
                ),
            )),
        )
    });
    Page { screen, open, ran }
}

fn page() -> Page {
    page_with(base(), |p| p)
}

impl Page {
    fn show(&self) {
        self.screen
            .get_by_role(Role::Button, "Open palette")
            .click();
    }
    fn field(&self) -> Element<'_> {
        self.screen.get_by_role(Role::TextBox, "Search commands")
    }
    /// The rows now listed (of the names the tests use), top to bottom.
    fn rendered(&self) -> Vec<Element<'_>> {
        let mut rows: Vec<Element<'_>> = NAMES
            .iter()
            .filter_map(|name| self.screen.query_by_role(Role::Option, name))
            .collect();
        let y = |e: &Element<'_>| e.rect().map(|r| r.y).unwrap_or_default();
        rows.sort_by(|a, b| y(a).total_cmp(&y(b)));
        rows
    }
    /// The one active row, if any: the field's active descendant.
    fn active(&self) -> Option<String> {
        let field = self.project();
        let rows = self.rendered();
        let mut active = rows.iter().filter(|r| field.has_active_descendant(r));
        let first = active.next().map(|r| r.name());
        assert!(active.next().is_none(), "at most one active descendant");
        first
    }
    fn selected(&self) -> Vec<String> {
        self.rendered()
            .iter()
            .filter(|r| r.is_selected())
            .map(|r| r.name())
            .collect()
    }
    fn no_errors(&self) {
        assert!(
            self.screen.errors().is_empty(),
            "{:?}",
            self.screen.errors()
        );
    }
}

#[test]
fn it_is_a_modal_dialog_named_after_the_palette_with_no_visible_heading() {
    let p = page();
    assert!(
        p.screen
            .query_by_role(Role::Dialog, "Command palette")
            .is_none()
    );
    p.show();
    assert!(p.open.get_untracked());
    p.screen.get_by_role(Role::Dialog, "Command palette");
    assert!(
        p.screen
            .query_by_role(Role::Heading, "Command palette")
            .is_none()
    );
    p.screen.get_by_role(Role::ListBox, "Command palette");
    assert_eq!(p.project().role(), Some(Role::TextBox));
    p.no_errors();
}

#[test]
fn the_arrow_keys_move_the_active_row_over_enabled_rows_and_wrap() {
    let p = page();
    p.show();
    p.screen.press_key(Key::ArrowDown);
    assert_eq!(p.active().as_deref(), Some("Open file"));
    p.screen.press_key(Key::ArrowDown);
    assert_eq!(
        p.active().as_deref(),
        Some("Toggle theme"),
        "Save is disabled"
    );
    p.screen.press_key(Key::ArrowDown);
    p.screen.press_key(Key::ArrowDown);
    assert_eq!(p.active().as_deref(), Some("Quit"));
    p.screen.press_key(Key::ArrowDown);
    assert_eq!(
        p.active().as_deref(),
        Some("New file"),
        "wraps to the start"
    );
    p.screen.press_key(Key::ArrowUp);
    assert_eq!(p.active().as_deref(), Some("Quit"), "wraps to the end");
    p.screen.press_key(Key::ArrowUp);
    assert_eq!(p.active().as_deref(), Some("Zoom in"));
    assert_eq!(p.selected(), ["Zoom in"]);
    assert!(p.project().is_focused());
    p.no_errors();
}

#[test]
fn enter_runs_the_active_command_and_closes() {
    let p = page();
    p.show();
    p.screen.press_key(Key::ArrowDown);
    p.screen.press_key(Key::Enter);
    assert_eq!(*p.ran.borrow(), ["open"]);
    assert!(!p.open.get_untracked());
    assert!(
        p.screen
            .query_by_role(Role::Dialog, "Command palette")
            .is_none()
    );
    p.no_errors();
}
