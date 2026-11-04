//! Role: Typed queues for unstamped and acknowledged mount callbacks.
use crate::reactive::arena::Key;
use crate::reactive::data::{Mount, Runtime};
use kivact_types::Revision;
impl Runtime {
    pub(crate) fn push_mount(&self, owner: Key, run: Box<dyn FnOnce()>) {
        self.unstamped.borrow_mut().push(Mount { owner, run });
    }
    pub(crate) fn stamp_mounts(&self, revision: Revision) {
        let mut awaiting = self.awaiting.borrow_mut();
        assert!(
            self.mount_revision.get() <= revision,
            "invariant: mount revisions only grow"
        );
        self.mount_revision.set(revision);
        awaiting.extend(self.unstamped.borrow_mut().drain(..).map(|m| (revision, m)));
    }
    pub(crate) fn run_mounts_acked(&self, acked: Revision) {
        let mut due = Vec::new();
        {
            let mut awaiting = self.awaiting.borrow_mut();
            while awaiting
                .front()
                .is_some_and(|(revision, _)| *revision <= acked)
            {
                if let Some((_, mount)) = awaiting.pop_front() {
                    due.push(mount);
                }
            }
        }
        for Mount { owner, run } in due {
            if self.owner_alive(owner) {
                let _frame = self.handler_frame(Some(owner));
                run();
            }
        }
    }
    /// Whether any mount callback awaits an acknowledged revision.
    pub fn mounts_due(&self, acked: Revision) -> bool {
        self.awaiting
            .borrow()
            .front()
            .is_some_and(|(revision, _)| *revision <= acked)
    }
}
