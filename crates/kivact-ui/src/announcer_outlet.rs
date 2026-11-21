//! Role: Mount the two live regions of an `Announcer` outside the layout.
pub(crate) mod look;

use crate::announcer_outlet::look::AnnouncerOutletTheme;
use crate::layer::with_base_recipe_style;
use kivact_core::state::Value;
use kivact_core::style::{Position, style};
use kivact_core::view::{LiveRegion, Node, Role, Variants, dynamic, node, text};
use kivact_interaction::{Announcement, Announcer};
use kivact_macros::component;

/// The polite and the assertive live region of `announcer`, outside the
/// layout. Mount it once.
#[component]
pub fn announcer_outlet(announcer: Announcer) -> impl Element {
    let root = with_base_recipe_style(
        node().hit_testable(false),
        |r: &AnnouncerOutletTheme| r.root_recipe(),
        &Variants::default(),
        || (),
    );
    root.child(region(announcer.polite(), LiveRegion::Polite))
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
