//! Role: Observe editor line-state destruction through the engine reader boundary.
use kivact_engine::api::Reader;
use kivact_types::NodeId;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
pub(super) struct Probe {
    // xtask-allow(state-fields): independent: a test diagnostic; the armed flag is a separate observation
    pub(super) armed: Cell<bool>,
    // xtask-allow(state-fields): statistic: a test counter of drops
    pub(super) drops: Rc<Cell<usize>>,
    // xtask-allow(state-fields): statistic: a test counter of re-entered drops
    pub(super) reentered: Rc<Cell<usize>>,
    // xtask-allow(state-fields): independent: a test diagnostic; the reader is set once and read on drop
    pub(super) reader: Rc<RefCell<Option<(Reader, NodeId)>>>,
}
impl Drop for Probe {
    fn drop(&mut self) {
        if !self.armed.get() {
            return;
        }
        self.drops.set(self.drops.get() + 1);
        let (reader, independent) = self.reader.borrow().as_ref().unwrap().clone();
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
