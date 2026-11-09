//! Role: The Review panel.

use kivact::prelude::*;

use crate::state::Session;

/// The comments of the review and their export.
pub(crate) fn view(session: Session) -> impl IntoView {
    let _ = session;
    text("Review is not built yet")
}
