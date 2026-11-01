//! Role: Publishes the child semantics to AT-SPI without calling App code off-thread.

use super::conn::Native;
use crate::a11y::native::{convert_update, translate_action};
use arvane_shell::{PlatformTree, ShellError, ShellWaker};
use x11rb::{connection::Connection, protocol::xproto::ConnectionExt};

struct Activation(PlatformTree, ShellWaker);
impl accesskit::ActivationHandler for Activation {
    fn request_initial_tree(&mut self) -> Option<accesskit::TreeUpdate> {
        let update = self.0.activate_with(convert_update);
        self.1.wake();
        update
    }
}
struct Actions(PlatformTree, ShellWaker);
impl accesskit::ActionHandler for Actions {
    fn do_action(&mut self, request: accesskit::ActionRequest) {
        if let Some((node, action, value)) = translate_action(&request)
            && self.0.queue_action(node, action, value.as_deref())
        {
            self.1.wake();
        }
    }
}
struct Deactivation(PlatformTree);
impl accesskit::DeactivationHandler for Deactivation {
    fn deactivate_accessibility(&mut self) {
        self.0.deactivate();
    }
}

pub(super) struct Bridge {
    pub(super) tree: PlatformTree,
    adapter: accesskit_unix::Adapter,
}
impl Bridge {
    pub(super) fn new(scale: f64, waker: ShellWaker) -> Self {
        let tree = PlatformTree::new(scale);
        // The adapter owns a process-lifetime AT-SPI executor.
        crate::module::retain_module();
        let adapter = accesskit_unix::Adapter::new(
            Activation(tree.clone(), waker.clone()),
            Actions(tree.clone(), waker),
            Deactivation(tree.clone()),
        );
        Self { tree, adapter }
    }
    pub(super) fn focus(&mut self, focused: bool) {
        self.adapter.update_window_focus_state(focused);
    }
    pub(super) fn publish(
        &mut self,
        native: &Native,
        width: u32,
        height: u32,
        scale: f32,
    ) -> Result<(), ShellError> {
        self.tree.rescale(f64::from(scale));
        // Without an assistive client nothing is published, so the round trip
        // for the window origin is skipped. Activation wakes the loop, and the
        // next pass sets the bounds before its first publication.
        if !self.tree.active() {
            return Ok(());
        }
        let connection = &native.connection;
        let root = connection.setup().roots[native.screen as usize].root;
        let origin = connection
            .translate_coordinates(native.id.get(), root, 0, 0)
            .map_err(super::error::platform)?
            .reply();
        let origin = match origin {
            Ok(origin) => origin,
            Err(error) if super::error::destroyed(&error, native.id.get()) => {
                // The host can destroy its parent after the event queue was
                // drained. A checked reply can observe that before DestroyNotify.
                native
                    .alive
                    .store(false, std::sync::atomic::Ordering::Release);
                return Ok(());
            }
            Err(error) => return Err(super::error::platform(error)),
        };
        let bounds = accesskit::Rect::new(
            f64::from(origin.dst_x),
            f64::from(origin.dst_y),
            f64::from(origin.dst_x) + f64::from(width),
            f64::from(origin.dst_y) + f64::from(height),
        );
        self.adapter.set_root_window_bounds(bounds, bounds);
        if self.tree.has_pending(true) {
            self.adapter.update_if_active(|| {
                self.tree
                    .pending_with(true, convert_update)
                    .expect("active publication retains its accepted tree")
            });
        }
        Ok(())
    }
}
