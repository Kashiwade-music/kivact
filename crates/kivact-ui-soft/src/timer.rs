//! Role: Display the elapsed time since a wall-clock instant.
use crate::{TextColor, TextKind, TextSize, TextWeight, styled_text};

use kivact::Prop;
use kivact::component;
use kivact::state::Machine;
use kivact::state::Step;
use kivact::state::Value;
use kivact::state::effect_on;
use kivact::state::machine;
use kivact::state::memo;
use kivact::view::Element;
use kivact::view::LiveRegion;
use kivact::view::node;
use std::time::{Duration, SystemTime};

/// How elapsed time is displayed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Prop)]
pub enum TimerFormat {
    #[default]
    /// Show elapsed units such as seconds, minutes and hours.
    Elapsed,
    /// Show elapsed time as minutes:seconds or hours:minutes:seconds.
    Clock,
}

/// A live elapsed-time display that counts from `start` (now when omitted).
///
/// The text is wrapped in a node that carries the caller's modifiers, because the text takes
/// none. `kind`, `color`, `size` and `weight` set its typography, and `live` makes the changing
/// text a live region.
#[component]
pub fn timer(
    start: Option<Value<SystemTime>>,
    #[prop(default = TimerFormat::Elapsed)] format: Value<TimerFormat>,
    #[prop(default = TextKind::Supporting)] kind: Value<TextKind>,
    #[prop(default = TextColor::Secondary)] color: Value<TextColor>,
    size: Option<Value<TextSize>>,
    weight: Option<Value<TextWeight>>,
    live: Option<Value<LiveRegion>>,
) -> impl Element {
    let clock = machine(
        ElapsedTimer::Unstarted,
        move || ElapsedClock {
            format: format.get(),
            now: SystemTime::now(),
        },
        |_, ()| {},
    );
    let display = memo(move || match clock.get() {
        ElapsedTimer::Unstarted => String::new(),
        ElapsedTimer::Running { start, next } => presentation(start, start + next, format.get()).0,
    });
    effect_on(format, move |_| clock.send(ElapsedEvent::Tick));
    if let Some(start) = start {
        clock.feed(start, ElapsedEvent::Start);
    }
    clock.send(ElapsedEvent::Start(
        start.map_or_else(SystemTime::now, |value| value.get()),
    ));
    let mut text = styled_text(display)
        .kind(kind)
        .color(color)
        .tabular_numbers(true);
    if let Some(size) = size {
        text = text.size(size);
    }
    if let Some(weight) = weight {
        text = text.weight(weight);
    }
    let root = node().child(text);
    match live {
        Some(live) => root.with_node(move |node| node.live_region(live)),
        None => root,
    }
}

fn presentation(start: SystemTime, now: SystemTime, format: TimerFormat) -> (String, Duration) {
    let (elapsed, until_start) = match now.duration_since(start) {
        Ok(elapsed) => (elapsed, None),
        Err(future) => (Duration::ZERO, Some(future.duration())),
    };
    let seconds = elapsed.as_secs();
    let hours = seconds / 3600;
    let minutes = seconds / 60 % 60;
    let remainder = seconds % 60;
    let text = match format {
        TimerFormat::Clock if hours > 0 => format!("{hours}:{minutes:02}:{remainder:02}"),
        TimerFormat::Clock => format!("{minutes}:{remainder:02}"),
        TimerFormat::Elapsed if hours > 0 => format!("{hours}h {minutes:02}m"),
        TimerFormat::Elapsed if minutes > 0 => format!("{minutes}m {remainder:02}s"),
        TimerFormat::Elapsed => format!("{remainder}s"),
    };
    let precision = if format == TimerFormat::Elapsed && hours > 0 {
        60_000
    } else {
        1_000
    };
    let delay = until_start.map_or_else(
        || Duration::from_millis((precision - elapsed.as_millis() % precision) as u64),
        |until| {
            until
                .saturating_add(Duration::from_secs(1))
                .min(Duration::from_millis(i32::MAX as u64))
        },
    );
    (text, delay)
}

#[derive(Clone, PartialEq)]
enum ElapsedTimer {
    /// Built but not mounted; `into_view` sends the first `Start`.
    Unstarted,
    Running {
        start: SystemTime,
        next: Duration,
    },
}
enum ElapsedEvent {
    Start(SystemTime),
    Tick,
}
struct ElapsedClock {
    format: TimerFormat,
    now: SystemTime,
}
impl Machine for ElapsedClock {
    type State = ElapsedTimer;
    type Event = ElapsedEvent;
    type Effect = ();
    fn transition(
        &self,
        state: &ElapsedTimer,
        event: ElapsedEvent,
    ) -> Step<ElapsedTimer, ElapsedEvent, ()> {
        let start = match (state, event) {
            (_, ElapsedEvent::Start(start)) => start,
            (ElapsedTimer::Running { start, .. }, ElapsedEvent::Tick) => *start,
            (ElapsedTimer::Unstarted, ElapsedEvent::Tick) => return Step::stay(),
        };
        let next = self.now.duration_since(start).unwrap_or_default();
        let delay = presentation(start, self.now, self.format).1;
        Step::to(ElapsedTimer::Running { start, next }).after(delay, ElapsedEvent::Tick)
    }
}
