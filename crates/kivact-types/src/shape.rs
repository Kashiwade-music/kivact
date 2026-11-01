//! Role: Fill rules, stroke styles and border styles.
//! Plain data usable on any thread.

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
#[non_exhaustive]
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
/// let dotted = Stroke::new(2.0).cap(LineCap::Round).dash([0.0, 4.0]);
/// assert_eq!(dotted.width, 2.0);
/// ```
/// Built with [`Stroke::new`].
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct Stroke {
    /// Line width.
    pub width: f32,
    /// Ends of open contours and dashes.
    pub cap: LineCap,
    /// Joints.
    pub join: LineJoin,
    /// The longest miter, in line widths, before a miter join is beveled
    /// (at least 1).
    pub miter_limit: f32,
    /// Dash and gap lengths, alternating, repeated along each contour;
    /// empty for a solid line. An odd count repeats as if doubled, as in
    /// SVG.
    pub dash: Vec<f32>,
    /// How far into the dash pattern each contour starts.
    pub dash_offset: f32,
}

impl Stroke {
    /// A solid line of `width` with butt caps and miter joins (limit 10,
    /// as canvas).
    pub fn new(width: impl Into<f64>) -> Self {
        Stroke {
            width: width.into() as f32,
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
        self.miter_limit = limit.into() as f32;
        self
    }
    /// Set alternating dash and gap lengths in logical pixels.
    pub fn dash(mut self, lengths: impl IntoIterator<Item = f32>) -> Self {
        self.dash = lengths.into_iter().collect();
        self
    }
    /// Set the starting offset within the dash pattern.
    pub fn dash_offset(mut self, offset: impl Into<f64>) -> Self {
        self.dash_offset = offset.into() as f32;
        self
    }
}

impl From<i32> for Stroke {
    fn from(width: i32) -> Self {
        Self::new(width)
    }
}

impl From<f32> for Stroke {
    fn from(width: f32) -> Self {
        Self::new(width)
    }
}

impl From<f64> for Stroke {
    fn from(width: f64) -> Self {
        Self::new(width)
    }
}

/// How a border's line is drawn.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum BorderStyle {
    #[default]
    Solid,
    Dashed,
    Dotted,
}
