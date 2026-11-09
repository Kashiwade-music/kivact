//! Role: The Theme panel.

use kivact::prelude::*;

use crate::state::Session;

/// The theme tokens and their overrides.
pub(crate) fn view(session: Session) -> impl IntoView {
    let _ = session;
    text("Theme is not built yet")
}
