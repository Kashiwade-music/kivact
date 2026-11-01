//! Role: Test controls and component building.
//!
//! Host controls of a `Screen`: environment and surface changes, resize,
//! scale, close, clipboard, platform scripting and focus.

use std::path::PathBuf;
use std::time::Duration;

use kivact_wire as p;

use kivact_engine::testing::Insets;

use super::model::{Element, Screen};
use kivact_style::Cursor;

impl Screen {
    /// Logical content sizes requested through `platform::request_window_size`.
    pub fn requested_sizes(&self) -> Vec<crate::draw::Size> {
        self.host
            .borrow()
            .requested_sizes()
            .iter()
            .map(|s| crate::draw::Size::new(s.width, s.height))
            .collect()
    }

    /// Report native window focus loss or restoration (cancels pointer capture).
    pub fn set_window_focused(&self, focused: bool) {
        self.send(kivact_engine::testing::ShellEvent::Focused(focused));
        self.run_until_idle();
    }
    /// The host reports reduced motion on or off (`EnvChanged`).
    pub fn set_reduced_motion(&self, on: bool) {
        self.log(format!("reduced motion {on}"));
        self.host.borrow_mut().set_reduced_motion(on);
        self.run_until_idle();
    }

    /// The host reports a dark (or light) system theme (`EnvChanged`).
    pub fn set_dark(&self, dark: bool) {
        self.log(format!("dark {dark}"));
        self.host.borrow_mut().set_theme(if dark {
            p::ThemeHint::Dark
        } else {
            p::ThemeHint::Light
        });
        self.run_until_idle();
    }

    /// The host reports another system locale (a BCP 47 tag; `None`: the
    /// system names none) (`EnvChanged`). An App that named no locale
    /// (`App::locale`) follows it; one that did ignores it.
    pub fn set_host_locale(&self, locale: Option<&str>) {
        self.log(format!("host locale {locale:?}"));
        self.host.borrow_mut().set_locale(locale);
        self.run_until_idle();
    }

    /// The window's safe area changes, as a phone's system UI (status bar,
    /// notch) does when it rotates (`EnvChanged`; read by
    /// `platform::safe_area`).
    pub fn set_safe_area(&self, insets: crate::platform::Insets) {
        self.log(format!("safe area {insets:?}"));
        self.host.borrow_mut().set_safe_area(Insets {
            top: insets.top,
            right: insets.right,
            bottom: insets.bottom,
            left: insets.left,
        });
        self.run_until_idle();
    }

    /// The platform suspends the App (sends it to the background, as
    /// Android and iOS do): nothing is drawn and the frame clock stops
    /// (`platform::surface_active` is false) until [`Screen::resume`]. With
    /// `destroys_surface` the window's surface goes too, as on Android.
    pub fn suspend(&self, destroys_surface: bool) {
        self.log(format!("suspended (surface destroyed: {destroys_surface})"));
        self.host.borrow_mut().suspend(destroys_surface);
        self.run_until_idle();
    }

    /// The suspended App comes back to the foreground (with a new surface
    /// if it lost its own) and draws again.
    pub fn resume(&self) {
        self.log("resumed".into());
        self.host.borrow_mut().resume();
        self.run_until_idle();
    }

    /// The platform warns that memory is short.
    pub fn memory_warning(&self) {
        self.log("memory warning".into());
        self.host.borrow_mut().memory_warning();
        self.run_until_idle();
    }

    /// The window's surface is lost, as when the window is minimized or
    /// the platform destroys it (no frames, no frame ticks).
    pub fn lose_surface(&self) {
        self.log("surface lost".into());
        self.host.borrow_mut().set_surface(false);
        self.run_until_idle();
    }

    /// The window is restored: the surface is back.
    pub fn restore_surface(&self) {
        self.log("surface restored".into());
        self.host.borrow_mut().set_surface(true);
        self.run_until_idle();
    }

    /// Resize the window to `width` × `height` logical pixels.
    pub fn resize(&self, width: f32, height: f32) {
        self.log(format!("resize {width} x {height}"));
        self.host.borrow_mut().resize(width, height);
        self.run_until_idle();
    }

    /// Change the scale (physical pixels per logical pixel), as when the
    /// window moves to a monitor with another DPI.
    pub fn set_scale(&self, scale: f32) {
        self.log(format!("scale {scale}"));
        self.scale.set(scale);
        self.host.borrow_mut().set_scale(scale);
        self.run_until_idle();
    }

    /// Ask the App to close; returns whether it exited (queued actions
    /// drained or abandoned after the 3 s deadline, advancing virtual time
    /// as needed).
    pub fn request_close(&self) -> bool {
        self.log("close requested".into());
        self.host.borrow_mut().request_close();
        self.run_until_idle();
        for _ in 0..1000 {
            if self.host.borrow().exited() {
                return true;
            }
            self.advance(Duration::from_millis(100));
        }
        self.host.borrow().exited()
    }

    /// Whether the App exited (it accepted a close request).
    pub fn exited(&self) -> bool {
        self.host.borrow().exited()
    }

    /// The host's (in-memory) clipboard text; `None` while empty.
    pub fn clipboard(&self) -> Option<String> {
        let host = self.host.borrow();
        let text = host.clipboard();
        (!text.is_empty()).then(|| text.to_owned())
    }

    /// Put `text` on the host's (in-memory) clipboard.
    pub fn set_clipboard(&self, text: &str) {
        self.host.borrow_mut().set_clipboard(text);
    }

    /// Script the answer of the next file, save or folder dialog (`None` =
    /// cancelled). An unscripted dialog is cancelled.
    pub fn respond_file_dialog(&self, path: Option<PathBuf>) {
        self.host.borrow_mut().push_file_dialog(path);
    }

    /// Reply to the next multiple-file dialog; `None` cancels it.
    pub fn respond_files_dialog(&self, paths: Option<Vec<PathBuf>>) {
        self.host.borrow_mut().push_files_dialog(paths);
    }

    /// The paths the App asked to open with the OS default handler
    /// (`platform::open_path`), in order. The test host records them
    /// instead of starting a program and does not read the file system; a
    /// failed request is not recorded.
    pub fn opened_paths(&self) -> Vec<PathBuf> {
        self.host.borrow().opened_paths().to_vec()
    }

    /// The paths the App asked to reveal in the OS file manager
    /// (`platform::reveal_path`), in order, recorded like
    /// [`Screen::opened_paths`].
    pub fn revealed_paths(&self) -> Vec<PathBuf> {
        self.host.borrow().revealed_paths().to_vec()
    }

    /// Make the next platform request (clipboard, dialog, open or reveal)
    /// fail.
    pub fn fail_next_platform_request(&self, reason: &str) {
        self.host.borrow_mut().fail_next_effect(reason);
    }

    /// The focused element, if any.
    pub fn focused(&self) -> Option<Element<'_>> {
        let f = self.host.borrow().focused()?;
        Some(Element {
            screen: self,
            node: f.node,
            region: f.region,
        })
    }

    /// The element under the pointer, if any.
    pub fn hovered(&self) -> Option<Element<'_>> {
        let h = self.host.borrow().hovered()?;
        Some(Element {
            screen: self,
            node: h,
            region: None,
        })
    }

    /// The mouse cursor the window shows over its content. Never
    /// [`Cursor::Auto`]: the window resolves it to a concrete cursor.
    pub fn cursor(&self) -> Cursor {
        self.host.borrow().cursor()
    }

    /// Whether the software keyboard is shown: the host shows it while an
    /// editable text input has the focus (a phone's on-screen keyboard; a
    /// desktop has none and ignores the request, the test window records
    /// it).
    pub fn soft_keyboard_shown(&self) -> bool {
        self.host.borrow().soft_keyboard().0
    }

    /// How often the host asked to show or hide the software keyboard.
    pub fn soft_keyboard_requests(&self) -> u32 {
        self.host.borrow().soft_keyboard().1
    }

    /// The window title the App set.
    pub fn title(&self) -> String {
        self.host.borrow().title()
    }

    /// Number of frame ticks the App has been given.
    pub fn frame_ticks(&self) -> u64 {
        kivact_motion::MotionState::of(&self.rt).stats().ticks
    }

    /// Whether the screen is shown in a real window (not headless).
    pub fn has_window(&self) -> bool {
        self.host.borrow().has_window()
    }
}
