//! Role: Render a themed command palette over the shared palette behavior.
use super::layer::with_recipe_style;
use super::variants::{Variant, Variants};
use kivact_core::control::{NodeBuilder, column, icon, node, row, scroll, text};
use kivact_core::i18n::message;
use kivact_core::primitive::{NodeExt, PartState};
use kivact_core::style::{Icon, current_theme, style};
use kivact_core::view::{
    HandlerResult, IntoText, IntoValue, IntoView, KeyEvent, Placement, View, dynamic, for_each,
    node_ref, overlay, show,
};
use kivact_core::{Binding, Value, memo};
use kivact_interaction::{Command, use_command_palette, use_text_edit};
use std::hash::Hash;
use std::rc::Rc;

type Filter<K> = Rc<dyn Fn(&str, &Command<K>) -> bool>;
type Footer = Rc<dyn Fn() -> View>;

/// A searchable command palette in a modal, created by [`command_palette`].
///
/// It is a view: put it in the tree once; it draws nothing while the palette
/// is closed.
pub struct CommandPalette<K: Clone + Eq + Hash + 'static> {
    open: Binding<bool>,
    commands: Value<Vec<Command<K>>>,
    run: Rc<dyn Fn(K) -> kivact_core::Result<()>>,
    filter: Option<Filter<K>>,
    close_on: Option<fn(&KeyEvent) -> bool>,
    placeholder: Option<Value<String>>,
    empty: Option<Value<String>>,
    footer: Option<Footer>,
    variants: Variants,
}

/// A command palette shown while `open` is true over `commands`, calling
/// `run` with the key of the command the user chooses.
///
/// The palette is a modal `Role::Dialog` named by the message
/// `palette.label` ("Command palette"), with a search field named by
/// `palette.search` ("Search commands") over the commands, grouped by
/// [`Command::group`] and filtered as the user types. The arrow keys move the
/// active row, Enter or a click runs it and closes the palette (a failing
/// `run` is reported and leaves it open), Escape closes it, and the focus
/// stays in the field. A row shows the command's [`icon`](Command::icon),
/// label and hint. The key that opens the palette (Ctrl K, say) is the
/// application's to bind; hand the same key to
/// [`close_on`](CommandPalette::close_on) so that it closes the palette too.
///
/// ```no_run
/// use kivact::prelude::*;
/// use kivact::ui::{Command, command_palette};
///
/// fn palette(open: Signal<bool>, dark: Signal<bool>) -> impl IntoView {
///     let commands = vec![
///         Command::new("theme", "Toggle theme").group("Actions").hint("Ctrl T").icon("moon"),
///         Command::new("quit", "Quit"),
///     ];
///     command_palette(open, commands, move |key| {
///         if key == "theme" {
///             dark.update(|dark| *dark = !*dark);
///         }
///     })
///     .close_on(|key| key.modifiers().ctrl && key.key() == Key::Char('k'))
/// }
/// ```
#[track_caller]
pub fn command_palette<K: Clone + Eq + Hash + 'static, R: HandlerResult>(
    open: impl Into<Binding<bool>>,
    commands: impl IntoValue<Vec<Command<K>>>,
    run: impl Fn(K) -> R + 'static,
) -> CommandPalette<K> {
    CommandPalette {
        open: open.into(),
        commands: commands.into_value(),
        run: Rc::new(move |key| run(key).into_outcome().map(|_| ())),
        filter: None,
        close_on: None,
        placeholder: None,
        empty: None,
        footer: None,
        variants: Variants::default(),
    }
}

impl<K: Clone + Eq + Hash + 'static> CommandPalette<K> {
    /// Give the component a [`Variant`]: a look beyond the meanings its
    /// state carries, which every recipe of the component reads with
    /// [`Look::variant`](crate::Look::variant). A later value of the same
    /// type replaces the earlier one; a theme that does not know the type
    /// draws the plain look.
    ///
    /// The value is a constant: the recipes read the values given before
    /// the component is mounted, and a value never changes afterwards.
    pub fn variant(self, v: impl Variant) -> Self {
        self.variants.add(v);
        self
    }

    /// Choose which commands the query shows. It receives the query as
    /// typed and a command, and reads the command's reactive texts with
    /// `get()`. The default is
    /// [`filter_commands`](kivact_interaction::filter_commands): the query is
    /// part of the label, the group, the hint or a keyword.
    pub fn filter(mut self, filter: impl Fn(&str, &Command<K>) -> bool + 'static) -> Self {
        self.filter = Some(Rc::new(filter));
        self
    }

    /// Close the palette on a key that `matches`, the way Escape does: a
    /// modal keeps the keys to itself, so the shortcut that opens the
    /// palette is given here to toggle it. The function sees every key
    /// pressed in the field.
    pub fn close_on(mut self, matches: fn(&KeyEvent) -> bool) -> Self {
        self.close_on = Some(matches);
        self
    }

    /// The text shown in the empty search field. The default is the message
    /// `palette.placeholder`, "Type a command or search...".
    pub fn placeholder(mut self, text: impl IntoText) -> Self {
        self.placeholder = Some(text.into_text_value());
        self
    }

    /// The text shown when no command matches. The default is the message
    /// `palette.empty`, "No results found.".
    pub fn empty_text(mut self, text: impl IntoText) -> Self {
        self.empty = Some(text.into_text_value());
        self
    }

    /// A line under the list, such as the keys that move, run and close.
    /// `content` runs each time the palette opens. Without one nothing is
    /// drawn there.
    pub fn footer<V: IntoView>(mut self, content: impl Fn() -> V + 'static) -> Self {
        self.footer = Some(Rc::new(move || content().into_view()));
        self
    }
}

impl<K: Clone + Eq + Hash + 'static> IntoView for CommandPalette<K> {
    fn into_view(self) -> View {
        let CommandPalette {
            open,
            commands,
            run,
            filter,
            close_on,
            placeholder,
            empty,
            footer,
            variants,
        } = self;
        let mut palette = use_command_palette(open, commands, move |key| run(key));
        if let Some(filter) = filter {
            palette = palette.filter(move |query, command| filter(query, command));
        }
        if let Some(matches) = close_on {
            palette = palette.close_on(matches);
        }
        let placeholder = placeholder.unwrap_or_else(|| {
            message("palette.placeholder", "Type a command or search...").into_value()
        });
        let empty =
            empty.unwrap_or_else(|| message("palette.empty", "No results found.").into_value());
        palette
            .show(move || {
                let plain = PartState::default;
                let groups = for_each(
                    palette.visible(),
                    |group| (group.index, group.heading.is_some()),
                    {
                        let variants = variants.clone();
                        move |(index, _), group| {
                            let heading = group.with_untracked(|group| group.heading.clone());
                            let commands = memo(move || group.with(|group| group.commands.clone()));
                            let rows = for_each(commands, |command| command.key().clone(), {
                                let variants = variants.clone();
                                move |key, command| row_view(palette, key, command, &variants)
                            });
                            let Some(heading) = heading else {
                                return rows.into_view();
                            };
                            column((
                                with_recipe_style(
                                    text(move || heading.get()),
                                    |r| r.command_palette_recipes().group_recipe(),
                                    &variants,
                                    plain,
                                ),
                                rows,
                            ))
                            .part(palette.group(index))
                            .into_view()
                        }
                    },
                );
                let field = with_recipe_style(
                    node()
                        .part(
                            use_text_edit(palette.query())
                                .placeholder(placeholder)
                                .control(),
                        )
                        .part(palette.input()),
                    |r| r.command_palette_recipes().field_recipe(),
                    &variants,
                    plain,
                );
                // The magnifier is drawn while the theme has an image for it.
                let mark = current_theme().icons().get("search").map(|_| {
                    with_recipe_style(
                        icon(Icon::named("search")),
                        |r| r.command_palette_recipes().search_icon_recipe(),
                        &variants,
                        plain,
                    )
                });
                let search = with_recipe_style(
                    row((mark, field)),
                    |r| r.command_palette_recipes().search_recipe(),
                    &variants,
                    plain,
                );
                let list = with_recipe_style(
                    scroll(column(groups).part(palette.list())),
                    |r| r.command_palette_recipes().list_recipe(),
                    &variants,
                    plain,
                );
                let nothing = show(move || palette.visible().with(Vec::is_empty), {
                    let variants = variants.clone();
                    move || {
                        with_recipe_style(
                            text(empty),
                            |r| r.command_palette_recipes().empty_recipe(),
                            &variants,
                            plain,
                        )
                    }
                });
                let footer = footer.as_ref().map(|footer| {
                    with_recipe_style(
                        row(footer()),
                        |r| r.command_palette_recipes().footer_recipe(),
                        &variants,
                        plain,
                    )
                });
                let content = palette.content();
                let state = content.state();
                let panel = with_recipe_style(
                    column((search, list, nothing, footer)),
                    |r| r.command_palette_recipes().panel_recipe(),
                    &variants,
                    move || state.get(),
                );
                (
                    with_recipe_style(
                        node(),
                        |r| r.command_palette_recipes().scrim_recipe(),
                        &variants,
                        plain,
                    ),
                    overlay(node_ref(), Placement::Center, panel).part(content),
                )
            })
            .into_view()
    }
}

/// One command's row: its icon, its label and its hint.
fn row_view<K: Clone + Eq + Hash + 'static>(
    palette: kivact_interaction::CommandPalette<K>,
    key: K,
    command: Value<Command<K>>,
    variants: &Variants,
) -> View {
    let part = palette.item(key);
    let state = part.state();
    let active = memo(move || state.get().active);
    let mark = dynamic(
        move || command.with(|command| command.icon_value().cloned()),
        {
            let variants = variants.clone();
            move |icon_of: Option<Icon>| {
                icon_of.map(|icon_of| {
                    with_recipe_style(
                        icon(icon_of),
                        |r| r.command_palette_recipes().icon_recipe(),
                        &variants,
                        move || state.get(),
                    )
                })
            }
        },
    );
    let label =
        text(move || command.with(|command| command.label())).style(style().grow(1).min_width(0));
    let hint = dynamic(move || command.with(|command| command.hint_value()), {
        let variants = variants.clone();
        move |hint: Option<String>| {
            hint.map(|hint| {
                with_recipe_style(
                    text(hint),
                    |r| r.command_palette_recipes().hint_recipe(),
                    &variants,
                    move || state.get(),
                )
            })
        }
    });
    with_recipe_style(
        node().part(part).selected(active),
        |r| r.command_palette_recipes().row_recipe(),
        variants,
        move || state.get(),
    )
    .child((mark, label, hint))
    .into_view()
}
