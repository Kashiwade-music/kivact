//! Role: Shared control values and edit snapshots.
use kivact::geometry::Point;
use kivact::style::Icon;
/// A directional command, in clockwise degrees from the positive horizontal axis.
#[derive(Clone, Debug, PartialEq)]
pub struct PieEntry {
    pub label: String,
    pub angle: f64,
    /// The icon of the command: a name in the theme's icon set or an image.
    pub icon: Icon,
}
impl PieEntry {
    pub fn new(label: impl Into<String>, angle: f64, icon: impl Into<Icon>) -> Self {
        Self {
            label: label.into(),
            angle,
            icon: icon.into(),
        }
    }
}
/// Public state permits a surrounding application to open the same menu from a hovered editor.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum PieState {
    #[default]
    Closed,
    Open {
        mode: PieMode,
        center: Point,
        hot: Option<usize>,
    },
}
/// How the directional menu was opened.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PieMode {
    Click,
    Held,
}
impl PieState {
    pub fn open_at(center: Point, held: bool) -> Self {
        Self::Open {
            mode: if held { PieMode::Held } else { PieMode::Click },
            center,
            hot: None,
        }
    }
    pub(crate) fn hot(self) -> Option<usize> {
        match self {
            Self::Closed => None,
            Self::Open { hot, .. } => hot,
        }
    }
    pub(crate) fn center(self) -> Point {
        match self {
            Self::Closed => Point::default(),
            Self::Open { center, .. } => center,
        }
    }
    /// Keep the center dead zone and select the closest offered direction.
    pub fn pointer(&mut self, pointer: Point, entries: &[PieEntry]) {
        let Self::Open { center, hot, .. } = self else {
            return;
        };
        let delta = Point::new(pointer.x - center.x, pointer.y - center.y);
        *hot = if delta.x.hypot(delta.y) < 22.0 {
            None
        } else {
            let angle = delta.y.atan2(delta.x).to_degrees();
            entries
                .iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| {
                    let distance = |entry: &PieEntry| {
                        ((angle - entry.angle + 180.0).rem_euclid(360.0) - 180.0).abs()
                    };
                    distance(a).total_cmp(&distance(b))
                })
                .map(|(index, _)| index)
        };
    }
}
