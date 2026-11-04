//! Role: Portable platform event injection through the real test window.
use super::model::Screen;
use crate::draw::Point;
use crate::view::{FilePhase, Key, KeyCode, Modifiers};
use kivact_engine::testing::ShellEvent;
impl Screen {
    /// Normalize raw platform input through the production shell pipeline.
    pub fn raw_input(&self, input: crate::test::raw::RawInput) {
        self.host.borrow_mut().raw_input(input);
        self.run_until_idle();
    }

    /// Inject a key press at a physical keyboard position.
    pub fn key_down(&self, key: Key, physical_code: KeyCode, modifiers: Modifiers) {
        self.send(ShellEvent::Modifiers(modifiers));
        self.send(ShellEvent::Key(kivact_types::KeyInput {
            key,
            code: physical_code,
            phase: kivact_types::KeyPhase::Down {
                repeat: false,
                text: None,
            },
            modifiers,
            pointer: None,
        }));
        self.run_until_idle();
    }
    /// Inject a release; it cannot type text or activate a control.
    pub fn key_up(&self, key: Key, physical_code: KeyCode, modifiers: Modifiers) {
        self.send(ShellEvent::Modifiers(modifiers));
        self.send(ShellEvent::Key(kivact_types::KeyInput {
            key,
            code: physical_code,
            phase: kivact_types::KeyPhase::Up,
            modifiers,
            pointer: None,
        }));
        self.run_until_idle();
    }
    /// Inject one OS file drag event with a window position.
    pub fn file_drag(&self, phase: FilePhase, files: Vec<std::path::PathBuf>, point: Point) {
        self.send(ShellEvent::File(kivact_types::FileDrop {
            phase,
            files,
            position: kivact_engine::testing::Point {
                x: point.x,
                y: point.y,
            },
        }));
        self.run_until_idle();
    }
}
