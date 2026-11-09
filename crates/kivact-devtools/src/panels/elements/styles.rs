//! Role: The Styles tab of the Elements panel.

use kivact::prelude::*;
use kivact_devtools_protocol as p;

use crate::state::Session;

/// The style layers and overrides of the selected node.
pub(crate) fn styles_tab(
    session: Session,
    detail: Resource<Option<p::NodeDetail>>,
) -> impl IntoView {
    let _ = (session, detail);
    node()
}
