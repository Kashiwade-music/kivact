//! Role: Passive state of shared layouts: the boxes of departed layout ids.
//!
//! The model's layout motion group holds this type by its path. It names no
//! model type; the steps that record and take the boxes are in `mailbox`.

use std::collections::HashMap;
use std::sync::Arc;

use kivact_types::Rect;

/// Window layout boxes of removed nodes with a layout id, kept until the
/// next layout pass has had the chance to use them (a per-pass mailbox).
#[derive(Default)]
pub(crate) struct SharedLayouts {
    /// The departed box of each layout id.
    pub(super) departed: HashMap<Arc<str>, Rect>,
}
