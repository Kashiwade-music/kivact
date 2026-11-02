//! Role: Screenshots and failure artifacts of a `Screen`.
//!
//! A failing test leaves the last frame (`<name>.png`) and the trees
//! (`<name>.txt`: the accessibility tree the host published and the
//! retained tree's semantic tree) in `kivact-test-failures/` under the
//! cargo target directory, named after the test; failure messages name
//! the files (or stderr does). `Screen::screenshot` is the last frame
//! (pixels, hash and PNG), `Element::screenshot` its part under the
//! element's box.

use std::cell::Cell;
use std::fmt::Write as _;
use std::panic::Location;
use std::path::{Path, PathBuf};
use std::time::Duration;

use kivact_engine::testing::{SemanticsNode, Transform};

use super::model::{Element, Screen};
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
    /// The last frame the host presented (settling first).
    pub fn screenshot(&self) -> Screenshot {
        self.run_until_idle();
        self.present();
        match self.frame_now() {
            Some(frame) => frame,
            None => self.fail("screenshot: no frame was drawn (the surface is lost)"),
        }
    }

    fn frame_now(&self) -> Option<Screenshot> {
        let host = self.host.borrow();
        let frame = host.frame()?;
        Some(Screenshot {
            width: frame.width,
            height: frame.height,
            pixels: frame.pixels.to_vec(),
        })
    }

    /// The scale (physical pixels per logical pixel) frames are drawn at.
    pub fn scale(&self) -> f32 {
        self.scale.get()
    }

    /// The accessibility tree the host published, one node per line:
    /// role, name, value, states, test id and bounds (in the window, as
    /// platforms map them: moved back by the scroll offsets above the
    /// node and mapped by the transforms, `SemanticsTree::window_bounds`).
    pub fn accessibility_tree(&self) -> String {
        let host = self.host.borrow();
        let Some(tree) = host.semantics() else {
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
            let b = n.transform.then(above).bounds(n.bounds);
            let (x, y) = (b.x, b.y);
            let mut states = String::new();
            for (on, s) in [
                (n.hidden, " hidden"),
                (n.disabled, " disabled"),
                (n.focusable, " focusable"),
                (n.selected, " selected"),
            ] {
                if on {
                    states.push_str(s);
                }
            }
            if let Some(c) = n.checked {
                let _ = write!(states, " checked={c:?}");
            }
            if let Some(e) = n.expanded {
                let _ = write!(states, " expanded={e}");
            }
            let _ = writeln!(
                out,
                "{}{:?} {:?}{}{}{} [{} {} {}x{}]",
                "  ".repeat(depth),
                n.role,
                n.label,
                n.value
                    .as_ref()
                    .map(|v| format!(" value={v:?}"))
                    .unwrap_or_default(),
                states,
                n.test_id
                    .as_ref()
                    .map(|t| format!(" #{t}"))
                    .unwrap_or_default(),
                x,
                y,
                b.width,
                b.height
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

    /// Write the failure artifacts; returns their paths.
    pub(super) fn write_artifacts(&self) -> Option<(PathBuf, PathBuf)> {
        if self.artifacts_written.replace(true) {
            return None;
        }
        let dir = failure_dir();
        std::fs::create_dir_all(&dir).ok()?;
        let png = dir.join(format!("{}.png", self.name));
        let txt = dir.join(format!("{}.txt", self.name));
        // Reads only (no turn runs while a test is failing). A failure is
        // reported with nothing borrowed: `fail` callers release the host,
        // and an unwinding test has dropped its borrows before the screen.
        match self.host.try_borrow_mut() {
            // The frame of the failing state (drawing runs no client code
            // the host has not already run).
            Ok(mut host) => {
                host.redraw();
            }
            Err(_) => return None,
        }
        if let Some(frame) = self.frame_now() {
            let _ = frame.save_png(&png);
        }
        let mut text = String::new();
        let _ = writeln!(text, "accessibility tree (published by the host):");
        text.push_str(&self.accessibility_tree());
        let _ = writeln!(text, "\nsemantic tree (retained tree):");
        text.push_str(&self.debug());
        let _ = writeln!(text, "\nrecent actions: {:?}", self.actions.borrow());
        let _ = writeln!(
            text,
            "virtual time: {:?}",
            Duration::from_nanos(self.now_ns())
        );
        std::fs::write(&txt, text).ok()?;
        Some((png, txt))
    }

    /// Fail the test with `message`, leaving the failure artifacts (named
    /// in the message).
    #[track_caller]
    pub(super) fn fail(&self, message: &str) -> ! {
        match self.write_artifacts() {
            Some((png, txt)) => panic!(
                "{message}\nfailure artifacts: {} {}",
                png.display(),
                txt.display()
            ),
            None => panic!("{message}"),
        }
    }
}

impl Element<'_> {
    /// The element's part of a screenshot (its layout box, in physical
    /// pixels of the frame).
    pub fn screenshot(&self) -> Screenshot {
        let shot = self.screen.screenshot();
        let rect = self.rect().unwrap_or(Rect::new(0.0, 0.0, 0.0, 0.0));
        let scale = self.screen.scale();
        let px = |v: f32| (v * scale).round().max(0.0) as u32;
        shot.crop(px(rect.x), px(rect.y), px(rect.width), px(rect.height))
    }
}

impl Drop for Screen {
    fn drop(&mut self) {
        if std::thread::panicking() {
            if let Some((png, txt)) = self.write_artifacts() {
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
            self.present();
            if let Ok(host) = self.host.try_borrow() {
                host.sync_window();
            }
            std::thread::sleep(Duration::from_millis(ms));
        }
    }
}
