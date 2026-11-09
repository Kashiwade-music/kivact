//! Role: CSS Grid tracks and item placement.
//!
//! CSS Grid in styles: a grid container's tracks
//! (`grid_template_columns((248, Track::fr(1)))`) and an item's place
//! (`grid_column((1, -1))`, `grid_column(GridPlacement::span(4))`),
//! resolved into the engine's grid props.
//!
//! `grid(children)` is the only grid box (`Display` is not public). A track
//! list is one [`Track`], a tuple or a `Vec`, of px, percent, calc, `auto`,
//! `MinContent`, `MaxContent`, `fr`, `minmax`, `repeat` and
//! auto-fill/auto-fit, for the template and auto columns and rows. A
//! [`GridPlacement`] is a line number, a span or a pair (negative lines
//! count from the end) for `grid_column` and `grid_row`. There are also
//! `grid_auto_flow`, `justify_items`, `justify_self` and `place_items`.
//!
//! Invariant: this module converts a style's grid group once into the
//! engine's `GridProps`, and only a grid container or a grid item carries
//! `LayoutProps::grid` (an `Option<Box<..>>`), so other nodes pay nothing.

use kivact_types as w;
use kivact_types::Align;

use super::length::Length;

/// One bound of a track's size.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Breadth {
    Length(Length),
    Fr(f64),
}

/// One entry of a grid's track list (CSS `grid-template-columns`): a
/// track's size (a number of px, `pct(..)`, a calc length, `auto()`,
/// [`Length::MinContent`] .., [`Track::fr`], [`Track::minmax`]) or tracks
/// repeated ([`Track::repeat`], [`Track::repeat_auto_fill`],
/// [`Track::repeat_auto_fit`]). Where a track list is taken, one track, a
/// tuple of them (`(248, Track::fr(1))`) or a `Vec<Track>` is.
///
/// ```
/// use kivact::prelude::*;
/// use kivact::style::{auto, pct};
/// use kivact::style::Track;
/// // grid-template-columns: repeat(auto-fill, minmax(330px, 1fr))
/// let cards = style()
///     .grid_template_columns(Track::repeat_auto_fill(Track::minmax(330, Track::fr(1))));
/// // grid-template-columns: 248px 1fr
/// let shell = style().grid_template_columns((248, Track::fr(1)));
/// // grid-template-columns: minmax(70px, 38%) 1fr 20px
/// let props = style().grid_template_columns((Track::minmax(70, pct(38)), Track::fr(1), 20));
/// # let _ = (cards, shell, props);
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Track(TrackKind);

#[derive(Clone, Debug, PartialEq)]
enum TrackKind {
    Size { min: Breadth, max: Breadth },
    Repeat(Repeat, Vec<Track>),
}

/// How often a repeat repeats its tracks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Repeat {
    /// This many times (CSS `repeat(4, ..)`).
    Times(u16),
    /// As many times as fit the grid's width (or height), keeping empty
    /// tracks (CSS `auto-fill`).
    AutoFill,
    /// As many times as fit, collapsing the empty tracks (CSS `auto-fit`).
    AutoFit,
}

impl Track {
    /// A track taking `n` shares of the grid's free space (CSS `1fr`: at
    /// least its content's minimum, `minmax(auto, 1fr)`; write
    /// `Track::minmax(0, Track::fr(1))` for tracks that share the space
    /// exactly).
    pub fn fr(n: impl Into<f64>) -> Track {
        let f = Breadth::Fr(n.into());
        Track(TrackKind::Size { min: f, max: f })
    }

    /// A track between `min` and `max` (CSS `minmax(min, max)`): `min` is a
    /// length (a share, [`Track::fr`], counts as `auto` there), `max` a
    /// length or a share.
    pub fn minmax(min: impl Into<Track>, max: impl Into<Track>) -> Track {
        let bound = |t: Track, lower: bool| match t.0 {
            TrackKind::Size { min, max } => {
                if lower {
                    min
                } else {
                    max
                }
            }
            TrackKind::Repeat(..) => Breadth::Length(Length::Auto),
        };
        Track(TrackKind::Size {
            min: bound(min.into(), true),
            max: bound(max.into(), false),
        })
    }

    /// `tracks` repeated `count` times (CSS `repeat(count, tracks)`; at
    /// least once): `Track::repeat(12, Track::minmax(0, Track::fr(1)))`. A
    /// repeat inside a repeat is left out.
    pub fn repeat(count: u16, tracks: impl Into<Tracks>) -> Track {
        Track::repeated(Repeat::Times(count.max(1)), tracks)
    }

    /// `tracks` repeated as many times as fit the grid's width (or height),
    /// keeping the empty tracks (CSS `repeat(auto-fill, tracks)`). The
    /// tracks must have a definite size or minimum.
    pub fn repeat_auto_fill(tracks: impl Into<Tracks>) -> Track {
        Track::repeated(Repeat::AutoFill, tracks)
    }

    /// `tracks` repeated as many times as fit, collapsing the empty tracks
    /// (CSS `repeat(auto-fit, tracks)`).
    pub fn repeat_auto_fit(tracks: impl Into<Tracks>) -> Track {
        Track::repeated(Repeat::AutoFit, tracks)
    }

    fn repeated(count: Repeat, tracks: impl Into<Tracks>) -> Track {
        Track(TrackKind::Repeat(count, tracks.into().0))
    }
}

impl From<Length> for Track {
    fn from(l: Length) -> Self {
        let b = Breadth::Length(l);
        Track(TrackKind::Size { min: b, max: b })
    }
}

macro_rules! track_from_number {
    ($($t:ty),*) => {$(
        impl From<$t> for Track {
            fn from(v: $t) -> Self {
                Track::from(Length::from(v))
            }
        }
    )*};
}

track_from_number!(i32, u32, f32, f64);

/// A grid's track list (CSS `grid-template-columns: 248px 1fr`): one
/// [`Track`], a tuple of them (`(248, Track::fr(1))`) or a `Vec`. The
/// setters take anything that converts to it; it is not named.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Tracks(Vec<Track>);

macro_rules! one_track {
    ($($t:ty),*) => {$(
        impl From<$t> for Tracks {
            fn from(t: $t) -> Self {
                Tracks(vec![Track::from(t)])
            }
        }
    )*};
}

one_track!(Track, Length, i32, u32, f32, f64);

impl From<Vec<Track>> for Tracks {
    fn from(v: Vec<Track>) -> Self {
        Tracks(v)
    }
}

macro_rules! track_list {
    ($($t:ident),+) => {
        impl<$($t: Into<Track>),+> From<($($t,)+)> for Tracks {
            #[allow(non_snake_case, reason = "the tuple's elements are named by their types")]
            fn from(($($t,)+): ($($t,)+)) -> Self {
                Tracks(vec![$($t.into()),+])
            }
        }
    };
}

track_list!(A, B);
track_list!(A, B, C);
track_list!(A, B, C, D);
track_list!(A, B, C, D, E);
track_list!(A, B, C, D, E, F);
track_list!(A, B, C, D, E, F, G);
track_list!(A, B, C, D, E, F, G, H);
track_list!(A, B, C, D, E, F, G, H, I);
track_list!(A, B, C, D, E, F, G, H, I, J);

/// One end of a grid item's place: a line number (from 1 at the start,
/// from -1 at the end) or a span.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
struct GridLine(w::GridPos);

/// Where a grid item goes along one axis (CSS `grid-column`, `grid-row`):
/// a start line (`2`, [`GridPlacement::line`]), a span
/// ([`GridPlacement::span`]), or `(start, end)` of those: `(1, -1)` spans
/// the whole row, `(2, GridPlacement::span(3))` three tracks from line 2.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct GridPlacement {
    start: GridLine,
    end: GridLine,
}

impl GridPlacement {
    /// From line `n` (from 1 at the start, from -1 at the end; 0 is auto),
    /// CSS `grid-column: n`. A number converts to it.
    pub fn line(n: i32) -> GridPlacement {
        let start = GridLine(match n {
            0 => w::GridPos::Auto,
            n => w::GridPos::Line(n.clamp(i16::MIN as i32, i16::MAX as i32) as i16),
        });
        GridPlacement {
            start,
            end: GridLine::default(),
        }
    }

    /// Spanning `n` tracks (CSS `span n`).
    pub fn span(n: u16) -> GridPlacement {
        GridPlacement {
            start: GridLine(w::GridPos::Span(n.max(1))),
            end: GridLine::default(),
        }
    }
}

impl From<i32> for GridPlacement {
    fn from(start: i32) -> Self {
        GridPlacement::line(start)
    }
}

/// `(start, end)`, each a line number or a span (the start of each
/// placement).
impl<A: Into<GridPlacement>, B: Into<GridPlacement>> From<(A, B)> for GridPlacement {
    fn from((start, end): (A, B)) -> Self {
        GridPlacement {
            start: start.into().start,
            end: end.into().start,
        }
    }
}

/// CSS `grid-auto-flow`: where auto-placed items go.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GridAutoFlow {
    /// Filling each row (the default).
    Row,
    /// Filling each column.
    Column,
    /// Filling rows, back-filling earlier holes with later items.
    RowDense,
    /// Filling columns, back-filling holes.
    ColumnDense,
}

/// The grid props a style sets (boxed in `Style`: most styles set none).
#[derive(Clone, Debug, PartialEq, Default)]
pub(crate) struct GridStyle {
    // xtask-allow(state-fields): passive record - columns
    pub(crate) columns: Option<Tracks>,
    // xtask-allow(state-fields): passive record - rows
    pub(crate) rows: Option<Tracks>,
    // xtask-allow(state-fields): passive record - auto columns
    pub(crate) auto_columns: Option<Tracks>,
    // xtask-allow(state-fields): passive record - auto rows
    pub(crate) auto_rows: Option<Tracks>,
    // xtask-allow(state-fields): passive record - flow
    pub(crate) flow: Option<GridAutoFlow>,
    // xtask-allow(state-fields): passive record - column
    pub(crate) column: Option<GridPlacement>,
    // xtask-allow(state-fields): passive record - row
    pub(crate) row: Option<GridPlacement>,
    // xtask-allow(state-fields): passive record - justify items
    pub(crate) justify_items: Option<Align>,
    // xtask-allow(state-fields): passive record - justify self
    pub(crate) justify_self: Option<Align>,
}

impl GridStyle {
    /// Overlay `o` on `self` (fields set in `o` win).
    pub(crate) fn merge(&mut self, o: &GridStyle) {
        macro_rules! take {
            ($($f:ident),*) => {$(
                if o.$f.is_some() {
                    self.$f = o.$f.clone();
                }
            )*};
        }
        take!(
            columns,
            rows,
            auto_columns,
            auto_rows,
            flow,
            column,
            row,
            justify_items,
            justify_self
        );
    }
}

/// A list of track sizes (repeats left out).
fn sizes(tracks: &[Track]) -> Vec<w::TrackSize> {
    tracks
        .iter()
        .filter_map(|t| match t.0 {
            TrackKind::Size { min, max } => Some(w::TrackSize {
                min: breadth(min),
                max: breadth(max),
            }),
            TrackKind::Repeat(..) => None,
        })
        .collect()
}

fn breadth(b: Breadth) -> w::Breadth {
    match b {
        Breadth::Length(l) => w::Breadth::Size(l.to_wire()),
        Breadth::Fr(f) => w::Breadth::Fr(f),
    }
}

fn template(tracks: &Tracks) -> Vec<w::GridTrack> {
    tracks
        .0
        .iter()
        .map(|t| match &t.0 {
            TrackKind::Size { min, max } => w::GridTrack::Single(w::TrackSize {
                min: breadth(*min),
                max: breadth(*max),
            }),
            TrackKind::Repeat(count, inner) => w::GridTrack::Repeat(
                match count {
                    Repeat::Times(n) => w::GridRepeat::Count(*n),
                    Repeat::AutoFill => w::GridRepeat::AutoFill,
                    Repeat::AutoFit => w::GridRepeat::AutoFit,
                },
                sizes(inner),
            ),
        })
        .collect()
}

fn placement(p: Option<GridPlacement>) -> w::GridLine {
    let p = p.unwrap_or_default();
    w::GridLine {
        start: p.start.0,
        end: p.end.0,
    }
}

/// The engine's grid props of a style: `container` says whether the box
/// is a grid (`None`: neither a grid nor placed in one).
pub(crate) fn resolve(g: Option<&GridStyle>, container: bool) -> Option<Box<w::GridProps>> {
    if g.is_none() && !container {
        return None;
    }
    let g = g.cloned().unwrap_or_default();
    Some(Box::new(w::GridProps {
        container: container.then(|| w::GridContainer {
            columns: g.columns.as_ref().map(template).unwrap_or_default(),
            rows: g.rows.as_ref().map(template).unwrap_or_default(),
            auto_columns: g
                .auto_columns
                .as_ref()
                .map(|t| sizes(&t.0))
                .unwrap_or_default(),
            auto_rows: g
                .auto_rows
                .as_ref()
                .map(|t| sizes(&t.0))
                .unwrap_or_default(),
            flow: match g.flow.unwrap_or(GridAutoFlow::Row) {
                GridAutoFlow::Row => w::GridFlow::Row,
                GridAutoFlow::Column => w::GridFlow::Column,
                GridAutoFlow::RowDense => w::GridFlow::RowDense,
                GridAutoFlow::ColumnDense => w::GridFlow::ColumnDense,
            },
            justify_items: g.justify_items,
        }),
        column: placement(g.column),
        row: placement(g.row),
        justify_self: g.justify_self,
    }))
}
