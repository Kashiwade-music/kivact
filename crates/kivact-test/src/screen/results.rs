//! Role: Define passive results returned by screen inspection.

use kivact_core::view::{CheckedState, Role};
use kivact_engine::testing::{ShellImeText, TestFrame};
use kivact_style::draw::Rect;
use kivact_types as p;
use std::fmt;
use std::ops::Range;
use std::panic::Location;
use std::path::Path;

/// Pixels of a frame the host presented (0x00RRGGBB, row by row, in
/// physical pixels), or of a part of it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Screenshot {
    /// Width in physical pixels.
    pub width: u32,
    /// Height in physical pixels.
    pub height: u32,
    /// `width × height` pixels, 0x00RRGGBB.
    pub pixels: Vec<u32>,
}

impl Screenshot {
    /// The pixel at physical `(x, y)` (0x00RRGGBB).
    pub fn pixel(&self, x: u32, y: u32) -> Option<u32> {
        (x < self.width && y < self.height).then(|| self.pixels[(y * self.width + x) as usize])
    }

    /// A hash of the pixels: equal screenshots hash equal.
    pub fn hash(&self) -> u64 {
        self.frame().hash()
    }

    /// Write the screenshot as a PNG file.
    pub fn save_png(&self, path: impl AsRef<Path>) -> std::io::Result<()> {
        self.frame().save_png(path.as_ref())
    }

    /// The part of the screenshot in the physical rect, clamped to it.
    pub fn crop(&self, x: u32, y: u32, width: u32, height: u32) -> Screenshot {
        let x0 = x.min(self.width);
        let y0 = y.min(self.height);
        let x1 = x.saturating_add(width).min(self.width);
        let y1 = y.saturating_add(height).min(self.height);
        let mut pixels = Vec::with_capacity(((x1 - x0) * (y1 - y0)) as usize);
        for row in y0..y1 {
            let start = (row * self.width + x0) as usize;
            pixels.extend_from_slice(&self.pixels[start..start + (x1 - x0) as usize]);
        }
        Screenshot {
            width: x1 - x0,
            height: y1 - y0,
            pixels,
        }
    }

    fn frame(&self) -> TestFrame {
        TestFrame {
            width: self.width,
            height: self.height,
            pixels: self.pixels.clone().into(),
        }
    }
}

/// What the host answers the platform IME about the focused text input
/// ([`super::model::Screen::ime_text`]). Ranges are in chars of `text`.
#[derive(Clone, Debug, PartialEq)]
pub struct ImeText {
    /// The text around the caret the IME is given, an open composition
    /// included: the input's value when it is short, else the lines around
    /// the caret (a few thousand characters at most).
    pub text: String,
    /// Where `text` starts in the value, in chars (0 when it starts with
    /// it).
    pub start: usize,
    /// The selection (empty: the caret).
    pub selection: Range<usize>,
    /// The open composition, if any.
    pub composition: Option<Range<usize>>,
    /// The boxes of the composed characters (none without a composition),
    /// then the caret's, in window coordinates.
    pub char_rects: Vec<Rect>,
    /// The input's text area in window coordinates.
    pub area: Rect,
}

/// The char offset of each UTF-16 offset of `text` (the end included); a
/// unit inside a surrogate pair maps to its char.
fn chars_of_units(text: &str) -> Vec<usize> {
    let mut out = Vec::with_capacity(text.len() + 1);
    for (i, c) in text.chars().enumerate() {
        out.extend(std::iter::repeat_n(i, c.len_utf16()));
    }
    out.push(text.chars().count());
    out
}

impl ImeText {
    /// The IME text `t`, which starts at char `start` of the value.
    pub(super) fn from_host(t: ShellImeText, start: usize) -> Self {
        let chars = chars_of_units(&t.text);
        let at = |unit: usize| chars.get(unit).copied().unwrap_or(chars[chars.len() - 1]);
        let range = |(start, end): (usize, usize)| at(start)..at(end);
        // One box per char: the first unit's (a surrogate pair repeats it),
        // then the caret's.
        let rects: Vec<Rect> = t.rects;
        let mut char_rects = Vec::new();
        let mut last = None;
        for (i, r) in rects.iter().enumerate().take(rects.len().saturating_sub(1)) {
            let c = at(t.rects_from + i);
            if last != Some(c) {
                char_rects.push(*r);
                last = Some(c);
            }
        }
        char_rects.extend(rects.last().copied());
        ImeText {
            start,
            selection: range(t.selection),
            composition: t.composition.map(range),
            char_rects,
            area: t.area,
            text: t.text,
        }
    }
}

pub use kivact_types::{LayoutCheck, LayoutLimit, LayoutRule};

/// A layout defect of one text.
#[derive(Clone, Debug, PartialEq)]
pub struct LayoutViolation {
    /// Canonical geometry, rule and magnitude of the finding.
    pub finding: p::LayoutFinding,
    /// The role the host publishes for the text (`None`: it is not
    /// exposed).
    pub role: Option<Role>,
    /// The accessible name of the text.
    pub name: String,
    /// What the text shows (a text input: its value, else its
    /// placeholder).
    pub text: String,
    /// The test id of the text, if it has one.
    pub test_id: Option<String>,
    /// Where the text was created: the call of the builder that created it,
    /// else of its nearest located ancestor.
    pub created_at: &'static Location<'static>,
}

impl LayoutViolation {
    pub fn rule(&self) -> LayoutRule {
        self.finding.rule
    }
    pub fn amount(&self) -> f64 {
        self.finding.amount
    }
    pub fn text_rect(&self) -> Rect {
        self.finding.text_rect
    }
    pub fn limit(&self) -> Rect {
        self.finding.limit_rect
    }
}

impl fmt::Display for LayoutViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let r = |r: Rect| format!("[{:.1} {:.1} {:.1}x{:.1}]", r.x, r.y, r.width, r.height);
        let what = match self.rule() {
            LayoutRule::Overflow {
                into: LayoutLimit::OwnBox,
            } => {
                format!("overflows its own box by {:.1}px", self.amount())
            }
            LayoutRule::Overflow {
                into: LayoutLimit::VisualParent,
            } => {
                format!("overflows its parent by {:.1}px", self.amount())
            }
            LayoutRule::Wrapped { lines } => format!(
                "is wrapped into {lines} lines ({:.1}px below the first)",
                self.amount()
            ),
            LayoutRule::OffCenter => {
                format!("is {:.1}px off the center of its parent", self.amount())
            }
            LayoutRule::ClippedWithoutEllipsis => {
                format!("is cut {:.1}px by a clip with no ellipsis", self.amount())
            }
        };
        write!(
            f,
            "{}: {:?}{} {what}: text {} limit {}",
            self.created_at,
            self.text,
            self.test_id
                .as_ref()
                .map(|t| format!(" #{t}"))
                .unwrap_or_default(),
            r(self.text_rect()),
            r(self.limit()),
        )?;
        if let Some(role) = self.role {
            write!(f, " ({role:?})")?;
        }
        Ok(())
    }
}

/// Semantic snapshot of a node.
#[derive(Clone, Debug, PartialEq)]
pub struct SemanticNode {
    pub role: Option<Role>,
    pub name: String,
    pub value: Option<String>,
    pub test_id: Option<String>,
    pub focused: bool,
    pub disabled: bool,
    pub selected: bool,
    pub checked: Option<CheckedState>,
    /// Where the node was created: the call of the builder that created it
    /// (`text(..)`, `button(..)`, ...). Nodes a builder creates internally
    /// or from a plain string child (a button's label, `"a"` in a tuple) and
    /// draw regions carry the location of the nearest located ancestor; the
    /// root has none. Two instances of the same component share their
    /// locations; tell them apart with `Screen::within`.
    pub at: Option<&'static Location<'static>>,
    pub children: Vec<SemanticNode>,
}
