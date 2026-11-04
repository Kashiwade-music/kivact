//! Role: Verify the native backend opens, presents and closes one window.

use arvane_shell::{
    ShellControl, ShellError, ShellEvent, ShellHandler, ShellOptions, ShellWaker, ShellWindow,
};

struct Smoke {
    presented: bool,
}
impl ShellHandler for Smoke {
    fn close(&mut self, _window: &mut dyn ShellWindow) {}
    fn start(
        &mut self,
        window: &mut dyn ShellWindow,
        _waker: ShellWaker,
    ) -> Result<ShellControl, ShellError> {
        window.request_redraw();
        Ok(ShellControl::Wait)
    }
    fn event(
        &mut self,
        event: ShellEvent,
        window: &mut dyn ShellWindow,
    ) -> Result<ShellControl, ShellError> {
        if matches!(event, ShellEvent::Redraw) {
            let (width, height) = window.physical_size();
            self.presented = window.present_with(width, height, &mut |pixels| {
                pixels.fill(0x00336699);
                true
            })?;
            return Ok(ShellControl::Exit);
        }
        Ok(ShellControl::Wait)
    }
}
#[test]
fn native_window_opens_presents_and_closes() {
    let backend = match arvane_native::backend(ShellOptions {
        width: 160.0,
        height: 120.0,
        title: "Arvane native smoke".into(),
    }) {
        Ok(backend) => backend,
        Err(error) => {
            eprintln!("native smoke skipped: display unavailable: {error}");
            return;
        }
    };
    let mut smoke = Smoke { presented: false };
    backend.run(&mut smoke).expect("native window runs");
    assert!(smoke.presented, "native window presented a frame");
}
