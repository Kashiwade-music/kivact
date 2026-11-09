//! Role: The Emulation panel.

use kivact::prelude::*;

use crate::state::Session;

/// The environment emulation controls.
pub(crate) fn view(session: Session) -> impl IntoView {
    let _ = session;
    text("Emulation is not built yet")
}
