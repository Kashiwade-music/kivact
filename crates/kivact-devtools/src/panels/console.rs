//! Role: The Console panel.

use kivact::prelude::*;

use crate::state::Session;

/// The errors and notes the app reported.
pub(crate) fn view(session: Session) -> impl IntoView {
    let _ = session;
    text("Console is not built yet")
}
