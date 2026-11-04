//! Role: The IME session of a focusable node that edits its own text.
//!
//! A node outside the native editors can carry an IME session
//! (`NodeProperties::ime`): the text around its caret, its selection and the
//! caret's place. While such a node has the focus the window allows the IME,
//! puts its candidate window at the session's caret, and answers the IME's
//! queries from the session; the IME's composition, commits and reconversion
//! ranges leave as `HostEvent::Ime` for the node's own handler.
//!
//! Invariants:
//! - A native editor never has a session of this kind: its own path keeps
//!   the IME. A disabled node has none.
//! - The session's text is the committed text. The composition belongs to the
//!   node, which shows it where it wants and moves the caret rectangle to
//!   it; the IME is given no composition range.
//! - The IME is given its text only when the session, the node's box or
//!   its transforms changed since the last time; its answers' offsets are
//!   UTF-16 code units of that text, which starts at byte 0, and reach the
//!   node as byte offsets (clamped to the text).

use super::state::{ImeGiven, SessionSource};
use crate::contract::{HostEvent, ImeSessionProps};
use crate::model::Model;
use crate::model::transform::map_rect;
use kivact_types::{ImeInput, NodeId, Rect};

impl Model {
    /// The focused node that has an IME session (it is not a native editor
    /// and not disabled).
    pub(crate) fn focused_session(&self) -> Option<NodeId> {
        let target = self.focused().filter(|f| f.region.is_none())?;
        let n = self.tree.get(target.node)?;
        (n.properties.ime.is_some() && self.editors[n.index.0].is_none() && !n.disabled())
            .then_some(target.node)
    }

    /// The session of the focused node, if it has one.
    fn session(&self, id: NodeId) -> Option<&ImeSessionProps> {
        self.tree.get(id)?.properties.ime.as_deref()
    }

    /// Where the IME's candidate window goes while a node with a session has
    /// the focus: its caret in the window, as drawn.
    pub(crate) fn session_area(&self) -> Option<Rect> {
        let id = self.focused_session()?;
        let caret = self.session(id)?.caret;
        Some(self.session_rect(id, caret))
    }

    /// `rect` in the node's own coordinates, in the window as drawn.
    fn session_rect(&self, id: NodeId, rect: Rect) -> Rect {
        let at = self.rect(id).unwrap_or_default();
        let t = self.transform_between(id, None);
        map_rect(
            t.as_ref(),
            Rect {
                x: at.x + rect.x,
                y: at.y + rect.y,
                ..rect
            },
        )
    }

    /// Send `event` to the focused node with a session; whether there is
    /// one (otherwise the native editors take the event).
    pub(super) fn session_event(&mut self, event: ImeInput) -> bool {
        let Some(node) = self.focused_session() else {
            return false;
        };
        self.push(HostEvent::Ime { node, event });
        true
    }

    /// The IME's composition, to the focused session node if there is one.
    pub(super) fn session_preedit(
        &mut self,
        text: &str,
        selection: Option<(usize, usize)>,
    ) -> bool {
        self.session_event(ImeInput::Preedit {
            text: text.to_string(),
            selection: selection.map(|(a, b)| a.min(b)..a.max(b)),
        })
    }

    /// The range the IME took for reconversion (UTF-16 code units of the
    /// session's text), to the focused session node if there is one.
    pub(super) fn session_select(&mut self, start: usize, end: usize) -> bool {
        let Some(id) = self.focused_session() else {
            return false;
        };
        let Some(session) = self.session(id) else {
            return false;
        };
        let text = &session.text;
        let (a, b) = (
            kivact_types::text_utf::floor_char_boundary(text, start),
            kivact_types::text_utf::floor_char_boundary(text, end),
        );
        self.session_event(ImeInput::Selection(a.min(b)..a.max(b)))
    }

    /// What the focused session node `id` shows the IME, when it changed
    /// since the last time given (`Model::ime_changed` calls it).
    pub(super) fn session_ime_changed(&mut self, id: NodeId) -> Option<arvane::ShellImeText> {
        let session = self.session(id)?.clone();
        let t = self.transform_between(id, None);
        let source = SessionSource {
            node: id,
            session: session.clone(),
            area: self.rect(id),
            transform: t,
        };
        if matches!(&self.edits.given, ImeGiven::Session(given) if *given == source) {
            return None;
        }
        self.edits.given = ImeGiven::Session(source);
        let caret = self.session_rect(id, session.caret);
        let area = self.rect(id).unwrap_or_default();
        let area = map_rect(self.transform_between(id, None).as_ref(), area);
        let text = if session.password {
            arvane::ShellImeText {
                password: true,
                rects: vec![caret],
                area,
                ..arvane::ShellImeText::default()
            }
        } else {
            let units = |byte: usize| kivact_types::text_utf::utf16_offset(&session.text, byte);
            let (start, end) = (units(session.selection.start), units(session.selection.end));
            arvane::ShellImeText {
                password: false,
                text: session.text.clone(),
                text_start: 0,
                selection: (start.min(end), start.max(end)),
                caret: end,
                composition: None,
                rects_from: end,
                rects: vec![caret],
                area,
            }
        };
        Some(text)
    }
}
