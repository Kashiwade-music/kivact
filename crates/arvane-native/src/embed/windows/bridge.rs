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
        if let Some((node, request)) = translate_action(&request)
            && self.tree.queue_action(node, request)
        {
            self.waker.wake();
        }
    }
}
/// The UIA adapter of `hwnd` over the child's accepted `tree`.
pub(super) fn adapter(
    hwnd: isize,
    tree: PlatformTree,
    waker: ShellWaker,
) -> accesskit_windows::Adapter {
    accesskit_windows::Adapter::new(
        accesskit_windows::HWND(hwnd as *mut _),
        false,
        Actions { tree, waker },
    )
}
