//! Role: Commit the focused control's edit as leaving it would, and wait for the commit.

use std::panic::Location;

use kivact_runtime::{Completion, Owner, require_effectful};

use crate::view::state::{BlurRequest, BlurStage, ViewState, current_window};

/// Run the focused control's "commit on focus loss" (a text input's draft
/// goes to its value, a number field parses and clamps) and finish once
/// that commit is written to the state. The control loses the focus; focus
/// it again with `NodeRef::focus` when the app wants it back. Nothing is
/// focused in this window: it finishes at once.
///
/// Call it from a handler or a task, for example a Save shortcut:
/// `spawn(async move { commit_focused_edit().await; save(model.get()) })`.
#[track_caller]
pub fn commit_focused_edit() -> impl Future<Output = ()> + 'static {
    let at = Location::caller();
    require_effectful("commit_focused_edit");
    let owner = Owner::running("commit_focused_edit", at);
    let rt = owner.runtime();
    let window = current_window(owner);
    let done = Completion::new();
    let state = ViewState::of(&rt);
    let focused = state
        .focus_keys(window)
        .is_some_and(|keys| keys.focus.get_untracked().is_some());
    if focused {
        state.requests.borrow_mut().blurs.push(BlurRequest {
            window,
            stage: BlurStage::Queued,
            done: done.clone(),
        });
        rt.request_turn();
    } else {
        done.complete(());
    }
    done
}
