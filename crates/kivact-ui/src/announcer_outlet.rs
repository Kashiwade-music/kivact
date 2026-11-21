//! Role: Mount the two live regions of an `Announcer` outside the layout.
use kivact_core::state::Value;
use kivact_core::style::{Position, style};
use kivact_core::view::{LiveRegion, Node, Role, dynamic, node, text};
use kivact_interaction::{Announcement, Announcer};

/// The polite and the assertive live region of `announcer`, outside the
/// layout. Mount it once.
pub fn announcer_outlet(announcer: Announcer) -> Node {
    node()
        .hit_testable(false)
        .style(
            style()
                .position(Position::Absolute)
                .left(0)
                .top(0)
                .size(0, 0),
        )
        .child(region(announcer.polite(), LiveRegion::Polite))
        .child(region(announcer.assertive(), LiveRegion::Assertive))
}

fn region(notice: Value<Announcement>, level: LiveRegion) -> Node {
    node()
        .role(Role::Status)
        .live_region(level)
        .hit_testable(false)
        .style(
            style()
                .position(Position::Absolute)
                .left(-1)
                .top(-1)
                .size(1, 1)
                .clip(true)
                .opacity(0.0),
        )
        .child(dynamic(
            move || notice.get(),
            |notice| (!notice.message.is_empty()).then(|| text(notice.message)),
        ))
}
