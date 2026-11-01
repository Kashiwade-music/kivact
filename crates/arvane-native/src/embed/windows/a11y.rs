//! Role: UIA messages and state publication.
use super::bridge::Activation;
use crate::a11y::native::convert_update;
use std::rc::Rc;
pub(super) fn get_object(state: &Rc<super::state::State>, w: usize, l: isize) -> Option<isize> {
    let result = {
        let mut slot = state.a11y.borrow_mut();
        let bridge = slot.as_mut()?;
        let mut activation = Activation {
            tree: bridge.tree.clone(),
            waker: state.waker(),
        };
        bridge.adapter.handle_wm_getobject(
            accesskit_windows::WPARAM(w),
            accesskit_windows::LPARAM(l),
            &mut activation,
        )
    };
    // Into<LRESULT> calls UIA and can synchronously reenter WM_GETOBJECT.
    result.map(|value| {
        let result: accesskit_windows::LRESULT = value.into();
        result.0
    })
}
pub(super) fn publish(state: &super::state::State) {
    let events = {
        let mut slot = state.a11y.borrow_mut();
        let Some(bridge) = slot.as_mut() else { return };
        let scale = f64::from(state.shared.scale());
        bridge.tree.rescale(scale);
        if !bridge.tree.has_pending(true) {
            return;
        }
        bridge.adapter.update_if_active(|| {
            bridge
                .tree
                .pending_with(true, convert_update)
                .expect("active publication retains its accepted tree")
        })
    };
    if let Some(events) = events {
        events.raise();
    }
}
pub(super) fn focus(state: &super::state::State, focused: bool) {
    let events = state
        .a11y
        .borrow_mut()
        .as_mut()
        .and_then(|bridge| bridge.adapter.update_window_focus_state(focused));
    if let Some(events) = events {
        events.raise();
    }
}

pub(super) fn disconnect(state: &super::state::State) {
    // SAFETY: disconnect only this child's provider before releasing its adapter.
    unsafe {
        windows_sys::Win32::UI::Accessibility::UiaReturnRawElementProvider(
            state.native.hwnd(),
            0,
            0,
            std::ptr::null_mut(),
        );
    }
}
