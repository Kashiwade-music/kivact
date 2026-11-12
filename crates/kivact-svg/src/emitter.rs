//! Role: The state of one SVG export: markup, definitions and the host.

use crate::Resources;

/// The state of one export: the markup written so far, the definitions the
/// markup refers to, and the host that answers the writer's questions. The
/// nesting of the groups is not kept here; it is the recursion of `commands`.
pub(super) struct Emitter<'a> {
    pub(super) body: String,
    pub(super) defs: String,
    next_id: u32,
    pub(super) resources: &'a mut dyn Resources,
}

impl<'a> Emitter<'a> {
    pub(super) fn new(resources: &'a mut dyn Resources) -> Self {
        Self {
            body: String::new(),
            defs: String::new(),
            next_id: 0,
            resources,
        }
    }

    /// A fresh identifier for a definition: `prefix` and a counter that only
    /// numbers the definitions in the order they are written.
    pub(super) fn id(&mut self, prefix: &str) -> String {
        self.next_id += 1;
        format!("{prefix}{}", self.next_id)
    }
}
