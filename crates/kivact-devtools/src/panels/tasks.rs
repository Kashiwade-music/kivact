//! Role: The Async panel.

use kivact::prelude::*;

use crate::state::Session;

/// The tasks, resources, actions and timers.
pub(crate) fn view(session: Session) -> impl IntoView {
    let _ = session;
    text("Async is not built yet")
}
