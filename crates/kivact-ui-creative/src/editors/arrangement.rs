//! Role: Arrangement timeline editor: tracks, clips, automation knots and tools.

pub(crate) mod a11y;
pub(crate) mod edit;
pub(crate) mod gesture;
pub(crate) mod input;
pub(crate) mod layout;
pub(crate) mod model;
pub(crate) mod paint;
mod view;
pub(crate) mod view_state;

pub use crate::editors::arrangement::model::{
    ArrangementRequest, AutomationPoint, TimelineMarker, TimelineTool, TimelineTrack, TrackFlag,
    TrackKind,
};
pub use view::{Arrangement, arrangement};
