//! Role: Screenshots and failure artifacts of a `Screen`.
//!
//! A failing test leaves the last frame (`<name>.png`) and the trees
//! (`<name>.txt`: the accessibility tree the host published and the
//! retained tree's semantic tree) in `kivact-test-failures/` under the
//! cargo target directory, named after the test; a window beyond the main
//! one is named `<name>-w<id>`. Failure messages name the files (or stderr
//! does). `Screen::screenshot` is the last frame (pixels, hash and PNG),
//! `Locator::screenshot` its part under the element's box.

use std::cell::Cell;
use std::fmt::Write as _;
use std::panic::Location;
use std::path::{Path, PathBuf};
use std::time::Duration;

use kivact_engine::testing::{SemanticsNode, Transform};
use kivact_types::WindowId;

use super::model::{Locator, Screen, TestApp};
use super::results::Screenshot;
use kivact_style::draw::Rect;

thread_local! {
    /// Screens rendered on this thread (libtest runs each test on a thread
    /// of its own): the second screen of a test gets `-2`, ...
    static SCREENS: Cell<u32> = const { Cell::new(0) };
}

/// The artifact name of a screen rendered at `at`: the test's (its
/// thread's name), else the render location.
pub(super) fn test_name(at: &'static Location<'static>) -> String {
    let thread = std::thread::current();
    let base = match thread.name() {
        Some(name) if name != "main" => name.to_string(),
        _ => format!("{}-{}", at.file(), at.line()),
    };
    let n = SCREENS.with(|c| {
        c.set(c.get() + 1);
        c.get()
    });
    let base: String = base
        .chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' => c,
            _ => '_',
        })
        .collect();
    if n == 1 { base } else { format!("{base}-{n}") }
}

/// Where failure artifacts go: `kivact-test-failures/` in the cargo target
/// directory (`CARGO_TARGET_DIR`, else the one holding the test binary),
/// else in the temporary directory.
pub fn failure_dir() -> PathBuf {
    let target = std::env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .or_else(|| {
            let exe = std::env::current_exe().ok()?;
            let deps = exe.ancestors().find(|a| a.ends_with("deps"))?;
            deps.parent()?.parent().map(Path::to_path_buf)
        })
        .unwrap_or_else(std::env::temp_dir);
    target.join("kivact-test-failures")
}

impl Screen {
    /// Save catalog evidence when KIVACT_CATALOG_EVIDENCE names an output directory.
    #[track_caller]
    pub fn evidence(&self, name: &str) {
        let Some(dir) = std::env::var_os("KIVACT_CATALOG_EVIDENCE") else {
            return;
        };
        let dir = PathBuf::from(dir);
        std::fs::create_dir_all(&dir).expect("invariant: catalog evidence directory is writable");
        self.screenshot()
            .save_png(dir.join(format!("{name}.png")))
            .expect("invariant: catalog screenshot is writable");
        std::fs::write(
            dir.join(format!("{name}.txt")),
            format!(
                "accessibility tree:\n{}\ndebug tree:\n{}",
                self.accessibility_tree(),
                self.debug()
            ),
        )
        .expect("invariant: catalog evidence is writable");
    }

    /// The last frame the host presented (settling first).
    pub fn screenshot(&self) -> Screenshot {
        self.run_until_idle();
        match self.host().frame(self.window) {
            Some(frame) => Screenshot {
                width: frame.width,
                height: frame.height,
                pixels: frame.pixels.to_vec(),
            },
            None => self.fail("screenshot: no frame was drawn (the surface is lost)"),
        }
    }

    /// The scale (physical pixels per logical pixel) frames are drawn at.
    pub fn scale(&self) -> f64 {
        self.app
            .host
            .borrow()
            .scale(self.window)
            .expect("invariant: the window is open")
    }

    /// The accessibility tree the host published, one node per line:
    /// role, name, value, states, test id and bounds (in the window, as
    /// platforms map them: moved back by the scroll offsets above the
    /// node and mapped by the transforms, `SemanticsTree::window_bounds`).
    pub fn accessibility_tree(&self) -> String {
        self.check_open();
        self.app.accessibility_tree(self.window)
    }

    /// Fail the test with `message`, leaving the failure artifacts (named
    /// in the message).
    #[track_caller]
    pub(super) fn fail(&self, message: &str) -> ! {
        self.app.fail(self.window, message)
    }
}

impl Locator<'_> {
    /// The element's part of a screenshot (its layout box, in physical
    /// pixels of the frame).
    pub fn screenshot(&self) -> Screenshot {
        let shot = self.screen.screenshot();
        let rect = self.rect().unwrap_or(Rect::new(0.0, 0.0, 0.0, 0.0));
        let scale = self.screen.scale();
        let px = |v: f64| (v * scale).round().max(0.0) as u32;
        shot.crop(px(rect.x), px(rect.y), px(rect.width), px(rect.height))
    }
}

impl TestApp {
    /// The last frame the host presented for `window`.
    fn frame_now(&self, window: WindowId) -> Option<Screenshot> {
        let host = self.host.borrow();
        let frame = host.frame(window)?;
        Some(Screenshot {
            width: frame.width,
            height: frame.height,
            pixels: frame.pixels.to_vec(),
        })
    }

    /// The accessibility tree the host published for `window`, one node per
    /// line (see [`Screen::accessibility_tree`]).
    fn accessibility_tree(&self, window: WindowId) -> String {
        let host = self.host.borrow();
        let Some(tree) = host.semantics(window) else {
            return "(no accessibility tree published)\n".into();
        };
        let mut out = String::new();
        // `above` maps the parent's space to the window (as
        // `SemanticsTree::window_bounds` composes it).
        fn walk(
            nodes: &[SemanticsNode],
            id: u64,
            depth: usize,
            above: Transform,
            out: &mut String,
        ) {
            let Some(n) = nodes.iter().find(|n| n.id.0 == id) else {
                return;
            };
            let placed = n.bounds.map(|r| n.transform.then(above).bounds(r));
            let geometry = match placed {
                Some(b) => format!("{} {} {}x{}", b.x, b.y, b.width, b.height),
                None => "unplaced".to_owned(),
            };
            let mut states = String::new();
            for (on, s) in [
                (n.semantics.hidden, " hidden"),
                (n.semantics.disabled, " disabled"),
                (n.semantics.focusable, " focusable"),
                (n.semantics.selected.unwrap_or(false), " selected"),
            ] {
                if on {
                    states.push_str(s);
                }
            }
            if let Some(c) = n.semantics.checked {
                let _ = write!(states, " checked={c:?}");
            }
            if let Some(e) = n.semantics.expanded {
                let _ = write!(states, " expanded={e}");
            }
            let _ = writeln!(
                out,
                "{}{:?} {:?}{}{}{} [{}]",
                "  ".repeat(depth),
                n.semantics.role.unwrap_or(kivact_types::Role::Generic),
                n.semantics.name.clone().unwrap_or_default(),
                n.semantics
                    .value_text
                    .as_ref()
                    .map(|v| format!(" value={v:?}"))
                    .unwrap_or_default(),
                states,
                n.semantics
                    .test_id
                    .as_ref()
                    .map(|t| format!(" #{t}"))
                    .unwrap_or_default(),
                geometry
            );
            let below = Transform::translate(-n.scroll.x, -n.scroll.y)
                .then(n.transform)
                .then(above);
            for c in &n.children {
                walk(nodes, c.0, depth + 1, below, out);
            }
        }
        walk(&tree.nodes, tree.root.0, 0, Transform::IDENTITY, &mut out);
        if let Some(f) = tree.focus {
            let _ = writeln!(out, "focus: {}", f.0);
        }
        out
    }

    /// Write `window`'s failure artifacts; returns their paths.
    pub(super) fn write_artifacts(&self, window: WindowId) -> Option<(PathBuf, PathBuf)> {
        if !self.artifacts_written.note() {
            return None;
        }
        let dir = failure_dir();
        std::fs::create_dir_all(&dir).ok()?;
        let name = if window == WindowId::MAIN {
            self.name.clone()
        } else {
            format!("{}-w{}", self.name, window.get())
        };
        let png = dir.join(format!("{name}.png"));
        let txt = dir.join(format!("{name}.txt"));
        // Reads only (no turn runs while a test is failing). A failure is
        // reported with nothing borrowed: `fail` callers release the host,
        // and an unwinding test has dropped its borrows before the app.
        match self.host.try_borrow_mut() {
            // The frame of the failing state (drawing runs no client code
            // the host has not already run).
            Ok(mut host) => {
                host.redraw();
            }
            Err(_) => return None,
        }
        if let Some(frame) = self.frame_now(window) {
            let _ = frame.save_png(&png);
        }
        let mut text = String::new();
        let _ = writeln!(text, "accessibility tree (published by the host):");
        text.push_str(&self.accessibility_tree(window));
        if self.host.borrow().windows().contains(&window) {
            let _ = writeln!(text, "\nsemantic tree (retained tree):");
            text.push_str(&self.debug_tree(window));
        }
        let _ = writeln!(text, "\nrecent actions: {:?}", self.actions.borrow());
        let _ = writeln!(
            text,
            "virtual time: {:?}",
            Duration::from_nanos(self.host.borrow().now_ns())
        );
        std::fs::write(&txt, text).ok()?;
        Some((png, txt))
    }

    /// Fail the test with `message` for `window`, leaving its failure
    /// artifacts (named in the message).
    #[track_caller]
    pub(super) fn fail(&self, window: WindowId, message: &str) -> ! {
        match self.write_artifacts(window) {
            Some((png, txt)) => panic!(
                "{message}\nfailure artifacts: {} {}",
                png.display(),
                txt.display()
            ),
            None => panic!("{message}"),
        }
    }
}

impl Drop for TestApp {
    fn drop(&mut self) {
        if std::thread::panicking() {
            if let Some((png, txt)) = self.write_artifacts(WindowId::MAIN) {
                #[allow(
                    clippy::print_stderr,
                    reason = "the test harness shows a failing test's stderr: where its artifacts are"
                )]
                {
                    eprintln!(
                        "kivact test: failure artifacts: {} {}",
                        png.display(),
                        txt.display()
                    );
                }
            }
        } else if let Some(ms) = std::env::var("KIVACT_TEST_HOLD_MS")
            .ok()
            .and_then(|v| v.parse().ok())
        {
            self.run_until_idle(WindowId::MAIN);
            if let Ok(host) = self.host.try_borrow() {
                host.sync_window(WindowId::MAIN);
            }
            std::thread::sleep(Duration::from_millis(ms));
        }
    }
}
