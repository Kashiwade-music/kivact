//! Role: Test controls and component building.
//!
//! Host controls of a `Screen`: environment and surface changes, resize,
//! scale, close, clipboard, platform scripting and focus.

use std::path::PathBuf;
use std::time::Duration;

use kivact_types as p;

use kivact_engine::testing::Insets;

use super::model::{FIND_TIMEOUT, Locator, Screen};
use kivact_style::Cursor;

impl Screen {
    /// Logical content sizes requested through `platform::request_window_size`.
    pub fn requested_sizes(&self) -> Vec<kivact_core::geometry::Size> {
        self.host()
            .requested_sizes(self.window)
            .iter()
            .map(|s| kivact_core::geometry::Size {
                width: s.width,
                height: s.height,
            })
            .collect()
    }

    /// Report native window focus loss or restoration (cancels pointer capture).
    pub fn set_window_focused(&self, focused: bool) {
        self.send(kivact_engine::testing::ShellEvent::Focused(focused));
        self.run_until_idle();
    }

    /// The key presses nobody handled since the last call (what an embedded
    /// editor would give back to its host).
    pub fn unhandled_keys(&self) -> Vec<kivact_core::view::KeyCode> {
        self.host().unhandled_keys(self.window)
    }
    /// The host reports reduced motion on or off (`EnvChanged`).
    pub fn set_reduced_motion(&self, on: bool) {
        self.log(format!("reduced motion {on}"));
        self.host_mut().set_reduced_motion(self.window, on);
        self.run_until_idle();
    }

    /// The host reports a dark (or light) system theme (`EnvChanged`).
    /// Report platform pointer preferences and primary device precision.
    pub fn set_pointer_settings(&self, settings: p::PointerSettings) {
        self.app.host.borrow_mut().set_pointer_settings(settings);
        self.run_until_idle();
    }

    pub fn set_dark(&self, dark: bool) {
        self.log(format!("dark {dark}"));
        self.host_mut().set_theme(
            self.window,
            if dark {
                p::ThemeHint::Dark
            } else {
                p::ThemeHint::Light
            },
        );
        self.run_until_idle();
    }

    /// The host reports another system locale (a BCP 47 tag; `None`: the
    /// system names none) (`EnvChanged`). An App that named no locale
    /// (`App::locale`) follows it; one that did ignores it.
    pub fn set_host_locale(&self, locale: Option<&str>) {
        self.log(format!("host locale {locale:?}"));
        self.host_mut().set_locale(self.window, locale);
        self.run_until_idle();
    }

    /// The window's safe area changes, as a phone's system UI (status bar,
    /// notch) does when it rotates (`EnvChanged`; read by
    /// `platform::safe_area`).
    pub fn set_safe_area(&self, insets: kivact_core::geometry::Insets) {
        self.log(format!("safe area {insets:?}"));
        self.host_mut().set_safe_area(
            self.window,
            Insets {
                top: insets.top,
                right: insets.right,
                bottom: insets.bottom,
                left: insets.left,
            },
        );
        self.run_until_idle();
    }

    /// The platform suspends the App (sends it to the background, as
    /// Android and iOS do): nothing is drawn and the frame clock stops
    /// (`platform::surface_active` is false) until [`Screen::resume`]. With
    /// `destroys_surface` the window's surface goes too, as on Android.
    pub fn suspend(&self, destroys_surface: bool) {
        self.log(format!("suspended (surface destroyed: {destroys_surface})"));
        self.app.host.borrow_mut().suspend(destroys_surface);
        self.run_until_idle();
    }

    /// The suspended App comes back to the foreground (with a new surface
    /// if it lost its own) and draws again.
    pub fn resume(&self) {
        self.log("resumed".into());
        self.app.host.borrow_mut().resume();
        self.run_until_idle();
    }

    /// The platform warns that memory is short.
    pub fn memory_warning(&self) {
        self.log("memory warning".into());
        self.app.host.borrow_mut().memory_warning();
        self.run_until_idle();
    }

    /// The window's surface is lost, as when the window is minimized or
    /// the platform destroys it (no frames, no frame ticks).
    pub fn lose_surface(&self) {
        self.log("surface lost".into());
        self.host_mut().set_surface(self.window, false);
        self.run_until_idle();
    }

    /// The window is restored: the surface is back.
    pub fn restore_surface(&self) {
        self.log("surface restored".into());
        self.host_mut().set_surface(self.window, true);
        self.run_until_idle();
    }

    /// The window's GPU device is lost, as a device failure makes it; the
    /// frames go on, and nothing happens on a window that draws on the CPU.
    pub fn lose_gpu_device(&self) {
        self.log("gpu device lost".into());
        self.host_mut().lose_gpu_device(self.window);
        self.run_until_idle();
    }

    /// Resize the window to `width` × `height` logical pixels.
    pub fn resize(&self, width: f64, height: f64) {
        self.log(format!("resize {width} x {height}"));
        self.host_mut().resize(self.window, width, height);
        self.run_until_idle();
    }

    /// Change the scale (physical pixels per logical pixel), as when the
    /// window moves to a monitor with another DPI.
    pub fn set_scale(&self, scale: f64) {
        self.log(format!("scale {scale}"));
        self.host_mut().set_scale(self.window, scale);
        self.run_until_idle();
    }

    /// Make the close request of this handle's window. For the main window
    /// it asks the App to close and returns whether the App exited (queued
    /// actions drained or abandoned after the 3 s deadline, advancing
    /// virtual time as needed); for another window it returns whether that
    /// window closed.
    pub fn request_close(&self) -> bool {
        self.log("close requested".into());
        if self.window == p::WindowId::MAIN {
            self.host_mut().request_close(self.window);
            self.run_until_idle();
            for _ in 0..1000 {
                if self.app.host.borrow().exited() {
                    return true;
                }
                self.advance(Duration::from_millis(100));
            }
            self.app.host.borrow().exited()
        } else {
            self.host_mut().request_close(self.window);
            self.run_until_idle();
            !self.is_open()
        }
    }

    /// Whether the App exited (it accepted a close request).
    pub fn has_exited(&self) -> bool {
        self.app.host.borrow().exited()
    }

    /// The host's (in-memory) clipboard text; `None` while empty.
    pub fn clipboard(&self) -> Option<String> {
        let host = self.app.host.borrow();
        let text = host.clipboard();
        (!text.is_empty()).then(|| text.to_owned())
    }

    /// Put `text` on the host's (in-memory) clipboard.
    pub fn set_clipboard(&self, text: &str) {
        self.app.host.borrow_mut().set_clipboard(text);
    }

    /// Script the answer of the next file, save or folder dialog (`None` =
    /// cancelled). An unscripted dialog is cancelled.
    pub fn respond_file_dialog(&self, path: Option<PathBuf>) {
        self.app.host.borrow_mut().push_file_dialog(path);
    }

    /// Reply to the next multiple-file dialog; `None` cancels it.
    pub fn respond_files_dialog(&self, paths: Option<Vec<PathBuf>>) {
        self.app.host.borrow_mut().push_files_dialog(paths);
    }

    /// The paths the App asked to open with the OS default handler
    /// (`platform::open_path`), in order. The test host records them
    /// instead of starting a program and does not read the file system; a
    /// failed request is not recorded.
    pub fn opened_paths(&self) -> Vec<PathBuf> {
        self.app.host.borrow().opened_paths().to_vec()
    }

    /// The paths the App asked to reveal in the OS file manager
    /// (`platform::reveal_path`), in order, recorded like
    /// [`Screen::opened_paths`].
    pub fn revealed_paths(&self) -> Vec<PathBuf> {
        self.app.host.borrow().revealed_paths().to_vec()
    }

    /// Make the next platform request (clipboard, dialog, open or reveal)
    /// fail.
    pub fn fail_next_platform_request(&self, reason: &str) {
        self.app.host.borrow_mut().fail_next_effect(reason);
    }

    /// The focused element, if any.
    pub fn focused(&self) -> Option<Locator<'_>> {
        let f = self.host().focused(self.window)?;
        Some(Locator {
            screen: self,
            node: f.node,
            region: f.region,
        })
    }

    /// The element under the pointer, if any.
    pub fn hovered(&self) -> Option<Locator<'_>> {
        let h = self.host().hovered(self.window)?;
        Some(Locator {
            screen: self,
            node: h,
            region: None,
        })
    }

    /// The mouse cursor the window shows over its content. Never
    /// [`Cursor::Auto`]: the window resolves it to a concrete cursor.
    pub fn cursor(&self) -> Cursor {
        self.host().cursor(self.window)
    }

    /// Whether the software keyboard is shown: the host shows it while an
    /// editable text input has the focus (a phone's on-screen keyboard; a
    /// desktop has none and ignores the request, the test window records
    /// it).
    pub fn is_soft_keyboard_shown(&self) -> bool {
        self.host().soft_keyboard(self.window).0
    }

    /// How often the host asked to show or hide the software keyboard.
    pub fn soft_keyboard_requests(&self) -> u32 {
        self.host().soft_keyboard(self.window).1
    }

    /// The window title the App set.
    pub fn title(&self) -> String {
        self.host().title(self.window)
    }

    /// The native child window filling a window opened with
    /// `foreign_window`, once the engine created it.
    pub fn foreign_child(&self) -> Option<p::NativeWindow> {
        self.host().foreign_child(self.window)
    }

    /// Whether the user can resize this window.
    pub fn is_resizable(&self) -> bool {
        self.host().is_resizable(self.window)
    }

    /// Number of frame ticks the App has been given.
    pub fn frame_ticks(&self) -> u64 {
        self.inspect().motion_ticks()
    }

    /// Whether this window is shown in a real window (not headless).
    pub fn is_mirrored(&self) -> bool {
        self.host().has_window(self.window)
    }

    /// A handle to the open window titled `title`, waiting (advancing
    /// virtual time a frame at a time, at most 5 s) until it is open.
    #[track_caller]
    pub fn window(&self, title: &str) -> Screen {
        let start = self.now_ns();
        let limit = FIND_TIMEOUT.as_nanos() as u64;
        loop {
            self.run_until_idle();
            let open: Vec<(p::WindowId, String)> = {
                let host = self.app.host.borrow();
                host.windows()
                    .into_iter()
                    .filter_map(|id| host.window_title(id).map(|title| (id, title)))
                    .collect()
            };
            let found: Vec<p::WindowId> = open
                .iter()
                .filter(|(_, open)| open == title)
                .map(|(id, _)| *id)
                .collect();
            if let [window] = found[..] {
                return Screen {
                    app: self.app.clone(),
                    window,
                };
            }
            if found.len() > 1 {
                self.fail(&format!(
                    "{} windows are titled {title:?} (ambiguous)",
                    found.len()
                ));
            }
            if self.now_ns().saturating_sub(start) >= limit {
                let titles: Vec<&str> = open.iter().map(|(_, title)| title.as_str()).collect();
                self.fail(&format!(
                    "window: no window is titled {title:?} within {FIND_TIMEOUT:?} (virtual); \
                     open windows: {titles:?}"
                ));
            }
            self.step();
        }
    }

    /// Handles to the open windows, ascending by id.
    pub fn windows(&self) -> Vec<Screen> {
        self.app
            .host
            .borrow()
            .windows()
            .into_iter()
            .map(|window| Screen {
                app: self.app.clone(),
                window,
            })
            .collect()
    }

    /// Whether this handle's window is still open.
    pub fn is_open(&self) -> bool {
        self.app.host.borrow().windows().contains(&self.window)
    }
}
