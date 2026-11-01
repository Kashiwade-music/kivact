//! Role: Logical-pixel geometry.
//! Half-open rectangles keep adjacent hit regions disjoint.

/// Stable host-assigned accessibility node identity; usable on any thread.
/// Arvane treats it as an opaque key with no knowledge of the host's tree.
/// Built with [`NodeId::new`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub struct NodeId(pub u64);

impl NodeId {
    /// The node with this raw id; read it back as `.0`.
    pub const fn new(raw: u64) -> Self {
        NodeId(raw)
    }
}

/// Position in logical pixels; usable on any thread.
/// Built with [`Point::new`].
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct Point {
    /// Horizontal position.
    pub x: f32,
    /// Vertical position.
    pub y: f32,
}

/// Extent in logical pixels; usable on any thread.
/// Built with [`Size::new`].
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct Size {
    /// Horizontal extent.
    pub width: f32,
    /// Vertical extent.
    pub height: f32,
}

/// Half-open axis-aligned bounds in logical pixels; usable on any thread.
/// Operations using a rectangle specify its coordinate origin; image
/// source rectangles explicitly interpret its units as source pixels.
/// Built with [`Rect::new`].
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct Rect {
    /// Left edge.
    pub x: f32,
    /// Top edge.
    pub y: f32,
    /// Horizontal extent.
    pub width: f32,
    /// Vertical extent.
    pub height: f32,
}

impl Rect {
    /// Whether the point is inside these half-open bounds.
    pub fn contains(&self, p: Point) -> bool {
        p.x >= self.x && p.y >= self.y && p.x < self.x + self.width && p.y < self.y + self.height
    }

    /// The right edge.
    pub fn right(&self) -> f32 {
        self.x + self.width
    }

    /// The bottom edge.
    pub fn bottom(&self) -> f32 {
        self.y + self.height
    }

    /// The bounds grown by `dx` on the left and right and `dy` on the top
    /// and bottom (shrunk for negative values, never below zero size).
    pub fn inflate(&self, dx: f32, dy: f32) -> Rect {
        let width = (self.width + 2.0 * dx).max(0.0);
        let height = (self.height + 2.0 * dy).max(0.0);
        Rect {
            x: self.x + (self.width - width) / 2.0,
            y: self.y + (self.height - height) / 2.0,
            width,
            height,
        }
    }

    /// The smallest bounds holding both.
    pub fn union(&self, other: Rect) -> Rect {
        let (x0, y0) = (self.x.min(other.x), self.y.min(other.y));
        let (x1, y1) = (
            self.right().max(other.right()),
            self.bottom().max(other.bottom()),
        );
        Rect {
            x: x0,
            y: y0,
            width: x1 - x0,
            height: y1 - y0,
        }
    }

    /// Whether the two bounds share an area or an edge.
    pub fn intersects(&self, other: Rect) -> bool {
        self.x <= other.right()
            && other.x <= self.right()
            && self.y <= other.bottom()
            && other.y <= self.bottom()
    }
}

impl Point {
    /// The point at `x` and `y`.
    pub const fn new(x: f32, y: f32) -> Self {
        Point { x, y }
    }
}

impl Size {
    /// The size `width` by `height`.
    pub const fn new(width: f32, height: f32) -> Self {
        Size { width, height }
    }

    /// The rect at the origin with this size.
    pub fn to_rect(self) -> Rect {
        Rect::new(0.0, 0.0, self.width, self.height)
    }
}

impl Rect {
    /// The rectangle whose top-left corner is `x`, `y`, `width` wide and
    /// `height` tall.
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Rect {
            x,
            y,
            width,
            height,
        }
    }

    /// The rectangle between two corners, regardless of their order.
    /// Equal points produce a rectangle with zero width and height.
    pub fn from_points(a: Point, b: Point) -> Rect {
        Rect::new(
            a.x.min(b.x),
            a.y.min(b.y),
            (a.x - b.x).abs(),
            (a.y - b.y).abs(),
        )
    }

    pub fn center(&self) -> Point {
        Point::new(self.x + self.width / 2.0, self.y + self.height / 2.0)
    }
}

impl Rect {
    pub fn intersect(&self, other: &Rect) -> Rect {
        let x0 = self.x.max(other.x);
        let y0 = self.y.max(other.y);
        let x1 = (self.x + self.width).min(other.x + other.width);
        let y1 = (self.y + self.height).min(other.y + other.height);
        Rect {
            x: x0,
            y: y0,
            width: (x1 - x0).max(0.0),
            height: (y1 - y0).max(0.0),
        }
    }
}

/// Size hints for boxes whose size does not come from children.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct IntrinsicSize {
    pub min: Option<Size>,
    pub preferred: Option<Size>,
    pub max: Option<Size>,
}

/// Identity of a hit/semantics region inside one node's scene. Unique within
/// that node only.
/// Built with [`RegionId::new`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub struct RegionId(pub u64);

impl RegionId {
    /// The region with this raw id; read it back as `.0`.
    pub const fn new(raw: u64) -> Self {
        RegionId(raw)
    }
}
