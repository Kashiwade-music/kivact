//! Role: Synchronous UIA adapter, independent from application callback borrows.
use crate::a11y::native::{convert_update, translate_action};
use arvane_shell::{PlatformTree, ShellWaker};

pub(super) struct Activation {
    pub(super) tree: PlatformTree,
    pub(super) waker: ShellWaker,
}
impl accesskit::ActivationHandler for Activation {
    fn request_initial_tree(&mut self) -> Option<accesskit::TreeUpdate> {
        crate::retain_module();
        let update = self.tree.activate_with(convert_update);
        self.waker.wake();
        update
    }
}
struct Actions {
    pub(super) tree: PlatformTree,
    pub(super) waker: ShellWaker,
}
impl accesskit::ActionHandler for Actions {
    fn do_action(&mut self, request: accesskit::ActionRequest) {
        if let Some((node, action, value)) = translate_action(&request)
            && self.tree.queue_action(node, action, value.as_deref())
        {
            self.waker.wake();
        }
    }
}
pub(super) struct Bridge {
    pub(super) tree: PlatformTree,
    pub(super) adapter: accesskit_windows::Adapter,
}
impl Bridge {
    pub(super) fn new(hwnd: isize, waker: ShellWaker, scale: f64) -> Self {
        let tree = PlatformTree::new(scale);
        Self {
            adapter: accesskit_windows::Adapter::new(
                accesskit_windows::HWND(hwnd as *mut _),
                false,
                Actions {
                    tree: tree.clone(),
                    waker,
                },
            ),
            tree,
        }
    }
}
