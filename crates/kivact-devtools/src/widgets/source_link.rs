//! Role: A link to a source location in the user's editor.

use kivact::prelude::*;
use kivact_devtools_protocol as p;

use crate::state::Session;

/// A link that opens `location` through the session.
pub(crate) fn source_link(location: p::SourceLocation, session: Session) -> impl IntoView {
    let label = format!("{}:{}:{}", location.file, location.line, location.column);
    text(label.clone())
        .role(Role::Link)
        .name(label)
        .focusable(true)
        .style(style().font_size(11).single_line())
        .on_press(move || session.open_source(location.clone()))
}
