//! Role: Expose reusable live regions for imperative announcements.
use kivact_core::custom::Generation;
use kivact_core::state::{IntoValue, Signal, Value, signal};
use kivact_core::view::LiveRegion;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct NoticeId(Generation);

/// One message of a live region; a repeated message is a new value.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Announcement {
    id: NoticeId,
    pub message: String,
}

/// A handle to the polite and assertive regions, mounted by
/// `kivact::ui::announcer_outlet`.
#[derive(Clone, Copy)]
pub struct Announcer {
    polite: Signal<Announcement>,
    assertive: Signal<Announcement>,
}

/// Create a scoped announcement handle during component setup.
pub fn use_announcer() -> Announcer {
    Announcer {
        polite: signal(Announcement::default()),
        assertive: signal(Announcement::default()),
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

    /// The latest message of the polite region.
    pub fn polite(self) -> Value<Announcement> {
        self.polite.into_value()
    }

    /// The latest message of the assertive region.
    pub fn assertive(self) -> Value<Announcement> {
        self.assertive.into_value()
    }
}
