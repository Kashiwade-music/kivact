//! Role: The thumb geometry the default switch look draws, by size.
use crate::control_size::ControlSize;
use kivact_interaction::SwitchTrack;

pub(super) fn switch_track(size: ControlSize) -> SwitchTrack {
    if matches!(size, ControlSize::ExtraSmall | ControlSize::Small) {
        SwitchTrack {
            width: 32.0,
            height: 20.0,
            off_side: 14.0,
            on_side: 16.0,
            off_inset: 3.0,
        }
    } else {
        SwitchTrack {
            width: 40.0,
            height: 24.0,
            off_side: 16.0,
            on_side: 20.0,
            off_inset: 4.0,
        }
    }
}
