//! Role: Verify the native backend opens, presents and closes several windows.

use arvane_shell::{
    EventTarget, ShellControl, ShellError, ShellEvent, ShellHandler, ShellLoop, ShellWaker,
};
use kivact_types::{Size, WindowId, WindowMode, WindowPlacement, WindowRequest};
use std::collections::BTreeSet;

/// The second window the smoke test opens beside the main one.
fn second() -> WindowId {
    WindowId::new(2).expect("invariant: 2 is not 0")
}

struct Smoke {
    presented: BTreeSet<WindowId>,
    windows_after_close: Vec<WindowId>,
    closed: bool,
}
impl ShellHandler for Smoke {
    fn close(&mut self, _shell: &mut dyn ShellLoop) {}
    fn start(
        &mut self,
        shell: &mut dyn ShellLoop,
        _waker: ShellWaker,
    ) -> Result<ShellControl, ShellError> {
        shell.open_window(WindowRequest {
            id: second(),
            title: "smoke 2".into(),
            size: Size {
                width: 120.0,
                height: 90.0,
            },
            min_size: None,
            mode: WindowMode::Tool,
            placement: WindowPlacement::CenterOnParent,
            parent: Some(WindowId::MAIN),
        })?;
        for id in shell.windows() {
            if let Some(window) = shell.window(id) {
                window.request_redraw();
            }
        }
        Ok(ShellControl::Wait)
    }
    fn event(
        &mut self,
        target: EventTarget,
        event: ShellEvent,
        shell: &mut dyn ShellLoop,
    ) -> Result<ShellControl, ShellError> {
        let (EventTarget::Window(id), ShellEvent::Redraw) = (target, event) else {
            return Ok(ShellControl::Wait);
        };
        let Some(window) = shell.window(id) else {
            return Ok(ShellControl::Wait);
        };
        let (width, height) = window.physical_size();
        if window.present_with(width, height, None, &mut |pixels, _age| {
            pixels.fill(0x00336699);
            true
        })? {
            self.presented.insert(id);
        }
        if id == second() && !self.closed {
            shell.close_window(second());
            self.windows_after_close = shell.windows();
            self.closed = true;
        }
        if self.presented.contains(&WindowId::MAIN) && self.closed {
            return Ok(ShellControl::Exit);
        }
        Ok(ShellControl::Wait)
    }
}
#[allow(
    clippy::print_stderr,
    reason = "report an unavailable display explicitly when skipping the test"
)]
fn native_window_opens_presents_and_closes() {
    let backend = match arvane_native::backend(WindowRequest::main(
        "Arvane native smoke",
        Size {
            width: 160.0,
            height: 120.0,
        },
    )) {
        Ok(backend) => backend,
        Err(error) => {
            eprintln!("native smoke skipped: display unavailable: {error}");
            return;
        }
    };
    let mut smoke = Smoke {
        presented: BTreeSet::new(),
        windows_after_close: Vec::new(),
        closed: false,
    };
    backend.run(&mut smoke).expect("native window runs");
    assert_eq!(smoke.windows_after_close, [WindowId::MAIN]);
    assert!(smoke.presented.contains(&WindowId::MAIN));
    assert!(smoke.presented.contains(&second()));
}

// The native backend owns the process main thread, as it does in an application.
fn main() {
    native_window_opens_presents_and_closes();
}
