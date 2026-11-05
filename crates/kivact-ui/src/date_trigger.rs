//! Role: Preserve a caller-owned trigger for date-field dialog presentation.
use kivact_core::view::{IntoView, NodeBuilder, NodeRef, View, node_ref};
pub(crate) struct AttachTrigger {
    attach: Box<dyn FnOnce(crate::Dialog) -> View>,
    pub(crate) focus: NodeRef,
}
impl AttachTrigger {
    pub(crate) fn view(self, dialog: crate::Dialog) -> View {
        (self.attach)(dialog)
    }
}
pub(crate) enum FieldPresentation {
    Text,
    Trigger(AttachTrigger),
}
impl FieldPresentation {
    pub(crate) fn trigger<T: NodeBuilder + IntoView + 'static>(trigger: T) -> Self {
        let focus = node_ref();
        Self::Trigger(AttachTrigger {
            focus,
            attach: Box::new(move |dialog| dialog.attach(trigger.node_ref(focus))),
        })
    }
}
