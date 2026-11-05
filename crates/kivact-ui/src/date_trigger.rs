//! Role: Preserve a caller-owned trigger for date-field dialog presentation.
use kivact_core::view::{IntoView, NodeBuilder, View};
pub(crate) type AttachTrigger = Box<dyn FnOnce(crate::Dialog) -> View>;
pub(crate) enum FieldPresentation {
    Text,
    Trigger(AttachTrigger),
}
impl FieldPresentation {
    pub(crate) fn trigger<T: NodeBuilder + IntoView + 'static>(trigger: T) -> Self {
        Self::Trigger(Box::new(move |dialog| dialog.attach(trigger)))
    }
}
