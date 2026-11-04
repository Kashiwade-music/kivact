//! Role: Expose reusable live regions for imperative announcements.
use kivact_core::custom::Generation;
use kivact_core::state::{Signal, signal};
use kivact_core::style::{Position, style};
use kivact_core::view::{IntoView, LiveRegion, Role, View, dynamic};
use kivact_core::view::{Node, node, text};

#[derive(Clone, Default, PartialEq, Eq)]
struct NoticeId(Generation);

#[derive(Clone, Default, PartialEq, Eq)]
struct Notice {
    id: NoticeId,
    message: String,
}

/// A handle to the polite and assertive regions created by `outlet()`.
#[derive(Clone, Copy)]
pub struct Announcer {
    polite: Signal<Notice>,
    assertive: Signal<Notice>,
}

/// Create a scoped announcement handle during component setup.
pub fn announcer() -> Announcer {
    Announcer {
        polite: signal(Notice::default()),
        assertive: signal(Notice::default()),
    }
}

impl Announcer {
    /// Publish a message to one live region. Calls to the same region within
    /// one UI turn use the latest message. `Off` leaves both regions alone.
    pub fn announce(self, message: impl Into<String>, level: LiveRegion) {
        let notice = match level {
            LiveRegion::Polite => self.polite,
            LiveRegion::Assertive => self.assertive,
            LiveRegion::Off => return,
        };
        notice.update(move |notice| {
            notice.id.0.next();
            notice.message = message.into();
        });
    }

    /// Mount one outlet for this handle. The live regions are accessible
    /// and stay outside normal layout.
    pub fn outlet(self) -> View {
        (
            region(self.polite, LiveRegion::Polite),
            region(self.assertive, LiveRegion::Assertive),
        )
            .into_view()
    }
}

fn region(notice: Signal<Notice>, level: LiveRegion) -> Node {
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
