//! Role: Pointer state machine: hover, press, capture, click, drag and wheel routing.
//!
//! The pointer state machine: one track per pointer (the mouse, each touch
//! or pen contact) with its hover chain, `Enter`/`Leave` of the nearest
//! pointer listener and its gesture (pointer capture and pressed chain,
//! a scrollbar drag or a touch pan, one at a time); press → focus, click →
//! `Activate`, drag selection in a text input, and wheel routing. The
//! caller hit-tests ([`Model::hit`]) and gives the pointer's window
//! position ([`PointerSpace`]).
//!
//! Invariants:
//! - A hit outside the scope of a shown modal hits nothing (the backdrop).
//! - Every pointer has a track of its own (`PointerState::tracks`): the
//!   mouse always, a touch or pen contact from its `Down` to its `Up` or
//!   `Cancel`. Its gesture (`Gesture::Route` with its capture, pressed
//!   chain and press target, `Gesture::Scrollbar` or `Gesture::Pan`),
//!   `Enter`/`Leave` listener and hover chain are its own; its events carry
//!   its id, kind, pressure and whether it is primary; no pointer waits for
//!   another. The buttons it holds are the shell's (`PointerSample::buttons`),
//!   kept once in its track's source.
//! - The primary pointers (Pointer Events' `isPrimary`: the mouse; the
//!   first touch and the first pen contact while no other of its kind is in
//!   contact) also make what is synthesized from the pointer: the hover and
//!   pressed state, `HoverChanged`, the cursor (the mouse's), and a press's
//!   outside dismissal, focus, caret and activation. Another contact only
//!   gets its pointer events, at the listener it captured.
//! - A pointer's hover chain is the hit node and its ancestors; its pressed
//!   chain is its hover chain at a primary press, until the release. A node
//!   is hovered while a primary pointer hovers it, and shows its pressed
//!   state only while a primary pointer that pressed it still hovers it, so
//!   a hover change repaints the pressed nodes it flips, hover style or not.
//! - `Enter` and `Leave` go to the nearest pointer listener under each
//!   pointer when it changes (moving between its children sends neither);
//!   `Move` goes to the pointer's capture, else to that listener once
//!   entered.
//! - `HoverChanged` reports the head of the first non-empty hover chain
//!   (the mouse's first, then the contacts' in the order they began: the
//!   hit itself, whatever listener or capture there is) whenever it
//!   changes. A commit clears a hit that got hidden or left the shown
//!   modal's scope, the removal of the hit clears it (its surviving
//!   ancestors are no new hit), and so does the mouse leaving the window
//!   with a button held. Otherwise only the pointer's own input hits again:
//!   a layout change under a pointer that stays still changes no hover.
//! - A primary pointer's press first dismisses the overlays it lands
//!   outside of, then, with no capture, any button first resolves the hit
//!   region or nearest declared pointer focus destination once, then falls
//!   back to its native focus target. Without a declaration, any button
//!   focuses the nearest focusable ancestor. A primary press on nothing
//!   focusable blurs (committing a composition), except on a modal's
//!   backdrop or inside a shown overlay whose anchor holds the focus
//!   ([`Model::press_blurs`]); a press of another button on nothing
//!   focusable, and any press while a capture is held, keep the focus. Only
//!   the primary button presses and activates.
//! - After its first `Down` the listener holds that pointer's capture: its
//!   later button `Down`, `Move`, each `Up` and `Cancel` go to it until its
//!   last held button releases, wherever the pointer is. A mouse move with
//!   no button held while a capture is active ends it with `Cancel`.
//! - A press on an enabled region offering `Activate` retains that exact region;
//!   `Activate` requires a release on the same still-available region. A
//!   disabled region blocks activation. Other presses retain the nearest
//!   activatable ancestor and activate its own action on a subtree release.
//! - Canceling one pointer ends only its own capture and press; losing the
//!   window's focus cancels them all (`Model::cancel_pointer`).

mod hover;
mod machine;
mod shell;
mod space;
pub(crate) mod state;
mod touch;

pub(crate) use space::PointerSpace;
