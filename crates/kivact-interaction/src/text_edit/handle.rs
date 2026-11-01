//! Role: The `TextEdit` handle: native editing options and the control part over a string binding.

use super::policy::text_edit_policy;
use kivact_core::diagnostics::ErrorKind;
use kivact_core::primitive::{
    NodeRefExt, Owned, Owner, Part, PartState, TextEditing, part, require_setup,
};
use kivact_core::view::{Handled, HandlerResult, IntoText, IntoValue, Key, Role};
use kivact_core::{Binding, Result, Value, on_cleanup};
use std::cell::{Cell, RefCell};
use std::panic::Location;
use std::rc::Rc;

/// Headless controlled editing attached to one native leaf at a time.
///
/// Configure it during setup before the first mount. Select multiline mode
/// before creating `control()`. Other settings may be configured after
/// creating the Part, provided it has not mounted. A disposed control may
/// be attached again; simultaneous controls report a diagnostic.
#[derive(Clone, Copy)]
pub struct TextEdit {
    data: Owned<Data>,
    multiline: bool,
}

/// Create native editing behavior for an application-owned string binding.
///
/// Preedit remains in the native editor; the binding receives committed
/// values. Changes are observed through the binding: pass
/// `binding(read, write)` to run more code on each committed value. A writer
/// that rejects a value restores its own value.
/// Each committed edit copies the whole value into the binding; an editor
/// for long documents takes range edits from `TextEditing` instead.
#[track_caller]
pub fn use_text_edit(value: impl Into<Binding<String>>) -> TextEdit {
    let owner = Owner::current("text_edit");
    let data = Rc::new(Data {
        owner,
        binding: value.into(),
        forward_keys: Cell::new(None),
        password: Cell::new(None),
        read_only: Cell::new(None),
        max_length: Cell::new(None),
        placeholder: Cell::new(None),
        submit: RefCell::new(None),
        composition: RefCell::new(None),
        ever_mounted: Cell::new(false),
        active: Cell::new(false),
        was_composing: Cell::new(false),
    });
    TextEdit {
        data: owner.store("TextEdit", data),
        multiline: false,
    }
}

impl TextEdit {
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
        if let Some(data) = self.configure("TextEdit::forward_keys") {
            data.forward_keys.set(Some(keys.into_value()));
        }
        self
    }

    #[track_caller]
    fn configure(self, api: &'static str) -> Option<Rc<Data>> {
        let data = self.data.get();
        require_setup(api);
        if data.ever_mounted.get() {
            data.owner.report(
                ErrorKind::Message,
                "TextEdit must be configured before mounting",
                Location::caller(),
            );
            None
        } else {
            Some(data)
        }
    }
    /// Accept line breaks; select this mode before creating the control Part.
    #[track_caller]
    pub fn multiline(mut self, value: bool) -> Self {
        if self.configure("TextEdit::multiline").is_some() {
            self.multiline = value;
        }
        self
    }
    /// Mask the value and omit its text from clipboard and a11y snapshots.
    #[track_caller]
    pub fn password(self, value: impl IntoValue<bool>) -> Self {
        if let Some(data) = self.configure("TextEdit::password") {
            data.password.set(Some(value.into_value()));
        }
        self
    }
    /// Permit selection while rejecting user edits and new compositions.
    #[track_caller]
    pub fn read_only(self, value: impl IntoValue<bool>) -> Self {
        if let Some(data) = self.configure("TextEdit::read_only") {
            data.read_only.set(Some(value.into_value()));
        }
        self
    }
    /// Limit committed text by Unicode scalar count; preedit is not truncated.
    #[track_caller]
    pub fn max_length(self, value: impl IntoValue<usize>) -> Self {
        if let Some(data) = self.configure("TextEdit::max_length") {
            data.max_length.set(Some(value.into_value()));
        }
        self
    }
    /// Show hint text while the editor is empty.
    #[track_caller]
    pub fn placeholder(self, value: impl IntoText) -> Self {
        if let Some(data) = self.configure("TextEdit::placeholder") {
            data.placeholder.set(Some(value.into_text_value()));
        }
        self
    }
    /// Handle Enter in a single-line editor after native composition routing.
    #[track_caller]
    pub fn on_submit<F, R>(self, f: F) -> Self
    where
        F: Fn() -> R + 'static,
        R: HandlerResult,
    {
        if let Some(data) = self.configure("TextEdit::on_submit") {
            *data.submit.borrow_mut() = Some(Rc::new(move || f().into_result()));
        }
        self
    }
    /// Observe transitions into and out of native composition.
    #[track_caller]
    pub fn on_composition_change<F, R>(self, f: F) -> Self
    where
        F: Fn(bool) -> R + 'static,
        R: HandlerResult,
    {
        if let Some(data) = self.configure("TextEdit::on_composition_change") {
            *data.composition.borrow_mut() = Some(Rc::new(move |value| f(value).into_result()));
        }
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
        let binding = self.data.get().binding;
        let accepted = Rc::new(Cell::new(false));
        let receiving = accepted.clone();
        let submitting = accepted.clone();
        let value = binding.into_value();
        let editing = TextEditing::new(move || Rc::<str>::from(value.get()))
            .report_text(true)
            .behavior(text_edit_policy)
            .forward_keys(move || {
                self.data
                    .get()
                    .forward_keys
                    .get()
                    .map_or_else(Vec::new, |keys| keys.get())
            })
            .multiline(self.multiline)
            .password(move || {
                self.data
                    .get()
                    .password
                    .get()
                    .is_some_and(|value| value.get())
            })
            .read_only(move || {
                self.data
                    .get()
                    .read_only
                    .get()
                    .is_some_and(|value| value.get())
            })
            .max_length(move || {
                self.data
                    .get()
                    .max_length
                    .get()
                    .map_or(usize::MAX, |value| value.get())
            })
            .placeholder(move || {
                self.data
                    .get()
                    .placeholder
                    .get()
                    .map_or_else(String::new, |value| value.get())
            })
            .on_edit(move |edit| {
                if !receiving.get() {
                    return Ok(());
                }
                let data = self.data.get();
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
                if submitting.get() && !self.multiline && event.key == Key::Enter {
                    let submit = self.data.get().submit.borrow().clone();
                    if let Some(submit) = submit {
                        submit()?;
                        return Ok(Handled::Yes);
                    }
                }
                Ok(Handled::No)
            })
            .on_bind(move |_| {
                let data = self.data.get();
                if data.active.replace(true) {
                    data.owner.report(
                        ErrorKind::Message,
                        "TextEdit supports only one mounted control",
                        at,
                    );
                    return;
                }
                accepted.set(true);
                data.ever_mounted.set(true);
                data.was_composing.set(false);
                on_cleanup(move || {
                    data.active.set(false);
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
    forward_keys: Cell<Option<Value<Vec<Key>>>>,
    password: Cell<Option<Value<bool>>>,
    read_only: Cell<Option<Value<bool>>>,
    max_length: Cell<Option<Value<usize>>>,
    placeholder: Cell<Option<Value<String>>>,
    submit: RefCell<Option<Submit>>,
    composition: RefCell<Option<Composition>>,
    ever_mounted: Cell<bool>,
    active: Cell<bool>,
    was_composing: Cell<bool>,
}
