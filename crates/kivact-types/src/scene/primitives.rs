//! Role: Pure fill, border and stroke primitives shared with renderers.

/// Which points a self-intersecting or nested outline encloses.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FillRule {
    /// Points around which the outline winds a non-zero number of times.
    #[default]
    NonZero,
    /// Points the outline crosses around an odd number of times.
    EvenOdd,
}

/// How a line ends where a contour is open (and how a zero-length contour
/// is drawn: a round or square dot, or nothing for butt).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LineCap {
    /// Flat at the end point.
    #[default]
    Butt,
    /// A half circle beyond the end point.
    Round,
    /// A half square beyond the end point.
    Square,
}

/// How two segments of a line meet.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LineJoin {
    /// Extended edges meeting at a point, beveled beyond the miter limit.
    #[default]
    Miter,
    /// An arc around the joint.
    Round,
    /// A straight cut across the joint.
    Bevel,
}

/// How a shape's outline is stroked (SVG and canvas stroke properties).
///
/// ```
/// use kivact_types::{LineCap, Stroke};
/// // A dotted line: zero-length dashes with round caps make dots.
/// let dotted = Stroke {
///     cap: LineCap::Round,
///     dash: vec![0.0, 4.0],
///     ..Stroke::new(2.0)
/// };
/// assert_eq!(dotted.width, 2.0);
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Stroke {
    /// Line width.
    pub width: f64,
    /// Ends of open contours and dashes.
    pub cap: LineCap,
    /// Joints.
    pub join: LineJoin,
    /// The longest miter, in line widths, before a miter join is beveled
    /// (at least 1).
    pub miter_limit: f64,
    /// Dash and gap lengths, alternating, repeated along each contour;
    /// empty for a solid line. An odd count repeats as if doubled, as in
    /// SVG.
    pub dash: Vec<f64>,
    /// How far into the dash pattern each contour starts.
    pub dash_offset: f64,
}

impl Stroke {
    /// A solid line of `width` with butt caps and miter joins (limit 10,
    /// as canvas).
    pub fn new(width: impl Into<f64>) -> Self {
        Stroke {
            width: width.into(),
            cap: LineCap::Butt,
            join: LineJoin::Miter,
            miter_limit: 10.0,
            dash: Vec::new(),
            dash_offset: 0.0,
        }
    }
    /// Set the ends of open contours and dashes.
    pub fn cap(mut self, cap: LineCap) -> Self {
        self.cap = cap;
        self
    }
    /// Set how adjacent segments meet.
    pub fn join(mut self, join: LineJoin) -> Self {
        self.join = join;
        self
    }
    /// Set the miter limit in line widths.
    pub fn miter_limit(mut self, limit: impl Into<f64>) -> Self {
        self.miter_limit = limit.into();
        self
    }
    /// Set alternating dash and gap lengths in logical pixels.
    pub fn dash(mut self, lengths: impl IntoIterator<Item = f64>) -> Self {
        self.dash = lengths.into_iter().collect();
        self
    }
    /// Set the starting offset within the dash pattern.
    pub fn dash_offset(mut self, offset: impl Into<f64>) -> Self {
        self.dash_offset = offset.into();
        self
    }
}

impl From<i32> for Stroke {
    fn from(width: i32) -> Self {
        Self::new(width)
    }
}

impl From<f64> for Stroke {
    fn from(width: f64) -> Self {
        Self::new(width)
    }
}

/// How a border's line is drawn.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Hash)]
pub enum BorderStyle {
    /// One continuous line of the border's width.
    #[default]
    Solid,
    /// Dashes three border widths long, with gaps of the same length.
    Dashed,
    /// Round dots one border width across, spaced one width apart.
    Dotted,
    /// Two parallel solid lines, each a third of the width, with a gap of a
    /// third between them. A width below 3 draws as `Solid`.
    Double,
    /// A solid groove: on the top and left sides the outer half is dark and
    /// the inner half light; on the bottom and right sides the reverse.
    Groove,
    /// A solid ridge: on the top and left sides the outer half is light and
    /// the inner half dark; on the bottom and right sides the reverse.
    Ridge,
    /// A solid inset: the top and left sides are dark, the bottom and right
    /// sides light.
    Inset,
    /// A solid outset: the top and left sides are light, the bottom and right
    /// sides dark.
    Outset,
}
