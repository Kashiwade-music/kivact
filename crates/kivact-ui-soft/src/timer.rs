//! Role: Display the elapsed time since a wall-clock instant.
use crate::{Text, TextColor, TextKind, TextSize, TextWeight, text};
use kivact::control::{Node, NodeBuilder};
use kivact::primitive::NodeExt;
use kivact::task::{sleep, spawn};
use kivact::view::{IntoValue, IntoView, LiveRegion, View};
use kivact::{Value, effect, signal};
use std::cell::RefCell;
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
    display: kivact::Signal<String>,
    start: Option<Value<SystemTime>>,
    format: TimerFormat,
}

/// Start a timer when mounted.
pub fn timer() -> Timer {
    let display = signal(String::new());
    Timer {
        text: text(display)
            .kind(TextKind::Supporting)
            .color(TextColor::Secondary)
            .tabular_numbers(true),
        display,
        start: None,
        format: TimerFormat::default(),
    }
}

impl Timer {
    /// Measure elapsed time from a caller-supplied wall-clock instant.
    pub fn start(mut self, start: impl IntoValue<SystemTime>) -> Self {
        self.start = Some(start.into_value());
        self
    }
    /// Set the elapsed time format.
    pub fn format(mut self, format: TimerFormat) -> Self {
        self.format = format;
        self
    }
    /// Set the typography role.
    pub fn kind(mut self, kind: TextKind) -> Self {
        self.text = self.text.kind(kind);
        self
    }
    /// Override the text size.
    pub fn size(mut self, size: TextSize) -> Self {
        self.text = self.text.size(size);
        self
    }
    /// Override the text color.
    pub fn color(mut self, color: TextColor) -> Self {
        self.text = self.text.color(color);
        self
    }
    /// Override the text weight.
    pub fn weight(mut self, weight: TextWeight) -> Self {
        self.text = self.text.weight(weight);
        self
    }
    /// Optionally announce the changing text.
    pub fn live(self, live: LiveRegion) -> Self {
        self.live_region(live)
    }
}

impl NodeBuilder for Timer {
    fn node_mut(&mut self) -> &mut Node {
        self.text.node_mut()
    }
}

impl IntoView for Timer {
    fn into_view(self) -> View {
        let Self {
            text,
            display,
            start,
            format,
        } = self;
        let mounted = SystemTime::now();
        let task = RefCell::new(None::<kivact::task::Task>);
        effect(move || {
            let start = start.map_or(mounted, |value| value.get());
            if let Some(previous) = task.borrow_mut().take() {
                previous.cancel();
            }
            let (initial, delay) = presentation(start, SystemTime::now(), format);
            display.set(initial);
            *task.borrow_mut() = Some(spawn::<()>(async move {
                let mut delay = delay;
                loop {
                    sleep(delay).await;
                    let (next, next_delay) = presentation(start, SystemTime::now(), format);
                    if next != display.get_untracked() {
                        display.set(next);
                    }
                    delay = next_delay;
                }
            }));
        });
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
