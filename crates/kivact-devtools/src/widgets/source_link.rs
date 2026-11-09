//! Role: A link to a source location in the user's editor.

use kivact::prelude::*;
use kivact_devtools_protocol as p;

use crate::state::Session;

/// A link that opens `location` through the session.
pub(crate) fn source_link(location: p::SourceLocation, session: Session) -> impl IntoView {
    let _ = (location, session);
    node()
}
