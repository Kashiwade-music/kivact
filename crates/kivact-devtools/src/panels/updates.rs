//! Role: The Updates panel.

use kivact::prelude::*;

use crate::state::Session;

/// The recorded turns and cause graphs.
pub(crate) fn view(session: Session) -> impl IntoView {
    let _ = session;
    text("Updates is not built yet")
}
