//! Role: A command palette joins a modal dialog and a combobox over commands.
//!
//! Unstyled behavior and semantics of a command palette: a modal with a
//! search field over a filtered, grouped list of commands.
//!
//! [`use_command_palette`] is a Copy handle joining a [`ComboboxApi`] and a
//! modal [`DialogApi`]. Its parts are `content`, `input`, `list`,
//! `group(index)` and `item(key)`, and its state is `visible` (a
//! `Value<Vec<CommandGroup<K>>>`), `active` and `query`; the parts add the
//! input's `controls` and `active_descendant`, the group names and the
//! options' `selected`. [`CommandPaletteApi::show`] only manages the content's
//! lifetime. A [`Command`] is plain data (key, reactive label, group, hint,
//! keywords, disabled), and the caller draws its icon from the key. The
//! default search is [`filter_commands`] (a substring of the label, group,
//! hint or keywords), replaced with `filter`. A successful run closes the
//! palette and a failed one leaves it open and reports. The shortcut that
//! opens it is the caller's; inside the modal, the closing shortcut goes to
//! `close_on`. The search field, the empty result, the headings, the footer,
//! the scroll height and the look are the caller's drawing.

use crate::Selection;
use kivact_core::custom::Reactive;
use kivact_core::view::HandlerResult;
use std::rc::Rc;

use crate::{
    ComboboxApi, DialogApi, DialogMode, ListItem, ListItemState, PopupState, use_combobox,
    use_dialog,
};
use kivact_core::UiError;
use kivact_core::custom::{Part, PartState, part};
use kivact_core::i18n::message;
use kivact_core::state::{Binding, Signal, Value, binding, memo, signal};
use kivact_core::state::{IntoText, IntoValue, callback};
use kivact_core::style::Icon;
use kivact_core::view::{Handled, IntoView, KeyEvent, Role, View};

/// One command a [`CommandPaletteApi`] lists.
///
/// A command is plain data. The key is the application's identity for it:
/// the palette looks rows up by key, hands the key of the chosen command to
/// its callback and keeps the active row on the same key when the list
/// changes. Keys must be unique in one palette.
///
/// The label, the group and the hint are [`Reactive`], so a localized
/// `message(..)` text follows a locale switch, and a command can be built
/// anywhere (a handler, a memo, a test). The keywords are plain text that only
/// the search reads. The [`icon`](Self::icon) is a name in the theme's icon
/// set or an image, which a palette draws at the start of the row.
///
/// ```
/// # mod kivact {
/// #     //! Role: Supply facade paths for this documentation example.
/// #     pub extern crate kivact_interaction as interaction;
/// # }
/// use kivact::interaction::Command;
/// let commands = vec![
///     Command::new("new", "New file").with_group("File").with_hint("Ctrl N").with_icon("file"),
///     Command::new("open", "Open file").with_group("File").with_keywords(["load"]),
///     Command::new("quit", "Quit").disabled(true),
/// ];
/// assert_eq!(commands.len(), 3);
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Command<K> {
    /// The application's identity for the command.
    pub(crate) key: K,
    /// What the row says; also its accessible name.
    pub(crate) label: Reactive<String>,
    /// The heading of the run of rows this one belongs to. Rows with no group
    /// have no heading.
    pub(crate) group: Option<Reactive<String>>,
    /// Text shown for the command, such as its shortcut.
    pub(crate) hint: Option<Reactive<String>>,
    /// Words the search matches that the row does not show.
    pub(crate) keywords: Vec<String>,
    /// The icon drawn at the start of the row.
    pub(crate) icon: Option<Icon>,
    /// A disabled command is skipped by the arrow keys, never active and
    /// never run.
    pub(crate) disabled: bool,
}

impl<K> Command<K> {
    /// A command with this key and label.
    pub fn new(key: K, label: impl IntoText) -> Self {
        Self {
            key,
            label: Reactive::new(label.into_text()),
            group: None,
            hint: None,
            keywords: Vec::new(),
            icon: None,
            disabled: false,
        }
    }

    /// Put the command under this heading. Neighbouring commands with the
    /// same group share one heading.
    pub fn with_group(mut self, group: impl IntoText) -> Self {
        self.group = Some(Reactive::new(group.into_text()));
        self
    }

    /// Show this text on the row, such as a shortcut.
    pub fn with_hint(mut self, hint: impl IntoText) -> Self {
        self.hint = Some(Reactive::new(hint.into_text()));
        self
    }

    /// Words only the search matches.
    pub fn with_keywords(mut self, words: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.keywords = words.into_iter().map(Into::into).collect();
        self
    }

    /// Show `icon` at the start of the row: a name the theme's icon set
    /// resolves (`"file"`) or an image.
    pub fn with_icon(mut self, icon: impl Into<Icon>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Whether the command cannot be chosen.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// The application's identity of the command.
    pub fn key(&self) -> &K {
        &self.key
    }

    /// What the row says, also its accessible name; tracked when read
    /// inside a computation.
    #[track_caller]
    pub fn label(&self) -> String {
        self.label.get()
    }

    /// The heading's current text ([`group`](Self::group)); tracked.
    #[track_caller]
    pub fn group(&self) -> Option<String> {
        self.group.as_ref().map(Reactive::get)
    }

    /// The hint's current text ([`hint`](Self::hint)); tracked.
    #[track_caller]
    pub fn hint(&self) -> Option<String> {
        self.hint.as_ref().map(Reactive::get)
    }

    /// The words only the search matches ([`keywords`](Self::keywords)).
    pub fn keywords(&self) -> &[String] {
        &self.keywords
    }

    /// The icon given to [`icon`](Self::icon).
    pub fn icon(&self) -> Option<&Icon> {
        self.icon.as_ref()
    }

    /// Whether the command cannot be chosen ([`disabled`](Self::disabled)).
    pub fn is_disabled(&self) -> bool {
        self.disabled
    }
}

/// The default search: the query, trimmed and lowercased, is part of the
/// label, the group, the hint or a keyword, lowercased. An empty query matches
/// everything.
pub fn filter_commands<K>(query: &str, command: &Command<K>) -> bool {
    let query = query.trim().to_lowercase();
    let has = |text: &str| text.to_lowercase().contains(&query);
    has(&command.label.get())
        || command.group.as_ref().is_some_and(|g| has(&g.get()))
        || command.hint.as_ref().is_some_and(|h| has(&h.get()))
        || command.keywords.iter().any(|word| has(word))
}

/// A run of neighbouring shown commands with the same group, which the
/// caller renders below one heading.
#[derive(Clone, Debug, PartialEq)]
pub struct CommandGroup<K> {
    /// The position of the run; key the caller's `for_each` by it (with
    /// whether it has a heading) so its rows stay while the list changes.
    pub index: usize,
    /// The group's heading; `None` for commands with no group.
    pub heading: Option<Reactive<String>>,
    /// The commands of the run, in order.
    pub commands: Vec<Command<K>>,
}

type CommandFilter<K> = Rc<dyn Fn(&str, &Command<K>) -> bool>;
type CloseOn = fn(&KeyEvent) -> bool;

/// Unstyled behavior and semantics of a command palette, the counterpart of
/// cmdk's `Command.Dialog`.
///
/// The caller owns the open state, through the `Binding<bool>` given to
/// [`use_command_palette`], the commands (a value, so they can change while the
/// palette is open) and a callback that receives the key of the chosen
/// command. The palette holds the query and the active row, both forgotten
/// when it closes. The shortcut that opens it (Ctrl K in most apps) is the
/// application's to bind.
///
/// It joins a modal [`DialogApi`] and a [`ComboboxApi`]: [`show`](Self::show)
/// mounts the caller's content while it is open, with
/// [`content`](Self::content) on the caller's centered `overlay(..)`,
/// [`input`](Self::input) on a caller-owned editing node bound to [`query`](Self::query),
/// [`list`](Self::list) on the node that holds the rows, [`group`](Self::group)
/// on each run of [`visible`](Self::visible) that has a heading, and
/// [`item`](Self::item) on each row. Running a command and closing stay
/// here, so a kit draws only the look: Enter or a click calls the callback
/// with the key of the active command and closes the palette; if the callback
/// fails the error is reported and the palette stays open.
///
/// When it opens the query is empty, the focus is on the field and the active
/// row is the first enabled one. Keys and pointer follow [`ComboboxApi`]: the
/// arrow keys move the active row and wrap at the ends, Home and End move the
/// caret, typing makes the first enabled match active again. Escape closes it,
/// and so does a key that [`close_on`](Self::close_on) matches. A press inside
/// the surface that does not land on the field leaves the focus on the field.
///
/// Semantics: the surface is a modal `Role::Dialog` named by the message
/// `palette.label` (`Command palette`), the field is named `palette.search`
/// (`Search commands`), the list is a `Role::ListBox` named like the dialog,
/// a group is a `Role::Group` named by its heading, and a row is a
/// `Role::Option` named by its label, `selected` while active.
///
/// ```no_run
/// # mod kivact {
/// #     //! Role: Supply facade paths for this documentation example.
/// # pub use kivact_core::{view, state, style};
/// # pub mod prelude {
/// # //! Role: Supply the canonical imports used by this documentation example.
/// # pub use kivact_core::state::{Binding, Callback, IntoBinding, IntoText, IntoValue, Signal, Value, binding, effect, memo, on_cleanup, on_mount, signal};
/// # pub use kivact_core::view::{EditPhase, Component, IntoView, Element, Orientation, Role, column, for_each, node, node_ref, row, show, text};
/// # pub use kivact_core::style::{ColorToken, Cursor, Style, style};
/// # }
/// #     pub extern crate kivact_interaction as interaction;
/// # }
/// use kivact::interaction::{Command, use_command_palette};
/// use kivact_core::view::Element;
/// use kivact::prelude::*;
/// use kivact::view::{Placement, overlay};
///
/// fn palette(open: Signal<bool>, last: Signal<Option<&'static str>>) -> impl IntoView {
///     let commands = vec![
///         Command::new("save", "Save").with_group("File").with_hint("Ctrl S"),
///         Command::new("theme", "Toggle theme").with_keywords(["dark", "light"]),
///     ];
///     let p = use_command_palette(commands, open).on_activate(move |key| last.set(Some(key)));
///     p.show(move || {
///         let rows = for_each(p.visible(), |g| g.index, move |group| {
///             let commands = memo(move || group.with(|g| g.commands.clone()));
///             for_each(commands, |c| *c.key(), move |command| {
///                 let key = command.with(|c| *c.key());
///                 node()
///                     .part(p.item(key))
///                     .child(text(move || command.with(|c| c.label())))
///             })
///         });
///         overlay(node_ref(), Placement::Center, column((
///             kivact_core::view::node().name("").part(kivact_interaction::use_text_edit(p.query()).control()).part(p.input()),
///             column(rows).part(p.list()),
///         )))
///         .with_node(|node| node.part(p.content()))
///     })
/// }
/// ```
pub struct CommandPaletteApi<K: kivact_core::state::ItemKey> {
    open: Binding<bool>,
    run: kivact_core::state::Var<kivact_core::state::Callback<K>>,
    dialog: DialogApi,
    combobox: ComboboxApi<K>,
    filter: Signal<CommandFilter<K>>,
    close_on: Signal<Option<CloseOn>>,
    groups: Value<Vec<CommandGroup<K>>>,
}

impl<K: kivact_core::state::ItemKey> Clone for CommandPaletteApi<K> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<K: kivact_core::state::ItemKey> Copy for CommandPaletteApi<K> {}

/// A palette shown while `open` is true over `commands`, calling `run` with
/// the key of the command the user chooses. Setup only.
#[track_caller]
pub fn use_command_palette<K: kivact_core::state::ItemKey>(
    commands: impl IntoValue<Vec<Command<K>>>,
    open: impl kivact_core::state::IntoBinding<bool>,
) -> CommandPaletteApi<K> {
    let open = open.into_binding();
    let commands = commands.into_value();
    let run = kivact_core::state::var(callback(|_: K| ()));
    let filter = signal::<CommandFilter<K>>(Rc::new(filter_commands));
    let options = memo(move || {
        commands.with(|all| {
            all.iter()
                .map(|command| ListItem {
                    key: command.key.clone(),
                    label: command.label.clone(),
                    disabled: command.disabled,
                    icon: None,
                    pressed_icon: None,
                    description: None,
                })
                .collect::<Vec<_>>()
        })
    });
    // The palette runs the chosen command and closes; it keeps no chosen
    // value. A failing run is reported and leaves the palette open.
    let chosen = binding(
        None::<K>.into_value(),
        move |key: Option<K>| -> Result<(), UiError> {
            if let Some(key) = key {
                run.get().try_call(key)?;
                open.set(false);
            }
            Ok(())
        },
    );
    let combobox = use_combobox(options, Selection::Single(chosen)).filter(move |query, option| {
        commands.with(|all| {
            all.iter()
                .find(|command| command.key == option.key)
                .is_some_and(|command| filter.with(|filter| filter(query, command)))
        })
    });
    // The caller's `open` is the only open state. Each change to closed
    // resets the query and the highlight; a notification whose value the
    // binding no longer holds is ignored.
    kivact_core::state::effect_on(open, move |&shown| {
        if !shown && !open.get_untracked() {
            combobox.reset();
        }
    });
    let visible = combobox.visible();
    let groups = memo(move || command_groups(visible, commands));
    CommandPaletteApi {
        run,
        open,
        dialog: use_dialog(open, DialogMode::Modal).initial_focus(combobox.input_ref()),
        combobox,
        filter,
        close_on: signal(None),
        groups,
    }
}

impl<K: kivact_core::state::ItemKey> CommandPaletteApi<K> {
    /// Drawing state of the mounted palette surface.
    pub fn content_state(self) -> Value<PopupState> {
        self.dialog.content_state()
    }
    /// Drawing state of the mounted query editor.
    pub fn input_state(self) -> Value<PartState> {
        self.combobox.input_state()
    }
    /// Drawing state of the mounted result list.
    pub fn list_state(self) -> Value<PartState> {
        self.combobox.content_state()
    }
    /// Drawing state of a mounted command.
    pub fn item_state(self, key: K) -> Value<ListItemState> {
        self.combobox.item_state(key)
    }

    /// Handle activation of a command.
    pub fn on_activate<R: HandlerResult>(self, f: impl Fn(K) -> R + 'static) -> Self {
        self.run.set(callback(f));
        self
    }
    /// Whether the palette is open.
    pub fn is_open(self) -> Value<bool> {
        self.open.into_value()
    }

    /// Choose which commands the query shows. It receives the query as typed
    /// and a command, and reads the command's reactive texts with `get()`, so
    /// the list follows a locale switch. The default is [`filter_commands`].
    /// Setup only.
    #[track_caller]
    pub fn filter(self, filter: impl Fn(&str, &Command<K>) -> bool + 'static) -> Self {
        let filter: CommandFilter<K> = Rc::new(filter);
        self.filter.update(move |slot| *slot = filter);
        self
    }

    /// Close the palette on a key that `matches`, the same way Escape does.
    ///
    /// A modal keeps the keys to itself, so the application's own shortcut
    /// (Ctrl K, say) cannot reach it while the palette is open. Give the
    /// shortcut that opens it here and it toggles, as cmdk's does. The
    /// function sees every key pressed in the field. Setup only.
    #[track_caller]
    pub fn close_on(self, matches: fn(&KeyEvent) -> bool) -> Self {
        self.close_on.set(Some(matches));
        self
    }

    /// The commands the query shows, in runs of neighbouring commands with
    /// the same group, for the caller's `for_each`.
    pub fn visible(self) -> Value<Vec<CommandGroup<K>>> {
        self.groups
    }

    /// The active command, the one Enter runs.
    pub fn active(self) -> Value<Option<K>> {
        self.combobox.active()
    }

    /// The query: bind the field's text to it.
    pub fn query(self) -> Binding<String> {
        self.combobox.query()
    }

    /// The modal surface, for the caller's centered `overlay(..)`: a
    /// `Role::Dialog` named by the message `palette.label`. Setup only.
    #[track_caller]
    pub fn content(self) -> Part<PopupState> {
        let base = self.dialog.content();
        let part_state = base.state();
        Part::new(part_state, move |host| {
            host.part(base)
                .name(message("palette.label", "Command palette"))
                .focus_on_press(self.combobox.input_ref())
        })
    }

    /// The search field, named by the message `palette.search`, with
    /// [`close_on`](Self::close_on) before the combobox's keys. Bind its
    /// text to [`query`](Self::query). Setup only.
    #[track_caller]
    pub fn input(self) -> Part<PartState> {
        let close_on = self.close_on;
        let open = self.open;
        let base = self.combobox.input();
        let part_state = base.state();
        Part::new(part_state, move |host| {
            host.part(base)
                .name(message("palette.search", "Search commands"))
                .on_key(move |event: KeyEvent| {
                    if close_on.get().is_some_and(|matches| matches(&event)) {
                        open.set(false);
                        return Handled::Yes;
                    }
                    Handled::No
                })
        })
    }

    /// The list box that holds the rows, named like the dialog. Setup only.
    #[track_caller]
    pub fn list(self) -> Part<PartState> {
        let base = self.combobox.content();
        let part_state = base.state();
        Part::new(part_state, move |host| {
            host.part(base)
                .name(message("palette.label", "Command palette"))
        })
    }

    /// The run `index` of [`visible`](Self::visible), which has a heading:
    /// a `Role::Group` named by it. Setup only, in the run's row.
    #[track_caller]
    pub fn group(self, index: usize) -> Part<PartState> {
        let groups = self.groups;
        let input = part(|input| input);
        let part_state = input.state();
        Part::new(part_state, move |host| {
            host.part(input).role(Role::Group).name(move || {
                groups.with(|groups| {
                    groups
                        .get(index)
                        .and_then(|group| group.heading.as_ref())
                        .map(|heading| heading.get())
                        .unwrap_or_default()
                })
            })
        })
    }

    /// The row of the command `key`; see [`ComboboxApi::item`]. Its state's
    /// `active` is whether it is the active command. Setup only.
    #[track_caller]
    pub fn item(self, key: K) -> Part<ListItemState> {
        self.combobox.item(key)
    }

    /// Mount the caller's content while the palette is open, in a fresh
    /// owner on every opening. Nothing is rendered besides the factory's view.
    ///
    /// The palette keeps its open state with its dialog, so this method,
    /// like [`MenuApi::show`](crate::MenuApi::show), is how its content mounts;
    /// the other overlays mount theirs with `show(x.is_open(), ..)`.
    #[track_caller]
    pub fn show<V: IntoView>(self, factory: impl kivact_core::view::IntoViewFn<V>) -> View {
        let factory = factory.into_view_fn();
        kivact_core::view::show(self.dialog.is_open(), move || factory.call(())).into_view()
    }
}

fn command_groups<K: kivact_core::state::ItemKey>(
    visible: Value<Vec<K>>,
    commands: Value<Vec<Command<K>>>,
) -> Vec<CommandGroup<K>> {
    let mut out = Vec::<CommandGroup<K>>::new();
    let mut last: Option<Option<String>> = None;
    visible.with(|keys| {
        commands.with(|all| {
            for key in keys {
                let Some(command) = all.iter().find(|command| command.key == *key) else {
                    continue;
                };
                let group = command.group.as_ref().map(|g| g.get());
                if last.as_ref() != Some(&group) {
                    out.push(CommandGroup {
                        index: out.len(),
                        heading: command.group.clone(),
                        commands: Vec::new(),
                    });
                    last = Some(group);
                }
                if let Some(run) = out.last_mut() {
                    run.commands.push(command.clone());
                }
            }
        })
    });
    out
}
