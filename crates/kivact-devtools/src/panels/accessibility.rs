//! Role: The Accessibility panel.

use kivact::prelude::*;

use crate::state::Session;

/// The accessibility tree and its audits.
pub(crate) fn view(session: Session) -> impl IntoView {
    let _ = session;
    text("Accessibility is not built yet")
}
