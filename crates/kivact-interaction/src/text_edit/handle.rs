//! Role: The `TextEditApi` handle: native editing options and the control part.

use super::draft::{self, Draft};
use super::policy::text_edit_policy;
use crate::options_cell::OptionsCell;
use kivact_core::ErrorKind;
use kivact_core::state::{MachineHandle, machine, memo};

use super::machine;
use kivact_core::custom::{
    EditLineLayout, NodeRefExt, Owned, Owner, Part, PartState, TextEditing, part, raise,
    require_setup,
};
use kivact_core::state::{Binding, Signal, Value, on_cleanup, signal};
use kivact_core::state::{Callback, callback};
use kivact_core::state::{IntoText, IntoValue};
use kivact_core::view::{Composition as CompositionState, EditPhase, NodeRef, node_ref};
use kivact_core::view::{Handled, HandlerResult, Key, Role};
use kivact_types::cell::Generation;
use std::cell::Cell;
use std::rc::Rc;

/// Headless controlled editing attached to one native leaf at a time.
///
/// One line or several is chosen by the constructor ([`use_text_edit`],
/// [`use_text_area`]); every other setting may be given before or after
/// `control()` and reaches the mounted editor. A disposed control may be
/// attached again; simultaneous controls report a diagnostic.
#[derive(Clone, Copy)]
pub struct TextEditApi {
    data: Owned<Rc<Data>>,
}

#[derive(Clone)]
enum InputDraft {
    Immediate,
    Drafting(draft::Installed),
}

/// Whether Enter submits or inserts a line break.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Lines {
    Single,
    Multiple,
}

/// Create native editing behavior for an application-owned string binding,
/// one line: Enter submits ([`TextEditApi::on_submit`]).
///
/// Preedit remains in the native editor; the binding receives committed
/// values. Changes are observed through the binding: pass
/// `binding(read, write)` to run more code on each committed value. A writer
/// that rejects a value restores its own value.
/// Each committed edit copies the whole value into the binding; an editor
/// for long documents takes range edits from `TextEditing` instead. Setup only.
#[track_caller]
pub fn use_text_edit(value: impl kivact_core::state::IntoBinding<String>) -> TextEditApi {
    create(value.into_binding(), Lines::Single)
}

/// Create native editing behavior for an application-owned string binding,
/// several lines: Enter inserts a line break. Otherwise as [`use_text_edit`].
/// Setup only.
#[track_caller]
pub fn use_text_area(value: impl kivact_core::state::IntoBinding<String>) -> TextEditApi {
    create(value.into_binding(), Lines::Multiple)
}

#[track_caller]
fn create(binding: Binding<String>, lines: Lines) -> TextEditApi {
    let owner = Owner::current();
    let options = OptionsCell::new(TextEditOptions::default());
    let control = machine(
        machine::State::Unmounted,
        || machine::Machine,
        move |_, effect| match effect {
            machine::Effect::DuplicateMount => owner.report(
                ErrorKind::Message,
                "TextEditApi supports only one mounted control",
            ),
            machine::Effect::Composition(value) => {
                if let Some(callback) = options.get().composition {
                    callback.call(value);
                }
            }
        },
    );
    let data = Rc::new(Data {
        observed: signal(None),
        binding,
        lines,
        options,
        control,
        next_control: Cell::new(Generation::new()),
    });
    TextEditApi {
        data: owner.store("TextEditApi", data),
    }
}

impl TextEditApi {
    /// Drawing state of the actual mounted control.
    pub fn control_state(self) -> Value<PartState> {
        let observed = self.data.get().observed;
        kivact_core::state::memo(move || {
            observed.get().map(|state| state.get()).unwrap_or_default()
        })
    }

    /// Soft-wrap multiline content at the field's content width.
    pub fn wrap(self, on: impl IntoValue<bool>) -> Self {
        let on = on.into_value();
        self.setup("TextEditApi::wrap")
            .options
            .update(|options| options.wrap = Some(on));
        self
    }

    /// Grow a multiline field between these inclusive visual row counts.
    pub fn rows(self, rows: std::ops::RangeInclusive<usize>) -> Self {
        let min = (*rows.start()).max(1);
        let max = (*rows.end()).max(min);
        self.setup("TextEditApi::rows")
            .options
            .update(|options| options.rows = Some((min, max)));
        self
    }

    /// Give handlers ownership of selected control keys; the list may be reactive.
    ///
    /// Tab, arrows, Home, End, PageUp, PageDown, Escape and Enter can be forwarded.
    /// Characters and Space are ignored. Ctrl/Alt/Meta keep native behavior;
    /// Shift forwards only Tab. IME composition always consumes keys first.
    /// The default is empty, preserving native caret and Tab focus navigation.
    /// Forwarded keys have no native fallback when a handler returns `Handled::No`.
    ///
    /// The forwarded keys reach the ordinary key handlers before native
    /// editing does. Only unmodified keys and Shift+Tab qualify; characters,
    /// Space and keys during IME composition are never forwarded. A reactive
    /// list keeps applying its changes after the mount. Creative's
    /// multi-column picker, for example, forwards only Tab, ArrowLeft and
    /// ArrowRight, while a single-column picker leaves Tab to close its popup.
    #[track_caller]
    pub fn forward_keys(self, keys: impl IntoValue<Vec<Key>>) -> Self {
        let data = self.setup("TextEditApi::forward_keys");
        let keys = keys.into_value();
        data.options
            .update(|options| options.forward_keys = Some(keys));
        self
    }

    /// The editor's data, for a setter that is called during setup.
    #[track_caller]
    fn setup(self, api: &'static str) -> Rc<Data> {
        require_setup(api);
        self.data.get()
    }

    /// Mask the value and omit its text from clipboard and a11y snapshots.
    #[track_caller]
    pub fn password(self, value: impl IntoValue<bool>) -> Self {
        let data = self.setup("TextEditApi::password");
        let value = value.into_value();
        data.options
            .update(|options| options.password = Some(value));
        self
    }

    /// Permit selection while rejecting user edits and new compositions.
    #[track_caller]
    pub fn read_only(self, value: impl IntoValue<bool>) -> Self {
        let data = self.setup("TextEditApi::read_only");
        let value = value.into_value();
        data.options
            .update(|options| options.read_only = Some(value));
        self
    }

    /// Limit committed text by Unicode scalar count; preedit is not truncated.
    #[track_caller]
    pub fn max_length(self, value: impl IntoValue<usize>) -> Self {
        let data = self.setup("TextEditApi::max_length");
        let value = value.into_value();
        data.options
            .update(|options| options.max_length = Some(value));
        self
    }

    /// Show hint text while the editor is empty.
    #[track_caller]
    pub fn placeholder(self, value: impl IntoText) -> Self {
        let data = self.setup("TextEditApi::placeholder");
        let value = value.into_text_value();
        data.options
            .update(|options| options.placeholder = Some(value));
        self
    }

    /// Handle Enter in a single-line editor after native composition routing.
    /// A text area ([`use_text_area`]) takes Enter as a line break, so giving
    /// it a submit handler is a usage error.
    #[track_caller]
    pub fn on_submit(self, f: impl Into<Callback<()>>) -> Self {
        let data = self.setup("TextEditApi::on_submit");
        if data.lines == Lines::Multiple {
            raise(
                ErrorKind::Message,
                "TextEditApi::on_submit: a text area takes Enter as a line break",
            );
        }
        let f = f.into();
        data.options.update(|options| options.submit = Some(f));
        self
    }

    /// Commit a draft on Enter or a valid changed blur; Escape restores it.
    pub fn on_draft_commit<R: HandlerResult>(self, f: impl Fn(String) -> R + 'static) -> Self {
        self.setup("TextEditApi::on_draft_commit")
            .options
            .update(|options| options.draft_commit = Some(callback(f)));
        self
    }

    /// Observe native value edits, or the complete draft when drafting.
    pub fn on_edit<R: HandlerResult>(self, f: impl Fn(EditPhase) -> R + 'static) -> Self {
        self.setup("TextEditApi::on_edit")
            .options
            .update(|options| options.draft_edit = Some(callback(f)));
        self
    }

    /// Return focus after Enter or Escape ends the draft.
    pub fn return_focus(self, target: NodeRef) -> Self {
        self.setup("TextEditApi::return_focus")
            .options
            .update(|options| options.return_focus = Some(target));
        self
    }

    pub(crate) fn inline(self, label: impl IntoText, ended: Callback<()>) -> Self {
        let data = self.setup("TextEditApi::inline");
        let label = label.into_text_value();
        data.options.update(|options| {
            options.label = Some(label);
            options.draft_end = Some(ended);
            options.auto_focus = true;
        });
        self
    }

    /// Observe transitions into and out of native composition.
    #[track_caller]
    pub fn on_composition_change<F, R>(self, f: F) -> Self
    where
        F: Fn(CompositionState) -> R + 'static,
        R: HandlerResult,
    {
        let data = self.setup("TextEditApi::on_composition_change");
        data.options.update(|options| {
            options.composition = Some(callback(move |value| {
                f(if value {
                    CompositionState::Composing
                } else {
                    CompositionState::Idle
                })
            }))
        });
        self
    }
    pub(crate) fn draft_session(self, session: crate::EditSession<String>) -> Self {
        self.setup("TextEditApi::draft_session")
            .options
            .update(|options| options.draft_session = Some(session));
        self
    }
    /// Create the focusable TextBox Part for this editor's native leaf.
    #[track_caller]
    pub fn control(self) -> Part<PartState> {
        self.data.get();
        require_setup("TextEditApi::control");
        let input = part(|state| state);
        let state = input.state();
        self.data.get().observed.set(Some(state));
        let node = node_ref();
        let options = self.data.get().options.value();
        let line_layout = memo(move || {
            let options = options.get();
            EditLineLayout {
                wrap: options.wrap.is_some_and(|value| value.get()),
                rows: options.rows,
                ..EditLineLayout::default()
            }
        });
        Part::new(state, move |host| {
            let mut control = host
                .part(input)
                .node_ref(node)
                .role(Role::TextBox)
                .focusable(true);
            let data = self.data.get();
            if let Some(label) = data.options.get().label {
                control = control.name(label);
            }
            let (control, input_draft) = if let Some(commit) = data.options.get().draft_commit {
                let (control, installed) = draft::install(
                    Draft {
                        commit,
                        on_edit: data.options.get().draft_edit,
                        return_focus: data.options.get().return_focus,
                        ended: data.options.get().draft_end,
                        session: data.options.get().draft_session,
                    },
                    data.binding,
                    control,
                );
                (control, InputDraft::Drafting(installed))
            } else {
                (control, InputDraft::Immediate)
            };
            let draft_submit = input_draft.clone();
            let owned = self.data;
            let (binding, lines) = {
                let data = owned.get();
                (data.binding, data.lines)
            };
            // Only the control mounted now writes; each control has its own id.
            let id = {
                let data = owned.get();
                let mut next = data.next_control.get();
                let id = next.next();
                data.next_control.set(next);
                id
            };
            let value = binding.into_value();
            let editing = TextEditing::new(move || value.get())
                .on_edit(move |phase| {
                    let data = owned.get();
                    if data.options.get().draft_commit.is_none()
                        && let Some(observer) = data.options.get().draft_edit
                    {
                        observer.call(phase);
                    }
                })
                .report_text()
                .behavior(text_edit_policy)
                .forward_keys(move || {
                    owned
                        .get()
                        .options
                        .value()
                        .get()
                        .forward_keys
                        .map_or_else(Vec::new, |keys| keys.get())
                })
                .line_layout(line_layout)
                .password(move || {
                    owned
                        .get()
                        .options
                        .value()
                        .get()
                        .password
                        .is_some_and(|value| value.get())
                })
                .read_only(move || {
                    owned
                        .get()
                        .options
                        .value()
                        .get()
                        .read_only
                        .is_some_and(|value| value.get())
                })
                .max_length(move || {
                    owned
                        .get()
                        .options
                        .value()
                        .get()
                        .max_length
                        .map(|limit| limit.get())
                })
                .placeholder(move || {
                    owned
                        .get()
                        .options
                        .value()
                        .get()
                        .placeholder
                        .map_or_else(String::new, |value| value.get())
                })
                .on_text_change(move |edit| {
                    let data = owned.get();
                    if !data.is_mounted(id) {
                        return Ok(());
                    }
                    let composing = edit.composition().is_some();
                    if composing {
                        data.control
                            .send(machine::Event::Composition { id, composing });
                        return Ok(());
                    }
                    let edited = edit.text.unwrap_or_default();
                    if let InputDraft::Drafting(installed) = &input_draft {
                        (installed.edited)();
                    }
                    binding.set(edited.clone());
                    data.control.send(machine::Event::Composition {
                        id,
                        composing: false,
                    });
                    if let Some(value) = value.try_get()
                        && value != edited
                    {
                        node.set_edit_value(value);
                    }
                    Ok(())
                });
            let editing = if lines == Lines::Multiple {
                editing.multiline()
            } else {
                editing
            };
            control
                .text_editing(editing)
                .on_key(move |event| {
                    let data = owned.get();
                    if lines == Lines::Single && data.is_mounted(id) && event.key() == Key::Enter {
                        if let InputDraft::Drafting(installed) = &draft_submit {
                            (installed.submit)();
                            return Ok(Handled::Yes);
                        }
                        let submit = data.options.get().submit;
                        if let Some(submit) = submit {
                            submit.try_call(())?;
                            return Ok(Handled::Yes);
                        }
                    }
                    Ok(Handled::No)
                })
                .on_attach(move |_| {
                    let data = owned.get();
                    data.control.send(machine::Event::Mount(id));
                    if !data.is_mounted(id) {
                        return;
                    }
                    if data.options.get().auto_focus {
                        node.focus();
                        node.select_all();
                    }
                    on_cleanup(move || data.control.send(machine::Event::Unmount(id)));
                })
        })
    }
}

#[derive(Clone, Default)]
struct TextEditOptions {
    draft_commit: Option<Callback<String>>,
    draft_session: Option<crate::EditSession<String>>,
    draft_edit: Option<Callback<EditPhase>>,
    draft_end: Option<Callback<()>>,
    return_focus: Option<NodeRef>,
    label: Option<Value<String>>,
    auto_focus: bool,
    forward_keys: Option<Value<Vec<Key>>>,
    wrap: Option<Value<bool>>,
    rows: Option<(usize, usize)>,
    password: Option<Value<bool>>,
    read_only: Option<Value<bool>>,
    max_length: Option<Value<usize>>,
    placeholder: Option<Value<String>>,
    submit: Option<Callback<()>>,
    composition: Option<Callback<bool>>,
}

struct Data {
    observed: Signal<Option<Value<PartState>>>,
    binding: Binding<String>,
    lines: Lines,
    options: OptionsCell<TextEditOptions>,
    control: MachineHandle<machine::Machine>,
    next_control: Cell<Generation>,
}

impl Data {
    fn is_mounted(&self, id: Generation) -> bool {
        matches!(self.control.state().get_untracked(), machine::State::Mounted { id: current, .. } if current == id)
    }
}
