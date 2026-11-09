//! Role: The Scripts panel.

use kivact::prelude::*;

use crate::state::Session;

/// The scripts, their output and their tables.
pub(crate) fn view(session: Session) -> impl IntoView {
    let _ = session;
    text("Scripts is not built yet")
}
