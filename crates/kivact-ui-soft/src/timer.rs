//! Role: Display the elapsed time since a wall-clock instant.
use crate::{Text, TextColor, TextKind, TextSize, TextWeight, text};
use kivact::view::Node;
use kivact::view::NodeBuilder;

use kivact::state::IntoValue;
use kivact::state::Machine;
use kivact::state::MachineHandle;
use kivact::state::Step;
use kivact::state::Value;
use kivact::state::machine;
use kivact::state::memo;
use kivact::view::IntoView;
use kivact::view::LiveRegion;
use kivact::view::View;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, SystemTime};

/// How elapsed time is displayed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TimerFormat {
    #[default]
    Elapsed,
    Clock,
}

/// A live elapsed-time display.
pub struct Timer {
    text: Text,
    clock: MachineHandle<ElapsedClock>,
    settings: Rc<RefCell<TimerSettings>>,
}

/// Start a timer when mounted.
pub fn timer() -> Timer {
    let settings = Rc::new(RefCell::new(TimerSettings {
        start: None,
        format: TimerFormat::default().into_value(),
    }));
    let input = settings.clone();
    let clock = machine(
        ElapsedTimer::Running {
            start: SystemTime::now(),
            next: Duration::ZERO,
        },
        move || ElapsedClock {
            format: input.borrow().format.get(),
            now: SystemTime::now(),
        },
        |_, ()| {},
    );
    let format = settings.clone();
    let display = memo(move || {
        let ElapsedTimer::Running { start, next } = clock.get();
        presentation(start, start + next, format.borrow().format.get()).0
    });
    Timer {
        text: text(display)
            .kind(TextKind::Supporting)
            .color(TextColor::Secondary)
            .tabular_numbers(true),
        clock,
        settings,
    }
}

struct TimerSettings {
    start: Option<Value<SystemTime>>,
    format: Value<TimerFormat>,
}

impl Timer {
    /// Measure elapsed time from a caller-supplied wall-clock instant.
    pub fn start(self, start: impl IntoValue<SystemTime>) -> Self {
        self.settings.borrow_mut().start = Some(start.into_value());
        self
    }
    /// Set the elapsed time format.
    pub fn format(self, format: impl IntoValue<TimerFormat>) -> Self {
        self.settings.borrow_mut().format = format.into_value();
        self
    }
    /// Set the typography role.
    pub fn kind(mut self, kind: impl IntoValue<TextKind>) -> Self {
        self.text = self.text.kind(kind);
        self
    }
    /// Override the text size.
    pub fn size(mut self, size: impl IntoValue<TextSize>) -> Self {
        self.text = self.text.size(size);
        self
    }
    /// Override the text color.
    pub fn color(mut self, color: impl IntoValue<TextColor>) -> Self {
        self.text = self.text.color(color);
        self
    }
    /// Override the text weight.
    pub fn weight(mut self, weight: impl IntoValue<TextWeight>) -> Self {
        self.text = self.text.weight(weight);
        self
    }
    /// Optionally announce the changing text.
    pub fn live(self, live: impl IntoValue<LiveRegion>) -> Self {
        self.with_node(move |node| node.live_region(live))
    }
}

impl NodeBuilder for Timer {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        self.text.node_mut()
    }
}

impl IntoView for Timer {
    fn into_view(self) -> View {
        let Self {
            text,
            clock,
            settings,
        } = self;
        let format = settings.borrow().format;
        kivact::state::effect_on(format, move |_| clock.send(ElapsedEvent::Tick));
        let start = settings.borrow().start;
        if let Some(start) = start {
            clock.feed(start, ElapsedEvent::Start);
        }
        clock.send(ElapsedEvent::Start(
            start.map_or_else(SystemTime::now, |value| value.get()),
        ));
        text.into_view()
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
    Running { start: SystemTime, next: Duration },
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
        let ElapsedTimer::Running { start, .. } = *state;
        let start = match event {
            ElapsedEvent::Start(start) => start,
            ElapsedEvent::Tick => start,
        };
        let next = self.now.duration_since(start).unwrap_or_default();
        let delay = presentation(start, self.now, self.format).1;
        Step::to(ElapsedTimer::Running { start, next }).after(delay, ElapsedEvent::Tick)
    }
}

kivact::state::constant_value!(TimerFormat);
