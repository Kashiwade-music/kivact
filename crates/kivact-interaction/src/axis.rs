//! Role: The two directions a behavior moves along, read from a public Orientation.

use kivact_core::ErrorKind;
use kivact_core::custom::raise;
use kivact_core::view::Orientation;

/// A direction a behavior moves along: `Orientation` without `Unspecified`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Axis {
    Horizontal,
    Vertical,
}

impl Axis {
    /// The axis of a public `orientation`; `api` names the public call in the
    /// message. `Orientation::Unspecified` is a usage error.
    #[track_caller]
    pub(crate) fn of(orientation: Orientation, api: &'static str) -> Self {
        match orientation {
            Orientation::Horizontal => Self::Horizontal,
            Orientation::Vertical => Self::Vertical,
            Orientation::Unspecified => unspecified(api),
        }
    }

    /// The public orientation of this axis.
    pub(crate) fn orientation(self) -> Orientation {
        match self {
            Self::Horizontal => Orientation::Horizontal,
            Self::Vertical => Orientation::Vertical,
        }
    }

    pub(crate) fn is_horizontal(self) -> bool {
        self == Self::Horizontal
    }
}

/// A list policy's axis: `None` (all four arrows) or one axis;
/// `Some(Orientation::Unspecified)` is a usage error.
#[track_caller]
pub(crate) fn check_list_axis(axis: Option<Orientation>, api: &'static str) {
    if axis == Some(Orientation::Unspecified) {
        raise(
            ErrorKind::Message,
            format!(
                "{api}: ListOptions::axis is Some(Orientation::Unspecified); use None for all four arrow keys"
            ),
        );
    }
}

#[track_caller]
fn unspecified(api: &'static str) -> ! {
    raise(
        ErrorKind::Message,
        format!(
            "{api}: Orientation::Unspecified is not a direction; pass Orientation::Horizontal or Orientation::Vertical"
        ),
    )
}
