//! Role: The text inputs' editing state, its index, and the IME's view of it.
//!
//! The edit group: which nodes are text inputs with a native editing
//! state (`HostNode::edit`), and the IME text last given to the window;
//! with the writes that keep a node's editing state in step with its
//! `Editing` prop and controlled value, and the IME's entry points.
//!
//! Invariants:
//! - [`Model::set_edit`] is the only place a node gains or loses its
//!   editing state, and it keeps the index of text inputs with it: the
//!   index lists exactly the live nodes with one.
//! - A controlled value replaces the value but keeps the composition
//!   (`Editor::set_controlled`); one given while composing is held and
//!   taken, as a recorded change in the same `TextChanged`, by the edit
//!   that ends the composition (`Model::adopt_held`). A node's first value
//!   starts with the caret at the end.
//! - A controlled value is known by its `Rc`: the same one given again
//!   while the text keeps the revision it was given at changes nothing
//!   without comparing texts. Texts are compared only for another value,
//!   or for the same one after an edit. A controlled value queues no
//!   `TextChanged` (it is the application's own), and is taken by a
//!   disabled or read-only input too.
//! - A state kivact gives back changes the text by the changes it
//!   recorded since it was read; texts are compared only for a state read
//!   at another revision: another text is taken whole, as one range
//!   replacement, and the same text replaces only the session, keeping
//!   its revision (`Taken::Same`). It goes through the editing entry
//!   point, so a disabled input refuses it and a read-only one takes only
//!   its selection.
//! - A session kivact gives (`Model::replace_edit_session`) replaces the
//!   selection and the composition without reading or copying the text,
//!   clamped to its char boundaries; a disabled input refuses it, a
//!   read-only one takes only the selection, and a composition it starts
//!   or ends queues a `TextChanged`.
//! - A text input that reports its session queues one `SelectionChanged`
//!   whenever its session is not the one it last reported, after the
//!   entry point that changed it (`Model::sync`).
//! - The IME is given its text only when the text's revision, the session
//!   or the geometry changed since the last time. The offsets the IME
//!   answers with are in the text it was last given, which starts at
//!   `Edits::ime_start` in the value.
//! - The range the IME takes for reconversion (`ShellEvent::ImeSelection`,
//!   [`Model::ime_select`]) becomes the selection, which the composition
//!   that follows replaces. It is ignored while composing, and in a
//!   password or read-only input.

mod ime;
mod inputs;
mod session;
pub(crate) mod state;
