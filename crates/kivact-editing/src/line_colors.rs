//! Role: Per-line coloring hooks and the state carried from one line to the next.

use crate::edit_state::floor_boundary;
use kivact_types::{Rgba, TextDecoration};
use std::any::Any;
use std::ops::Range;
use std::rc::Rc;

/// A value a [`LineState`] holds: compared by value.
trait StateValue {
    fn as_any(&self) -> &dyn Any;
    fn same(&self, other: &dyn StateValue) -> bool;
}

impl<S: PartialEq + 'static> StateValue for S {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn same(&self, other: &dyn StateValue) -> bool {
        other.as_any().downcast_ref::<S>() == Some(self)
    }
}

/// What coloring one line leaves for the next (a block comment still
/// open, say): opaque to the engine, which compares it to know whether
/// the lines after a changed line must be colored again. The first line
/// starts from the default (empty) state.
#[derive(Clone, Default)]
pub struct LineState(Option<Rc<dyn StateValue>>);

impl LineState {
    /// A state holding `state`.
    pub fn new<S: PartialEq + 'static>(state: S) -> Self {
        Self(Some(Rc::new(state)))
    }

    /// The value this state holds, if it holds an `S` (`None` for the
    /// first line's state).
    pub fn get<S: 'static>(&self) -> Option<&S> {
        self.0.as_ref()?.as_any().downcast_ref()
    }
}

impl PartialEq for LineState {
    fn eq(&self, other: &Self) -> bool {
        match (&self.0, &other.0) {
            (None, None) => true,
            (Some(a), Some(b)) => Rc::ptr_eq(a, b) || a.same(&**b),
            _ => false,
        }
    }
}

impl std::fmt::Debug for LineState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(if self.0.is_some() {
            "LineState(..)"
        } else {
            "LineState(start)"
        })
    }
}

// xtask-allow(state-fields): Foreground, background and decoration independently override one resolved byte range.
/// Resolved paint for one non-overlapping line byte range.
#[derive(Clone, Debug, PartialEq)]
pub struct LineStyle {
    pub range: Range<usize>,
    pub color: Option<Rgba>,
    pub background: Option<Rgba>,
    pub decoration: Option<TextDecoration>,
}

/// What line colors run: the colored byte ranges of one line, which
/// excludes its line break, from the state the line before left, and the
/// state it leaves.
type Colorize = dyn Fn(&str, &LineState) -> (Vec<LineStyle>, LineState);

/// Colors for the lines of a native editor's text, computed by kivact and
/// run synchronously while the engine draws the editor. Two are equal when
/// they are the same function object.
#[derive(Clone)]
pub struct LineColors(Rc<Colorize>);

impl LineColors {
    /// Color each line with `f`: `(range, color)` in the line and the
    /// state the line leaves, from the state the line before left.
    pub fn new(f: impl Fn(&str, &LineState) -> (Vec<LineStyle>, LineState) + 'static) -> Self {
        Self(Rc::new(f))
    }

    /// The sorted, non-overlapping colored byte ranges of `line` (clamped
    /// to its char boundaries) and the state it leaves, from `input`.
    pub fn line(&self, line: &str, input: &LineState) -> (Vec<LineStyle>, LineState) {
        let (mut ranges, output) = (self.0)(line, input);
        ranges.sort_by_key(|r| r.range.start);
        let mut out = Vec::with_capacity(ranges.len());
        let mut end = 0;
        for mut style in ranges {
            let r = &style.range;
            let a = floor_boundary(line, r.start).max(end);
            let b = floor_boundary(line, r.end);
            if a < b {
                style.range = a..b;
                out.push(style);
                end = b;
            }
        }
        (out, output)
    }
}

impl PartialEq for LineColors {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl std::fmt::Debug for LineColors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("LineColors(..)")
    }
}

/// Resolved document-range paint transported to the editor.
#[derive(Clone, Debug, PartialEq)]
pub struct RangeDecoration {
    pub range: Range<usize>,
    pub kind: RangeDecorationKind,
    pub color: Rgba,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RangeDecorationKind {
    Underline(kivact_types::TextDecorationStyle),
    Outline,
    Background,
}
