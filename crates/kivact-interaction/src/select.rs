//! Role: A select attaches a combo box trigger and its list box to caller nodes.
//!
//! Unstyled behavior and semantics of a select: a combo box that opens a
//! list box of keyed options.
//!
//! The closed trigger and the open list are two [`List`]s over
//! the same options with the listbox policy, sharing one private typeahead
//! slot for the select's lifetime, so a letter typed at the trigger and one
//! typed in the list continue the same search. The closed trigger can select
//! options that are not mounted; there is no public raw input dispatch for
//! it. Choosing uses the list's private mounted selection path, and the
//! focus after closing is the popup's (see `crate::popup`). Row count, scroll
//! height and the selected mark are the caller's look; the standard
//! `kivact::ui::select` makes its list at least as wide as the trigger
//! (`at_least_anchor_width`).

use crate::list::typeahead::Typeahead;
use kivact_core::primitive::Part;
use kivact_core::view::HandlerResult;
use std::hash::Hash;

use super::{
    List, ListItem, ListItemState, ListPolicy, ListSelection, Popup, PopupCloseReason, PopupState,
    use_list, use_popup,
};
use kivact_core::i18n::message;
use kivact_core::view::{
    Handled, IntoText, IntoValue, Key, KeyEvent, PointerEvent, PointerPhase, Role, callback,
};
use kivact_core::{
    Binding, Callback, Signal, Value, Var, effect, memo, on_cleanup, on_mount, signal, untrack, var,
};

/// A select over keyed [`ListItem`]s, without a look of its own.
///
/// `SelectApi` adds the behavior and semantics of a combo box and its list box
/// to nodes the caller draws: [`trigger`](Self::trigger) is the combo box,
/// [`content`](Self::content) the caller's `overlay(..)` holding the list, and
/// [`item`](Self::item) each row, rendered with `for_each` over the
/// options while [`shown`](Self::shown) is true. [`label`](Self::label) is what
/// the trigger shows. The selection is the caller's `Binding<Option<K>>`; the
/// select holds no state besides whether the list is open. A selected key
/// that is not among the options (filtered out, not loaded yet) stays
/// selected, and the trigger shows the placeholder until the option returns.
///
/// Keys at the closed trigger: Up and Down select the previous and next
/// enabled option and stop at the ends, Home and End select the first and the
/// last, and a letter key selects the next enabled option whose label starts
/// with it (letters typed within 500 ms of each other are matched together),
/// all without opening the list. Alt+Down, Alt+Up, F4, Enter, Space, a click
/// and an assistive technology's Activate or Expand open it; a click or Escape
/// closes it again.
///
/// The open list takes the focus on the selected option, or else on the first
/// enabled one. Up, Down, Home, End and letter keys move the focus only; Enter,
/// Space and a click choose the focused option, close the list and give the
/// focus back to the trigger. Escape and Alt+Up close it without choosing, and
/// so does Tab, which moves the focus on from the list as usual. The focused
/// option is scrolled into view; how many rows show before the list scrolls
/// is the caller's layout. Disabled options are skipped by every key and
/// cannot be chosen. If the binding rejects or fails the write, the error is
/// reported and the list closes as for any choice.
///
/// Semantics: the trigger is a `Role::ComboBox` with the label as its value,
/// `expanded` and `controls` the list; the surface is a `Role::ListBox`, which
/// the caller names like the trigger; each option is a `Role::Option` named by
/// its label, `selected` when it is the selection.
///
/// The placeholder is the message `select.placeholder` (default `Select…`)
/// unless [`placeholder`](Self::placeholder) gives another text.
///
/// [`on_preview`](Self::on_preview) lets the application try an option out
/// while the list is open, such as showing a theme before it is chosen.
///
/// ```no_run
/// # mod kivact {
/// #     //! Role: Supply facade paths for this documentation example.
/// #     pub use kivact_core::{prelude, view};
/// #     pub extern crate kivact_interaction as interaction;
/// # }
/// use kivact::interaction::{ListItem, use_select};
/// use kivact_core::primitive::NodeExt;
/// use kivact::prelude::*;
/// use kivact::view::{Placement, overlay};
///
/// fn size_picker(size: Signal<Option<&'static str>>) -> impl IntoView {
///     let options = vec![
///         ListItem::new("s", "Small"),
///         ListItem::new("m", "Medium"),
///         ListItem::new("l", "Large").disabled(true),
///     ];
///     let sizes = use_select(options.clone(), size);
///     let trigger = sizes.trigger();
///     let anchor = trigger.node_ref();
///     (
///         node().part(trigger).name("Size").child(text(sizes.label())),
///         show(sizes.shown(), move || {
///             let rows = for_each(options.clone(), |o| *o.key(), move |key, o| {
///                 node()
///                     .part(sizes.item(key))
///                     .child(text(move || o.with(|o| o.label())))
///             });
///             overlay(anchor, Placement::Below, column(rows))
///                 .part(sizes.content())
///                 .name("Size")
///         }),
///     )
/// }
/// ```
pub struct SelectApi<K: Clone + Eq + Hash + 'static> {
    options: Value<Vec<ListItem<K>>>,
    selected: Binding<Option<K>>,
    /// What the trigger shows while nothing is selected; read tracked.
    placeholder: Signal<Value<String>>,
    /// Reports the option being tried out; read when the surface binds.
    preview: Var<Option<Callback<Option<K>>>>,
    popup: Popup,
    /// Moves the selection while the list is closed.
    closed: List<K>,
    arrow_open: bool,
    retain_trigger_focus: bool,
    arrow_edge: Signal<Option<Key>>,
    /// The open list's options.
    list: List<K>,
    /// The option the pointer rests on, for the preview.
    hovered: Signal<Option<K>>,
}

impl<K: Clone + Eq + Hash + 'static> Clone for SelectApi<K> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<K: Clone + Eq + Hash + 'static> Copy for SelectApi<K> {}

/// A select over `options` whose choice is written to `value`. Setup only.
#[track_caller]
pub fn use_select<K: Clone + Eq + Hash + 'static>(
    options: impl IntoValue<Vec<ListItem<K>>>,
    value: impl Into<Binding<Option<K>>>,
) -> SelectApi<K> {
    let options = options.into_value();
    let selected = value.into();
    let open = signal(false);
    let typed = var(Typeahead::default());
    let hovered = signal(None::<K>);
    effect(move || {
        if !open.get() {
            hovered.set(None);
        }
    });
    SelectApi {
        options,
        selected,
        placeholder: signal(message("select.placeholder", "Select…")),
        preview: var(None),
        popup: use_popup(open),
        // The closed trigger moves the selection through the logical order,
        // including options that are not mounted.
        closed: group(options, selected).shared_typeahead(typed),
        arrow_open: false,
        retain_trigger_focus: false,
        arrow_edge: signal(None),
        list: group(options, selected).shared_typeahead(typed),
        hovered,
    }
}

impl<K: Clone + Eq + Hash + 'static> SelectApi<K> {
    /// Open on a bare arrow instead of changing the closed selection.
    ///
    /// ArrowDown focuses the first enabled option and ArrowUp focuses the last.
    /// The selected value changes only when the user commits an option.
    pub fn open_on_arrow(mut self, enabled: bool) -> Self {
        self.arrow_open = enabled;
        self
    }

    /// Keep physical focus on the trigger while the open list highlights options.
    /// The trigger publishes the active mounted option as its active descendant.
    /// Configure before mounting the trigger or content.
    pub fn retain_trigger_focus(mut self, enabled: bool) -> Self {
        self.retain_trigger_focus = enabled;
        self.list.retain_focus(enabled);
        self.popup = self.popup.anchor_starts_focus_range(enabled);
        self
    }

    /// What the trigger shows while no option is selected. Setup only.
    #[track_caller]
    pub fn placeholder(self, text: impl IntoText) -> Self {
        let text = text.into_text_value();
        self.placeholder.update(|placeholder| *placeholder = text);
        self
    }

    /// Report the option the user is trying out while the list is open, so the
    /// application can show it before it is chosen.
    ///
    /// `f` receives `Some(key)` for the enabled option that has the focus, or
    /// that the pointer rests on (the pointer wins while it is over a row), and
    /// again each time that option changes. When the list closes it receives
    /// `None`, whether an option was chosen or not; a choice is written to the
    /// selection first. A preview never selects: the selection is still only
    /// what the binding receives. Setup only.
    ///
    /// ```no_run
    /// # mod kivact {
    /// #     //! Role: Supply facade paths for this documentation example.
    /// #     pub use kivact_core::{prelude, view};
    /// #     pub extern crate kivact_interaction as interaction;
    /// # }
    /// use kivact::interaction::{ListItem, use_select};
    /// use kivact::prelude::*;
    ///
    /// fn theme_picker(chosen: Signal<Option<&'static str>>, shown: Signal<&'static str>) {
    ///     let options = vec![ListItem::new("dark", "Dark"), ListItem::new("light", "Light")];
    ///     let themes = use_select(options, chosen).on_preview(move |key| {
    ///         shown.set(key.or(chosen.get_untracked()).unwrap_or("dark"));
    ///     });
    ///     let _ = themes;
    /// }
    /// ```
    pub fn on_preview<R: HandlerResult>(self, f: impl Fn(Option<K>) -> R + 'static) -> Self {
        let f = callback(f);
        self.preview.set(Some(f));
        self
    }

    /// The label of the selected option, or the placeholder while none is
    /// selected (or the selected key is not among the options). The caller
    /// puts it in the trigger; it is also the trigger's accessible value.
    /// Setup only.
    #[track_caller]
    pub fn label(self) -> Value<String> {
        let (options, selected, placeholder) =
            (self.options, self.selected.into_value(), self.placeholder);
        memo(move || {
            selected.with(|key| {
                options.with(|rows| {
                    key.as_ref()
                        .and_then(|key| rows.iter().find(|row| &row.key == key))
                        .map_or_else(
                            || placeholder.with(|text| text.get()),
                            |row| row.label.get(),
                        )
                })
            })
        })
    }

    /// Whether the list is open. The caller mounts the content while it is
    /// true, and may draw the trigger with it, such as a chevron that flips.
    pub fn shown(self) -> Value<bool> {
        self.popup.shown()
    }

    /// The binding shared with an alternative select surface.
    pub fn open_binding(self) -> Binding<bool> {
        self.popup.open_binding()
    }

    /// Activate the trigger using the ordinary popup's toggle policy.
    pub fn toggle(self) {
        if self.shown().get_untracked() {
            self.popup.close(PopupCloseReason::Programmatic);
        } else {
            self.open_binding().set(true);
        }
    }

    /// The combo box: a focusable `Role::ComboBox` that publishes the label
    /// as its value, `expanded` and the list it `controls`, and opens the
    /// list. Its key and activation handling runs after the caller's own
    /// handlers. The part's `node_ref` is the anchor for the surface's
    /// overlay and where the focus returns. Setup only.
    #[track_caller]
    pub fn trigger(self) -> Part<PopupState> {
        let label = self.label();
        let popup = self.popup;
        let closed = self.closed;
        let (arrow_open, arrow_edge) = (self.arrow_open, self.arrow_edge);
        let retained = self.retain_trigger_focus;
        let trigger = popup
            .trigger()
            .role(Role::ComboBox)
            .value_text(label)
            .on_key(move |event: KeyEvent| {
                let m = event.modifiers();
                if popup.shown().get() {
                    // The focus is back on the trigger while the list is open
                    // (Shift+Tab, or a click on it): the list's keys apply.
                    return if event.key() == Key::Escape || is_alt_up(&event) {
                        popup.close(PopupCloseReason::Escape);
                        Handled::Yes
                    } else if retained {
                        retained_key(self.list, popup, event)
                    } else {
                        Handled::No
                    };
                }
                let alt_arrow = m.alt
                    && !m.ctrl
                    && !m.meta
                    && matches!(event.key(), Key::ArrowDown | Key::ArrowUp);
                let f4 = event.key() == Key::F(4) && !m.ctrl && !m.alt && !m.meta;
                if alt_arrow || f4 {
                    popup.open_binding().set(true);
                    return Handled::Yes;
                }
                if m.ctrl || m.alt || m.meta {
                    return Handled::No;
                }
                if arrow_open && !m.shift && matches!(event.key(), Key::ArrowDown | Key::ArrowUp) {
                    arrow_edge.set(Some(event.key()));
                    popup.open_binding().set(true);
                    return Handled::Yes;
                }
                if retained
                    && matches!(
                        event.key(),
                        Key::Home | Key::End | Key::PageUp | Key::PageDown
                    )
                {
                    return Handled::Yes;
                }
                closed.key_selection(&event)
            });
        if retained {
            trigger.active_descendant(move || {
                if popup.shown().get() {
                    self.list.active_ref()
                } else {
                    None
                }
            })
        } else {
            trigger
        }
    }

    /// The list's surface, for the caller's `overlay(..)` anchored to the
    /// trigger: a `Role::ListBox` connected to the popup's dismissal.
    ///
    /// On each opening it focuses the selected option, or else the first
    /// enabled one, and starts the preview, which ends with the surface.
    /// Escape and Alt+Up close the list, and so does the release of Tab. The
    /// name and placement stay the caller's. Setup only.
    #[track_caller]
    pub fn content(self) -> Part<PopupState> {
        self.content_in(false)
    }

    /// The listbox inside a modal surface that handles its own dismissal.
    /// Its options take physical focus and are rendered with [`Self::sheet_item`].
    #[track_caller]
    pub fn sheet_content(self) -> Part<PopupState> {
        Self {
            retain_trigger_focus: false,
            ..self
        }
        .content_in(true)
    }

    fn content_in(self, sheet: bool) -> Part<PopupState> {
        let SelectApi {
            selected,
            options,
            preview,
            popup,
            list,
            hovered,
            arrow_edge,
            retain_trigger_focus,
            ..
        } = self;
        list.retain_focus(retain_trigger_focus);
        let content = if sheet {
            let shown = popup.shown();
            kivact_core::primitive::part(move |input| PopupState {
                input,
                open: shown.get(),
            })
            .reference(popup.content_ref())
        } else {
            popup.content()
        };
        let content = content
            .role(Role::ListBox)
            .on_key(move |event: KeyEvent| {
                if !sheet && (event.key() == Key::Escape || is_alt_up(&event)) {
                    popup.close(PopupCloseReason::Escape);
                    return Handled::Yes;
                }
                Handled::No
            })
            // The host consumes Tab to move focus, so the list sees only its
            // release, aimed at the option that has the focus now.
            .on_key_up(move |event: KeyEvent| {
                if !sheet && event.key() == Key::Tab {
                    popup.close(PopupCloseReason::Tab);
                    Handled::Yes
                } else {
                    Handled::No
                }
            })
            .on_bind(move |_| {
                if let Some(preview) = preview.get() {
                    track_preview(list, hovered, preview);
                }
                if retain_trigger_focus {
                    on_cleanup(move || list.reset_highlight());
                }
                let anchor = popup.anchor_ref();
                let anchor_focused = anchor.focused();
                on_mount(move || {
                    if retain_trigger_focus && !anchor_focused.get() {
                        anchor.restore_focus();
                    }
                    let edge = arrow_edge.get_untracked();
                    if let Some(edge) = edge {
                        arrow_edge.set(None);
                        let key = options.with(|options| {
                            let mut enabled = options.iter().filter(|option| !option.disabled);
                            let option = if edge == Key::ArrowUp {
                                enabled.next_back()
                            } else {
                                enabled.next()
                            };
                            option.map(|option| option.key.clone())
                        });
                        if let Some(key) = key {
                            list.focus(key);
                        }
                        return;
                    }
                    if selected.get().is_some_and(|key| list.focus(key)) {
                        return;
                    }
                    if let Some(key) = options.with(|options| {
                        options
                            .iter()
                            .find(|option| !option.disabled)
                            .map(|option| option.key.clone())
                    }) {
                        list.focus(key);
                    }
                });
            });
        if retain_trigger_focus {
            content.focus_on_press(popup.anchor_ref())
        } else {
            content
        }
    }

    /// The option `key` of the open list: a `Role::Option` named by its
    /// label and `selected` when it is the selection. Activation chooses it
    /// and closes the list; the focused option is scrolled into view. Setup
    /// only, in the option's row.
    #[track_caller]
    pub fn item(self, key: K) -> Part<ListItemState> {
        self.item_in(key, false)
    }

    /// An option in the modal listbox; choosing it closes that surface.
    #[track_caller]
    pub fn sheet_item(self, key: K) -> Part<ListItemState> {
        Self {
            retain_trigger_focus: false,
            ..self
        }
        .item_in(key, true)
    }

    fn item_in(self, key: K, sheet: bool) -> Part<ListItemState> {
        let SelectApi {
            popup,
            list,
            hovered,
            retain_trigger_focus,
            ..
        } = self;
        let member = list.item(key.clone());
        let option_ref = member.node_ref();
        let hover_key = key.clone();
        let invoke_key = key;
        member
            .on_press(move || {
                if list.select_mounted(invoke_key.clone()) {
                    if sheet {
                        popup.open_binding().set(false);
                    } else {
                        popup.close(PopupCloseReason::Selection);
                    }
                }
            })
            .on_pointer(move |event: PointerEvent| {
                match event.phase() {
                    PointerPhase::Enter | PointerPhase::Move => {
                        if retain_trigger_focus {
                            list.highlight_mounted(hover_key.clone());
                        } else {
                            hovered.set(Some(hover_key.clone()));
                        }
                    }
                    PointerPhase::Leave => hovered.update(|slot| {
                        if slot.as_ref() == Some(&hover_key) {
                            *slot = None;
                        }
                    }),
                    _ => {}
                }
                Handled::No
            })
            .on_bind(move |_| {
                let focused = option_ref.focused();
                effect(move || {
                    if focused.get() {
                        option_ref.scroll_into_view();
                    }
                });
            })
    }
}

fn retained_key<K: Clone + Eq + Hash + 'static>(
    list: List<K>,
    popup: Popup,
    event: KeyEvent,
) -> Handled {
    let modifiers = event.modifiers();
    if modifiers.ctrl || modifiers.alt || modifiers.meta {
        return Handled::No;
    }
    let active = list.active().get();
    let command = match event.key() {
        Key::Enter | Key::Space if !modifiers.shift => {
            // Consume even an empty list or a repeat. Otherwise the core's
            // default Activate would reach the trigger and toggle the popup.
            if !event.repeat() && active.is_some_and(|key| list.select_mounted(key)) {
                popup.close(PopupCloseReason::Selection);
            }
            return Handled::Yes;
        }
        Key::PageUp => Key::Home,
        Key::PageDown => Key::End,
        key => key,
    };
    match command {
        Key::ArrowDown | Key::ArrowUp | Key::Home | Key::End => {
            if let Some(key) = active {
                list.route_focused_key(&key, event, Some(command));
            }
            Handled::Yes
        }
        Key::Char(_) => active.map_or(Handled::No, |key| {
            list.route_focused_key(&key, event, Some(command))
        }),
        _ => Handled::No,
    }
}

fn is_alt_up(event: &KeyEvent) -> bool {
    let m = event.modifiers();
    event.key() == Key::ArrowUp && m.alt && !m.ctrl && !m.meta
}

/// A group over the options with the listbox policy: Up and Down stop at the
/// ends and moving never selects on its own.
fn group<K: Clone + Eq + Hash + 'static>(
    options: Value<Vec<ListItem<K>>>,
    selected: Binding<Option<K>>,
) -> List<K> {
    use_list(
        options,
        ListSelection::Single(selected),
        ListPolicy::listbox(),
    )
}

/// Report the previewed option to `preview` while the list is open: the row
/// the pointer rests on, else the focused one. The list closing ends the
/// preview with `None`. Setup only, under the mounted surface's owner.
fn track_preview<K: Clone + Eq + Hash + 'static>(
    list: List<K>,
    hovered: Signal<Option<K>>,
    preview: Callback<Option<K>>,
) {
    let active = list.active();
    let reported = var(None::<K>);
    effect(move || {
        let key = hovered.get().or_else(|| active.get());
        untrack(|| {
            if reported.with(|last| *last != key) {
                reported.set(key.clone());
                preview.call(key);
            }
        });
    });
    on_cleanup(move || {
        if reported.with(Option::is_some) {
            reported.set(None);
            preview.call(None);
        }
    });
}
