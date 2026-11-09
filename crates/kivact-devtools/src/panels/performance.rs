//! Role: The Performance panel.

use kivact::prelude::*;

use crate::state::Session;

/// Frame timings and the flame chart.
pub(crate) fn view(session: Session) -> impl IntoView {
    let _ = session;
    text("Performance is not built yet")
}
