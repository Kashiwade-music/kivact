//! Role: The `TextEdit` handle: native editing options and the control part.

use super::policy::text_edit_policy;
use kivact_core::diagnostics::ErrorKind;
use kivact_core::primitive::{
    EditLineLayout, NodeRefExt, Owned, Owner, Part, PartState, TextEditing, part, raise,
    require_setup,
};
use kivact_core::view::{Handled, HandlerResult, IntoText, IntoValue, Key, Role};
use kivact_core::{Binding, Result, Signal, Value, on_cleanup, signal};
use std::cell::{Cell, RefCell};
use std::panic::Location;
use std::rc::Rc;

/// Headless controlled editing attached to one native leaf at a time.
///
/// One line or several is chosen by the constructor ([`use_text_edit`],
/// [`use_text_area`]); every other setting may be given before or after
/// `control()` and reaches the mounted editor. A disposed control may be
/// attached again; simultaneous controls report a diagnostic.
#[derive(Clone, Copy)]
pub struct TextEdit {
    data: Owned<Data>,
}

/// Whether Enter submits or inserts a line break.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Lines {
    Single,
    Multiple,
}

/// Create native editing behavior for an application-owned string binding,
/// one line: Enter submits ([`TextEdit::on_submit`]).
///
/// Preedit remains in the native editor; the binding receives committed
/// values. Changes are observed through the binding: pass
/// `binding(read, write)` to run more code on each committed value. A writer
/// that rejects a value restores its own value.
/// Each committed edit copies the whole value into the binding; an editor
/// for long documents takes range edits from `TextEditing` instead. Setup only.
#[track_caller]
pub fn use_text_edit(value: impl Into<Binding<String>>) -> TextEdit {
    create(value.into(), Lines::Single)
}

/// Create native editing behavior for an application-owned string binding,
/// several lines: Enter inserts a line break. Otherwise as [`use_text_edit`].
/// Setup only.
#[track_caller]
pub fn use_text_area(value: impl Into<Binding<String>>) -> TextEdit {
    create(value.into(), Lines::Multiple)
}

#[track_caller]
fn create(binding: Binding<String>, lines: Lines) -> TextEdit {
    let owner = Owner::current("text_edit");
    let data = Rc::new(Data {
        owner,
        binding,
        lines,
        forward_keys: signal(None),
        line_layout: signal(EditLineLayout::default()),
        password: signal(None),
        read_only: signal(None),
        max_length: signal(None),
        placeholder: signal(None),
        submit: RefCell::new(None),
        composition: RefCell::new(None),
        mounted: Cell::new(None),
        next_control: Cell::new(0),
        was_composing: Cell::new(false),
    });
    TextEdit {
        data: owner.store("TextEdit", data),
    }
}

impl TextEdit {
    /// Soft-wrap multiline content at the field's content width.
    pub fn wrap(self, on: bool) -> Self {
        self.setup("TextEdit::wrap")
            .line_layout
            .update(|layout| layout.wrap = on);
        self
    }

    /// Grow a multiline field between these inclusive visual row counts.
    pub fn rows(self, rows: std::ops::RangeInclusive<usize>) -> Self {
        let min = (*rows.start()).max(1);
        let max = (*rows.end()).max(min);
        self.setup("TextEdit::rows")
            .line_layout
            .update(|layout| layout.rows = Some((min, max)));
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
        let data = self.setup("TextEdit::forward_keys");
        let keys = keys.into_value();
        data.forward_keys.update(|slot| *slot = Some(keys));
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
        let data = self.setup("TextEdit::password");
        let value = value.into_value();
        data.password.update(|slot| *slot = Some(value));
        self
    }

    /// Permit selection while rejecting user edits and new compositions.
    #[track_caller]
    pub fn read_only(self, value: impl IntoValue<bool>) -> Self {
        let data = self.setup("TextEdit::read_only");
        let value = value.into_value();
        data.read_only.update(|slot| *slot = Some(value));
        self
    }

    /// Limit committed text by Unicode scalar count; preedit is not truncated.
    #[track_caller]
    pub fn max_length(self, value: impl IntoValue<usize>) -> Self {
        let data = self.setup("TextEdit::max_length");
        let value = value.into_value();
        data.max_length.update(|slot| *slot = Some(value));
        self
    }

    /// Show hint text while the editor is empty.
    #[track_caller]
    pub fn placeholder(self, value: impl IntoText) -> Self {
        let data = self.setup("TextEdit::placeholder");
        let value = value.into_text_value();
        data.placeholder.update(|slot| *slot = Some(value));
        self
    }

    /// Handle Enter in a single-line editor after native composition routing.
    /// A text area ([`use_text_area`]) takes Enter as a line break, so giving
    /// it a submit handler is a usage error.
    #[track_caller]
    pub fn on_submit<F, R>(self, f: F) -> Self
    where
        F: Fn() -> R + 'static,
        R: HandlerResult,
    {
        let data = self.setup("TextEdit::on_submit");
        if data.lines == Lines::Multiple {
            raise(
                ErrorKind::Message,
                "TextEdit::on_submit: a text area (use_text_area) takes Enter as a line break",
                Location::caller(),
            );
        }
        *data.submit.borrow_mut() = Some(Rc::new(move || f().into_outcome().map(|_| ())));
        self
    }

    /// Observe transitions into and out of native composition.
    #[track_caller]
    pub fn on_composition_change<F, R>(self, f: F) -> Self
    where
        F: Fn(bool) -> R + 'static,
        R: HandlerResult,
    {
        let data = self.setup("TextEdit::on_composition_change");
        *data.composition.borrow_mut() =
            Some(Rc::new(move |value| f(value).into_outcome().map(|_| ())));
        self
    }
    /// Create the focusable TextBox Part for this editor's native leaf.
    #[track_caller]
    pub fn control(self) -> Part<PartState> {
        let at = Location::caller();
        self.data.get();
        require_setup("TextEdit::control");
        let control = part(|state| state).role(Role::TextBox).focusable(true);
        let node = control.node_ref();
        let owned = self.data;
        let (binding, lines) = {
            let data = owned.get();
            (data.binding, data.lines)
        };
        // Only the control mounted now writes; each control has its own id.
        let id = {
            let data = owned.get();
            let id = data.next_control.get();
            data.next_control.set(id + 1);
            id
        };
        let value = binding.into_value();
        let editing = TextEditing::new(move || Rc::<str>::from(value.get()))
            .report_text(true)
            .behavior(text_edit_policy)
            .forward_keys(move || {
                owned
                    .get()
                    .forward_keys
                    .with(|keys| keys.map_or_else(Vec::new, |keys| keys.get()))
            })
            .multiline(lines == Lines::Multiple)
            .line_layout(owned.get().line_layout)
            .password(move || {
                owned
                    .get()
                    .password
                    .with(|value| value.is_some_and(|value| value.get()))
            })
            .read_only(move || {
                owned
                    .get()
                    .read_only
                    .with(|value| value.is_some_and(|value| value.get()))
            })
            .max_length(move || {
                owned
                    .get()
                    .max_length
                    .with(|limit| limit.map(|limit| limit.get()))
            })
            .placeholder(move || {
                owned
                    .get()
                    .placeholder
                    .with(|value| value.map_or_else(String::new, |value| value.get()))
            })
            .on_edit(move |edit| {
                let data = owned.get();
                if data.mounted.get() != Some(id) {
                    return Ok(());
                }
                let composing = edit.composition().is_some();
                let composition = (data.was_composing.replace(composing) != composing)
                    .then(|| data.composition.borrow().clone())
                    .flatten();
                if composing {
                    return composition.map_or(Ok(()), |f| f(true));
                }
                let edited = edit.text.unwrap_or_default();
                binding.set(edited.clone());
                let result = composition.map_or(Ok(()), |f| f(false));
                if let Some(value) = value.try_get()
                    && value != edited
                {
                    node.set_edit_value(value);
                }
                result
            });
        control
            .text_editing(editing)
            .on_key(move |event| {
                let data = owned.get();
                if lines == Lines::Single
                    && data.mounted.get() == Some(id)
                    && event.key() == Key::Enter
                {
                    let submit = data.submit.borrow().clone();
                    if let Some(submit) = submit {
                        submit()?;
                        return Ok(Handled::Yes);
                    }
                }
                Ok(Handled::No)
            })
            .on_bind(move |_| {
                let data = owned.get();
                if data.mounted.get().is_some() {
                    data.owner.report(
                        ErrorKind::Message,
                        "TextEdit supports only one mounted control",
                        at,
                    );
                    return;
                }
                data.mounted.set(Some(id));
                data.was_composing.set(false);
                on_cleanup(move || {
                    if data.mounted.get() == Some(id) {
                        data.mounted.set(None);
                    }
                    data.was_composing.set(false);
                });
            })
    }
}

type Submit = Rc<dyn Fn() -> Result<()>>;

type Composition = Rc<dyn Fn(bool) -> Result<()>>;

struct Data {
    owner: Owner,
    binding: Binding<String>,
    lines: Lines,
    forward_keys: Signal<Option<Value<Vec<Key>>>>,
    line_layout: Signal<EditLineLayout>,
    password: Signal<Option<Value<bool>>>,
    read_only: Signal<Option<Value<bool>>>,
    max_length: Signal<Option<Value<usize>>>,
    placeholder: Signal<Option<Value<String>>>,
    submit: RefCell<Option<Submit>>,
    composition: RefCell<Option<Composition>>,
    /// The control that is mounted now; others' edits are not written.
    mounted: Cell<Option<u64>>,
    next_control: Cell<u64>,
    was_composing: Cell<bool>,
}
