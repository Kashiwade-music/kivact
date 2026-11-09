//! Role: The nested boxes of a node's box model.

use kivact::prelude::*;
use kivact_devtools_protocol as p;

/// The margin, border, padding and content boxes of a node.
pub(crate) fn box_model(bounds: Value<Option<p::BoxModel>>) -> impl IntoView {
    let _ = bounds;
    node()
}
