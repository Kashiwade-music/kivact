//! Role: Tests for command palette behavior and filtering.
//!
//! The unstyled `CommandPalette`: opening, the active row, keys, filtering,
//! running, groups, the empty list, scrolling and the App's texts.

use kivact::primitive::NodeExt;
use kivact::view::View;
use std::cell::RefCell;
use std::rc::Rc;

use crate::support::mount;
use kivact::UiError;
use kivact::i18n::{Messages, message};
use kivact::interaction::{CommandPalette, use_command_palette};
use kivact::prelude::*;
use kivact::test::{Element, Screen};
use kivact::ui::Command;
use kivact::view::Modifiers;
use kivact::view::provide_messages;
use kivact::view::{KeyEvent, Placement, callback, overlay};

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

/// Caller rendering keeps behavior tests independent of kit appearance.
fn palette(p: CommandPalette<&'static str>) -> View {
    palette_with_height(p, 360.0)
}

fn palette_with_height(p: CommandPalette<&'static str>, height: f32) -> View {
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
                kivact::control::node()
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
    height: f32,
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
    fn option(&self, name: &str) -> Element<'_> {
        self.screen.get_by_role(Role::Option, name)
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
    /// The names of the rows, in the order they are listed.
    fn rows(&self) -> Vec<String> {
        self.rendered().iter().map(|e| e.name()).collect()
    }
    /// The one active row, if any: the field's active descendant.
    fn active(&self) -> Option<String> {
        let field = self.field();
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
    /// Replace what is in the field with `text`.
    fn set_query(&self, text: &str) {
        let field = self.field();
        field.press_key_with(
            Key::Char('a'),
            Modifiers {
                ctrl: true,
                ..Modifiers::default()
            },
        );
        if text.is_empty() {
            field.press_key(Key::Backspace);
        } else {
            field.type_text(text);
        }
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
    assert_eq!(p.field().role(), Some(Role::TextBox));
    p.no_errors();
}

#[test]
fn opening_focuses_the_field_and_makes_the_first_enabled_row_active() {
    let p = page();
    p.show();
    assert!(p.field().is_focused());
    assert_eq!(p.active().as_deref(), Some("New file"));
    assert_eq!(p.selected(), ["New file"]);
    assert_eq!(
        p.rows(),
        [
            "New file",
            "Open file",
            "Save",
            "Toggle theme",
            "Zoom in",
            "Quit"
        ]
    );
    p.no_errors();
}

#[test]
fn the_rows_are_never_focused_and_the_focus_stays_on_the_field() {
    let p = page();
    p.show();
    p.screen.press_key(Key::ArrowDown);
    p.screen.press_key(Key::ArrowDown);
    assert!(p.field().is_focused());
    for row in p.rendered() {
        assert!(!row.is_focused());
    }
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
    assert!(p.field().is_focused());
    p.no_errors();
}

#[test]
fn home_and_end_are_left_to_the_field() {
    let p = page();
    p.show();
    p.screen.press_key(Key::ArrowDown);
    p.screen.press_key(Key::End);
    assert_eq!(p.active().as_deref(), Some("Open file"));
    p.screen.press_key(Key::Home);
    assert_eq!(p.active().as_deref(), Some("Open file"));
    p.no_errors();
}

#[test]
fn typing_filters_and_makes_the_first_enabled_row_active_again() {
    let p = page();
    p.show();
    p.screen.press_key(Key::ArrowDown);
    p.screen.press_key(Key::ArrowDown);
    assert_eq!(p.active().as_deref(), Some("Toggle theme"));
    p.screen.type_text("o");
    assert_eq!(p.rows(), ["Open file", "Toggle theme", "Zoom in"]);
    assert_eq!(
        p.active().as_deref(),
        Some("Open file"),
        "back to the first"
    );
    p.screen.type_text("p");
    assert_eq!(p.rows(), ["Open file"]);
    assert_eq!(p.active().as_deref(), Some("Open file"));
    p.no_errors();
}

#[test]
fn the_default_filter_reads_the_label_group_hint_and_keywords() {
    let p = page();
    p.show();
    let search = |text: &str| {
        p.set_query(text);
        p.rows()
    };
    assert_eq!(search("zoom"), ["Zoom in"], "label");
    assert_eq!(search("  ZOOM "), ["Zoom in"], "case and surrounding space");
    assert_eq!(search("view"), ["Toggle theme", "Zoom in"], "group");
    assert_eq!(search("ctrl"), ["New file"], "hint");
    assert_eq!(search("browse"), ["Open file"], "keyword");
    assert_eq!(search("dark"), ["Toggle theme"], "keyword");
    assert_eq!(search("nothing like it"), Vec::<String>::new());
    assert_eq!(search("").len(), 6, "an empty query lists everything");
    p.no_errors();
}

#[test]
fn a_custom_filter_replaces_the_default() {
    let p = page_with(base(), |p| {
        p.filter(|query, command| command.label().starts_with(query))
    });
    p.show();
    p.set_query("S");
    assert_eq!(p.rows(), ["Save"], "only a prefix of the label");
    p.set_query("ave");
    assert!(p.rows().is_empty(), "the default would match 'ave' in Save");
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

#[test]
fn a_failed_command_keeps_the_palette_open_and_reports_the_error() {
    let (screen, open) = mount(|| {
        let open = signal(true);
        let p = use_command_palette(open, base(), |_: &'static str| {
            Err::<(), _>(UiError::msg("command failed"))
        });
        (open, palette(p))
    });
    screen.press_key(Key::Enter);
    assert!(open.get_untracked());
    assert!(
        screen
            .get_by_role(Role::TextBox, "Search commands")
            .is_focused()
    );
    screen.get_by_role(Role::Dialog, "Command palette");
    assert_eq!(screen.errors().len(), 1, "{:?}", screen.errors());
}

#[test]
fn enter_with_no_row_does_nothing_and_a_reopened_palette_starts_over() {
    let p = page();
    p.show();
    p.set_query("zzz");
    p.screen.press_key(Key::Enter);
    assert!(p.ran.borrow().is_empty());
    assert!(p.open.get_untracked(), "it stays open");
    p.screen.press_key(Key::Escape);
    assert!(!p.open.get_untracked());
    p.show();
    assert_eq!(p.rows().len(), 6, "the query is forgotten");
    assert_eq!(p.active().as_deref(), Some("New file"));
    p.no_errors();
}

#[test]
fn a_click_runs_the_command_and_closes() {
    let p = page();
    p.show();
    p.option("Toggle theme").click();
    assert_eq!(*p.ran.borrow(), ["theme"]);
    assert!(!p.open.get_untracked());
    p.no_errors();
}

#[test]
fn escape_closes_without_running() {
    let p = page();
    p.show();
    p.screen.press_key(Key::Escape);
    assert!(!p.open.get_untracked());
    assert!(p.ran.borrow().is_empty());
    assert!(
        p.screen
            .get_by_role(Role::Button, "Open palette")
            .is_focused()
    );
    p.no_errors();
}

fn ctrl_k(key: &KeyEvent) -> bool {
    (key.modifiers.ctrl || key.modifiers.meta)
        && matches!(key.key, Key::Char(c) if c.eq_ignore_ascii_case(&'k'))
}

#[test]
fn a_close_on_key_closes_like_escape_and_other_keys_do_not() {
    let p = page_with(base(), |p| p.close_on(ctrl_k));
    p.show();
    let ctrl = |key: char| {
        p.screen.press_key_with(
            Key::Char(key),
            Modifiers {
                ctrl: true,
                ..Modifiers::default()
            },
        )
    };
    ctrl('j');
    p.screen.press_key(Key::Char('k'));
    p.screen.press_key(Key::ArrowDown);
    assert!(p.open.get_untracked(), "other keys leave it open");
    ctrl('k');
    assert!(!p.open.get_untracked());
    assert!(
        p.screen
            .query_by_role(Role::Dialog, "Command palette")
            .is_none()
    );
    assert!(p.ran.borrow().is_empty(), "closing runs nothing");
    p.show();
    ctrl('K');
    assert!(!p.open.get_untracked(), "and it opens again to close again");
    p.no_errors();
}

#[test]
fn without_close_on_ctrl_k_does_nothing_special() {
    let p = page();
    p.show();
    p.screen.press_key_with(
        Key::Char('k'),
        Modifiers {
            ctrl: true,
            ..Modifiers::default()
        },
    );
    assert!(p.open.get_untracked());
    p.no_errors();
}

#[test]
fn the_pointer_moves_the_active_row_and_a_disabled_row_stays_out() {
    let p = page();
    p.show();
    p.option("Zoom in").hover();
    assert_eq!(p.active().as_deref(), Some("Zoom in"));
    assert_eq!(p.selected(), ["Zoom in"]);
    p.option("Save").hover();
    assert_eq!(
        p.active().as_deref(),
        Some("Zoom in"),
        "disabled: no change"
    );
    assert!(p.option("Save").is_disabled());
    let save = p.option("Save").rect().expect("laid out");
    p.screen
        .click_at(save.x + save.width / 2.0, save.y + save.height / 2.0);
    assert!(p.ran.borrow().is_empty(), "a disabled row does not run");
    assert!(p.open.get_untracked());
    assert!(p.field().is_focused());
    p.no_errors();
}

#[test]
fn neighbouring_commands_of_one_group_share_a_named_group_with_a_heading() {
    let p = page();
    p.show();
    let file = p.screen.get_by_role(Role::Group, "File");
    let view = p.screen.get_by_role(Role::Group, "View");
    assert_eq!(p.screen.all_by_text("File").len(), 1, "one heading");
    assert!(file.rect().is_some());
    // Rows of a group lie inside it; "Quit" has no group.
    let inside = |outer: &Element<'_>, inner: &Element<'_>| {
        let (o, i) = (outer.rect().unwrap(), inner.rect().unwrap());
        i.y >= o.y && i.y + i.height <= o.y + o.height + 0.5
    };
    assert!(inside(&file, &p.option("Open file")));
    assert!(!inside(&file, &p.option("Quit")));
    assert!(!inside(&view, &p.option("Quit")));
    assert!(inside(&view, &p.option("Zoom in")));
    p.no_errors();
}

#[test]
fn a_group_that_comes_back_later_gets_a_heading_of_its_own() {
    let commands = vec![
        Cmd::new("a", "Alpha").group("One"),
        Cmd::new("b", "Beta").group("Two"),
        Cmd::new("c", "Gamma").group("One"),
    ];
    let p = page_with(commands, |p| p);
    p.show();
    assert_eq!(p.screen.all_by_role(Role::Group, "One").len(), 2);
    p.no_errors();
}

#[test]
fn nothing_matching_shows_no_commands_and_nothing_to_run() {
    let p = page();
    p.show();
    assert!(p.screen.query_by_text("No commands").is_none());
    p.set_query("qqqq");
    p.screen.get_by_text("No commands");
    assert_eq!(p.active(), None);
    p.set_query("");
    assert!(p.screen.query_by_text("No commands").is_none());
    assert_eq!(p.active().as_deref(), Some("New file"));
    p.no_errors();
}

fn many() -> Vec<Cmd> {
    ["a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l"]
        .into_iter()
        .map(|k| Cmd::new(k, k.to_uppercase()))
        .collect()
}

#[test]
fn the_list_scrolls_at_max_height_and_keeps_the_active_row_in_view() {
    let p = page_with_height(many(), |p| p, 100.0);
    p.show();
    let scroller = p.screen.get_by_role(Role::ScrollView, "");
    let view = scroller.rect().expect("laid out");
    assert!(view.height <= 100.5, "the list is {} high", view.height);
    let last = p.option("L");
    let beyond = |row: &Element<'_>| {
        let r = row.rect().expect("laid out");
        r.y + r.height > view.y + view.height + 0.5
    };
    // The 12th row starts out of sight; wrapping up to it scrolls it in.
    p.screen.press_key(Key::ArrowUp);
    assert_eq!(p.active().as_deref(), Some("L"));
    assert!(!beyond(&last), "the active row was scrolled into view");
    p.screen.press_key(Key::ArrowDown);
    assert_eq!(p.active().as_deref(), Some("A"));
    let first = p.option("A").rect().expect("laid out");
    assert!(first.y >= view.y - 0.5, "and back to the top");
    p.no_errors();
}

#[test]
fn commands_that_change_while_open_keep_the_active_row_or_pick_the_first() {
    let ran = Rc::new(RefCell::new(Vec::new()));
    let log = ran.clone();
    let (screen, (list, open)) = mount(move || {
        let open = signal(true);
        let list = signal(base());
        (
            (list, open),
            palette(use_command_palette(open, list, move |key: &'static str| {
                log.borrow_mut().push(key)
            })),
        )
    });
    let page = Page { screen, open, ran };
    page.screen.press_key(Key::ArrowDown);
    assert_eq!(page.active().as_deref(), Some("Open file"));
    list.update(|l| l.insert(0, Cmd::new("first", "First of all")));
    page.screen.run_until_idle();
    assert_eq!(
        page.active().as_deref(),
        Some("Open file"),
        "same key stays"
    );
    list.update(|l| l.retain(|c| *c.key() != "open"));
    page.screen.run_until_idle();
    assert_eq!(page.active().as_deref(), Some("First of all"));
    page.screen.press_key(Key::Enter);
    assert_eq!(*page.ran.borrow(), ["first"]);
    page.no_errors();
}

#[test]
fn the_texts_and_the_labels_follow_a_locale_switch() {
    let (screen, messages) = mount(|| {
        let messages = signal(Messages::new());
        let open = signal(true);
        (
            messages,
            column((provide_messages(messages, move || {
                let commands = vec![
                    Cmd::new("save", message("cmd.save", "Save")),
                    Cmd::new("quit", "Quit"),
                ];
                palette(use_command_palette(open, commands, |_: &'static str| ()))
            }),)),
        )
    });
    screen.get_by_role(Role::Dialog, "Command palette");
    screen.get_by_role(Role::TextBox, "Search commands");
    screen.get_by_role(Role::Option, "Save");
    messages.set(
        Messages::new()
            .with("palette.label", "コマンドパレット")
            .with("palette.search", "コマンド検索")
            .with("palette.placeholder", "コマンドを入力")
            .with("palette.empty", "一致なし")
            .with("cmd.save", "保存"),
    );
    screen.run_until_idle();
    screen.get_by_role(Role::Dialog, "コマンドパレット");
    let field = screen.get_by_role(Role::TextBox, "コマンド検索");
    screen.get_by_role(Role::Option, "保存");
    field.focus();
    screen.type_text("zzz");
    screen.get_by_text("一致なし");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
