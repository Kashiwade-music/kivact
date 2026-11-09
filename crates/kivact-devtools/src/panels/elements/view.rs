//! Role: The element tree of the Elements panel.

use kivact::prelude::*;

use crate::state::Session;

/// The element tree and the details of the selected node.
pub(crate) fn view(session: Session) -> impl IntoView {
    let _ = session;
    text("Elements is not built yet")
}
