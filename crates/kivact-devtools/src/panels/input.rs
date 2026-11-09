//! Role: The Input panel.

use kivact::prelude::*;

use crate::state::Session;

/// The recorded input events and the IME state.
pub(crate) fn view(session: Session) -> impl IntoView {
    let _ = session;
    text("Input is not built yet")
}
