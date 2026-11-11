//! Role: Observe editor line-state destruction through the engine reader boundary.
use kivact_engine::api::Reader;
use kivact_types::NodeId;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
pub(super) struct Probe {
    // xtask-allow(state-fields): independent: the reader and the other live node the test arms this probe with; an unarmed probe skips the drop check, and the shared counters count drops whatever it holds
    pub(super) armed: RefCell<Option<(Reader, NodeId)>>,
    // xtask-allow(state-fields): statistic: a test counter of drops
    pub(super) drops: Rc<Cell<usize>>,
    // xtask-allow(state-fields): statistic: a test counter of re-entered drops
    pub(super) reentered: Rc<Cell<usize>>,
}
impl Drop for Probe {
    fn drop(&mut self) {
        let Some((reader, independent)) = self.armed.borrow_mut().take() else {
            return;
        };
        self.drops.set(self.drops.get() + 1);
        // This is a different still-live node: replacing the editor must
        // not invalidate the reference or hold the shared Model borrow.
        reader.with_node(independent, |node| assert!(node.is_some()));
        self.reentered.set(self.reentered.get() + 1);
    }
}
#[derive(Default)]
pub(super) struct State(pub(super) Option<Rc<Probe>>);
impl PartialEq for State {
    fn eq(&self, other: &Self) -> bool {
        match (&self.0, &other.0) {
            (None, None) => true,
            (Some(a), Some(b)) => Rc::ptr_eq(a, b),
            _ => false,
        }
    }
}
