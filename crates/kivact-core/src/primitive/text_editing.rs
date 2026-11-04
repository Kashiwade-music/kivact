//! Role: Attach native editing and query its geometry through public parts.
//!
//! Invariants:
//! - TextEditing methods stay here because highlighting reads ViewState;
//!   moving them into passive editing_data would create a state/tree cycle.
//!
//! `Part::text_editing(TextEditing)` gives an ordinary node the native
//! editing state, shaping, selection, caret and IME session. The source of
//! `TextEditing::new(text)` (an `Rc<str>`) and `password`, `read_only`,
//! `max_length` and `placeholder` bind to the one editing payload: whether a
//! node edits is this payload, not its kind, and a Box, a Text and an Image
//! take the same path. The author decides the Part's focusability, role and
//! name. An editor is a native leaf: children and custom drawing are
//! `Unsupported`, and an editing attachment on a canvas region is reported
//! as `Unsupported` and ignored. When several Parts are combined, the last
//! editing setting wins.
//!
//! The text lives in the editor only (no `InputValue` prop is written back),
//! and every change is a range replacement ([`TextEditEvent`]). Ranges are
//! byte offsets on UTF-8 char boundaries in `EditState`, `TextSession`,
//! `TextChange` and `TextEditEvent`; the platform IME uses UTF-16 units, and
//! scalar offsets appear in `TextLayout` carets and the accessibility
//! selection. The whole value (`event.text`) comes only with
//! [`TextEditing::report_text`], because it copies the text on every edit.
//! A controlled value is told apart by `Rc` identity (see
//! [`TextEditing::new`]). [`NodeRefExt::text_layout`] shapes only the lines
//! asked for; none of these queries subscribes.
//!
//! Invariants:
//! - The behavior and the line-color function are owned by the mount's
//!   owner; the engine's `EditBehavior` and `LineColors` (an `Rc` closure,
//!   compared with `Rc::ptr_eq`) hold them weakly, so the engine dropping a
//!   prop while its model is borrowed drops no user value.
//! - The behavior runs synchronously while the engine model is borrowed,
//!   through a `SyncHook`: pure and untracked, it reaches no node query or
//!   engine tree and its signal writes are rejected (a `WriteInPureContext`
//!   misuse panic). A panic in it propagates through the engine and ends the
//!   App.
//! - Without a behavior no key command edits by default and keys take the
//!   usual event route; there is no paste. IME and plain text input, pointer
//!   selection, shaping, caret drawing and reveal, clipboard IO and the
//!   accessibility `SetValue` stay on the shared native path. The standard
//!   policy is built on this contract alone by `interaction::use_text_edit`
//!   and `interaction::text_edit_policy`.
//! - A new `content` or theme makes the line-color effect write a new
//!   function; a panic in it propagates and ends the App.
//!
//! [`NodeRefExt::text_layout`]: crate::primitive::NodeRefExt::text_layout

use crate::view::editing_data::{
    EditorProps, HighlightAt, TextEditEvent, TextEditing, TextHighlight,
};
use crate::view::state::ViewState;
use crate::view::sync_hook::{Current, SyncHook};
use kivact_editing::{EditContext, EditOutcome, EditRequest, EditState, LineColors, LineState};
use kivact_engine::api::PropValue;
use kivact_reactive::EffectKind;
use kivact_reactive::HandlerResult;
use kivact_reactive::{IntoText, IntoValue, Prop as InputProp};
use kivact_style::Theme;
use std::cell::RefCell;
use std::rc::Rc;

impl TextEditing {
    /// Forward selected control keys to handlers instead of native focus or editing.
    ///
    /// Supported keys are Tab, arrows, Home, End, PageUp, PageDown, Escape and
    /// Enter. Character keys and Space are ignored. Ctrl/Alt/Meta retain native
    /// behavior; Shift is forwarded only with Tab. IME composition consumes all
    /// keys first. By default no keys are forwarded, which keeps Tab's focus
    /// move and the caret keys. A forwarded key has no native fallback even
    /// when its handler returns `Handled::No`, and the behavior does not see
    /// it. A reactive list takes effect after mount too. `TextInput` and
    /// `TextEdit` (`kivact::ui`, `kivact::interaction`) pass their
    /// `forward_keys` here.
    pub fn forward_keys(mut self, keys: impl IntoValue<Vec<crate::view::Key>>) -> Self {
        self.props.forward_keys = Some(keys.into_prop());
        self
    }

    /// Attach the policy for keys and paste; IME transport stays native.
    ///
    /// `f` runs synchronously for each routed key (Tab and Shift+Tab
    /// included: they move focus only when it returns
    /// [`EditOutcome::NotEditing`]) and each clipboard paste, while the
    /// engine holds its tree. It changes the supplied [`EditState`] and may
    /// read and write state it captured (`Cell`, `RefCell`); it must not
    /// query nodes or wait, and it runs pure and untracked: a signal write
    /// is rejected (a `WriteInPureContext` misuse panic) and a signal read
    /// subscribes nothing. The [`EditContext`]
    /// holds the editor's constraints and reads its laid-out lines (the
    /// line of an offset, the `x` of a caret, the offset nearest an `x`
    /// on a line: what moving up and down a line needs) for the text as
    /// the request found it. Without a behavior the editor has no key
    /// policy. A later call replaces an earlier one.
    pub fn behavior<F>(mut self, f: F) -> Self
    where
        F: Fn(&mut EditState, EditRequest<'_>, &EditContext<'_>) -> EditOutcome + 'static,
    {
        self.props.behavior = Some(Rc::new(f));
        self
    }

    /// Color the glyphs of each line with `f(&content, line, state)`.
    ///
    /// `f` gets one line at a time (without its line break) and the state
    /// the line before left (`S::default()` for the first line), and
    /// returns byte ranges in the line with their colors and the state the
    /// line leaves: a token that spans lines (a block comment, a string)
    /// carries over in the state. Overlapping and out-of-line ranges are
    /// cut, and text outside every range keeps the text color. Colors only
    /// change how the glyphs are drawn, never the shaping, the caret or
    /// the selection.
    ///
    /// `f` runs while the engine draws the editor, for the lines it draws
    /// and the lines before them it has not colored yet: after an edit it
    /// colors the changed lines again, and the lines after them only while
    /// the state they start from changed (as far as they are drawn; the
    /// others when they are). It must be a plain function of its arguments
    /// (nothing that waits; it runs pure and untracked, so a signal write
    /// is rejected as a `WriteInPureContext` misuse panic and a signal read
    /// subscribes nothing). A new `content`, or a new theme for token
    /// colors, colors the lines again. A panic in `f` propagates and ends
    /// the App. A password editor and its placeholder are not
    /// colored. A later call replaces an earlier one.
    pub fn highlight<T, S, F>(mut self, content: impl IntoValue<T>, f: F) -> Self
    where
        T: Clone + PartialEq + 'static,
        S: PartialEq + Default + 'static,
        F: Fn(&T, &str, &S) -> (Vec<TextHighlight>, S) + 'static,
    {
        let src = content.into_prop();
        let f = Rc::new(f);
        self.props.highlight = Some(Box::new(move |cx| bind_highlight(cx, src, f)));
        self
    }

    /// Attach the supplied text source. Later source changes preserve and
    /// clamp the selection; a composition holds them back until it ends.
    ///
    /// The editor holds its own copy of the text, which edits change by
    /// range ([`TextEditEvent`]): a source written only when the
    /// application replaces the text costs nothing per keystroke. The same
    /// `Rc` given again changes nothing unless the text was edited since.
    pub fn new(value: impl IntoValue<Rc<str>>) -> Self {
        Self {
            props: EditorProps {
                behavior: None,
                highlight: None,
                forward_keys: None,
                value: value.into_prop(),
                multiline: false,
                line_layout: None,
                report_text: false,
                password: None,
                read_only: None,
                max_length: None,
                placeholder: None,
            },
            on_edit: None,
            on_selection: None,
        }
    }

    /// Accept hard line breaks in the edited value.
    pub fn multiline(mut self, value: bool) -> Self {
        self.props.multiline = value;
        self
    }

    /// Configure soft wrapping and automatic row limits for a multiline editor.
    pub fn line_layout(mut self, value: impl IntoValue<kivact_types::EditLineLayout>) -> Self {
        self.props.line_layout = Some(value.into_prop());
        self
    }
    /// Also give every [`TextEditEvent`] the whole text after it
    /// (`event.text`), copied for each edit. An editor that keeps the
    /// value in a `String` binding needs it; one that applies the changes
    /// to its own copy does not.
    pub fn report_text(mut self, value: bool) -> Self {
        self.props.report_text = value;
        self
    }
    /// Draw masked text and omit its value from platform text snapshots.
    pub fn password(mut self, value: impl IntoValue<bool>) -> Self {
        self.props.password = Some(value.into_prop());
        self
    }
    /// Allow selection while rejecting value changes and new compositions.
    pub fn read_only(mut self, value: impl IntoValue<bool>) -> Self {
        self.props.read_only = Some(value.into_prop());
        self
    }
    /// Limit committed text by Unicode scalar count, constant or reactive: `Some(n)` limits, `None` does not
    /// (the default). Preedit is not truncated.
    pub fn max_length(mut self, value: impl IntoValue<Option<usize>>) -> Self {
        self.props.max_length = Some(value.into_prop());
        self
    }
    /// Show hint text while the editor is empty.
    pub fn placeholder(mut self, value: impl IntoText) -> Self {
        self.props.placeholder = Some(value.into_text());
        self
    }
    /// Observe native value and composition updates, including preedit.
    ///
    /// The event carries the range changes, not the whole text. An author
    /// implementing controlled editing applies them to a copy of the text
    /// and writes the binding only when `event.composition()` is `None`.
    /// Earlier caller handlers run first; an error stops the remaining
    /// handlers for that event.
    pub fn on_edit<F, R>(mut self, f: F) -> Self
    where
        F: Fn(TextEditEvent) -> R + 'static,
        R: HandlerResult,
    {
        self.on_edit = Some(Rc::new(move |event| f(event).into_result()));
        self
    }
    /// Observe the editor's session: `f` gets the selection and the
    /// composition (byte offsets of the text) whenever they change, from a
    /// caret move, a selection, an edit (after its [`on_edit`]) or a write
    /// ([`NodeRefExt::set_edit_session`]), and once when the editor is
    /// first laid out. A caret position display (line and column) reads
    /// it here. Earlier caller handlers run first; an error stops the
    /// remaining handlers for that change.
    ///
    /// [`on_edit`]: TextEditing::on_edit
    /// [`NodeRefExt::set_edit_session`]: crate::primitive::NodeRefExt::set_edit_session
    pub fn on_selection<F, R>(mut self, f: F) -> Self
    where
        F: Fn(crate::view::editing_data::TextSession) -> R + 'static,
        R: HandlerResult,
    {
        self.on_selection = Some(Rc::new(move |session| f(session).into_result()));
        self
    }
}

impl<S: Clone + PartialEq + 'static> crate::primitive::Part<S> {
    /// Add native editing to this attachment's node.
    ///
    /// Combining Parts keeps the last editing attachment. Regions report
    /// `Unsupported` and ignore it; an editing node remains a native leaf.
    pub fn text_editing(mut self, editing: crate::view::editing_data::TextEditing) -> Self {
        self.common.text_editing = Some(Box::new(editing));
        self
    }
}

/// Write the editor's line colors for every new content of `src` or new
/// theme, from one render effect that skips a content and theme equal to
/// the last (like a primitive's measure function).
fn bind_highlight<T, S, F>(cx: HighlightAt, src: InputProp<T>, f: Rc<F>)
where
    T: Clone + PartialEq + 'static,
    S: PartialEq + Default + 'static,
    F: Fn(&T, &str, &S) -> (Vec<TextHighlight>, S) + 'static,
{
    let HighlightAt {
        rt,
        owner,
        theme,
        id,
        at,
    } = cx;
    let current: Current<(T, Theme), Rc<F>> = Rc::new(RefCell::new(None));
    let held = current.clone();
    owner.on_cleanup(move || drop(held.borrow_mut().take()));
    let weak_rt = Rc::downgrade(&rt);
    let effect = move || {
        let content = src.get();
        let theme = theme.map_or_else(Theme::default, |theme| theme.get());
        let same = current
            .borrow()
            .as_ref()
            .is_some_and(|h| h.content.0 == content && h.content.1 == theme);
        if same {
            return;
        }
        let (hook, highlighted) = SyncHook::new(owner, (content, theme), f.clone());
        // The last content is dropped here, before the view tree is
        // borrowed: its `Drop` may read the tree.
        *current.borrow_mut() = Some(highlighted);
        let colors = LineColors::new(move |line, input: &LineState| {
            let start = S::default();
            let state = input.get::<S>().unwrap_or(&start);
            hook.call(|(content, theme), f| {
                let (ranges, output) = f(content, line, state);
                (
                    ranges
                        .into_iter()
                        .map(|r| (r.range, r.color.resolve(theme)))
                        .collect(),
                    LineState::new(output),
                )
            })
            .unwrap_or_else(|| (Vec::new(), input.clone()))
        });
        if let Some(rt) = weak_rt.upgrade() {
            ViewState::tree_mut(&rt).update(id, PropValue::LineColors(colors), rt.running_effect());
        }
    };
    owner.effect(EffectKind::Render, effect, at);
}
