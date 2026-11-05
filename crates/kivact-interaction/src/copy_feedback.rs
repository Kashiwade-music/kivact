//! Role: Copy text and expose a timed confirmation after successful clipboard writes.
use std::time::Duration;

use kivact_core::state::{Machine as MachineTrait, MachineHandle, Step, Value, machine, memo};
use kivact_core::task::spawn;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Copied {
    Idle,
    Copied,
}
enum Event {
    Copied,
    Expired,
}
enum Effect {}
struct Machine {
    duration: Duration,
}
impl MachineTrait for Machine {
    type State = Copied;
    type Event = Event;
    type Effect = Effect;

    fn transition(&self, _: &Copied, event: Event) -> Step<Copied, Event, Effect> {
        match event {
            Event::Copied => Step::to(Copied::Copied).after(self.duration, Event::Expired),
            Event::Expired => Step::to(Copied::Idle),
        }
    }
}

/// An owner-scoped clipboard operation and its temporary success confirmation.
#[derive(Clone, Copy)]
pub struct CopyFeedbackApi {
    state: MachineHandle<Machine>,
    copied: Value<bool>,
}

/// Create clipboard feedback lasting `duration` after each successful write.
///
/// Repeated successful copies restart the duration. Disposing the owner cancels
/// pending clipboard tasks and the confirmation timer. Setup only.
#[track_caller]
pub fn use_copy_feedback(duration: Duration) -> CopyFeedbackApi {
    let state = machine(
        Copied::Idle,
        move || Machine { duration },
        |_, effect| match effect {},
    );
    CopyFeedbackApi {
        state,
        copied: memo(move || state.get() == Copied::Copied),
    }
}

impl CopyFeedbackApi {
    /// Copy text to the system clipboard and show feedback on success.
    ///
    /// Clipboard errors reach the owner's error handler and do not start a
    /// confirmation. The operation belongs to the current owner.
    #[track_caller]
    pub fn copy(self, text: impl Into<String>) {
        let write = kivact_core::platform::copy_text(text);
        spawn(async move {
            write.await?;
            self.state.send(Event::Copied);
            Ok(())
        });
    }

    /// Whether the most recent successful copy is still being confirmed.
    pub fn is_copied(self) -> Value<bool> {
        self.copied
    }
}
