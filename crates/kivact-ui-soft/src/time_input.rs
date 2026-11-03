//! Role: Typed wall-clock time, independent of time zone.
use kivact::control::{Node, NodeBuilder, icon, node, row};
use kivact::i18n::message;
use kivact::style::{Align, style};
use kivact::ui::text_input;
use kivact::view::{IntoText, IntoView, View};
use kivact::{Binding, Value, memo, signal};

/// A time on a 24-hour clock with second precision.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct TimeOfDay {
    hour: u8,
    minute: u8,
    second: u8,
}

impl TimeOfDay {
    pub fn new(hour: u8, minute: u8, second: u8) -> Option<Self> {
        (hour < 24 && minute < 60 && second < 60).then_some(Self {
            hour,
            minute,
            second,
        })
    }

    pub fn hour(self) -> u8 {
        self.hour
    }
    pub fn minute(self) -> u8 {
        self.minute
    }
    pub fn second(self) -> u8 {
        self.second
    }

    pub fn adjust_minutes(self, delta: i32) -> Self {
        let minutes = (i32::from(self.hour) * 60 + i32::from(self.minute) + delta.rem_euclid(1440))
            .rem_euclid(1440);
        Self {
            hour: (minutes / 60) as u8,
            minute: (minutes % 60) as u8,
            second: self.second,
        }
    }
}

fn parse_time(input: &str, seconds: bool) -> Option<TimeOfDay> {
    let lower = input.trim().to_ascii_lowercase();
    let (input, meridiem) = if let Some(time) = lower
        .strip_suffix("am")
        .or_else(|| lower.strip_suffix("a.m."))
    {
        (time.trim_end(), Some(false))
    } else if let Some(time) = lower
        .strip_suffix("pm")
        .or_else(|| lower.strip_suffix("p.m."))
    {
        (time.trim_end(), Some(true))
    } else {
        (lower.as_str(), None)
    };
    if input.is_empty() || !input.bytes().all(|b| b.is_ascii_digit() || b == b':') {
        return None;
    }
    let fields = input.split(':').collect::<Vec<_>>();
    let (hour, minute, second) = match fields.as_slice() {
        [hour] if hour.len() <= 2 => (*hour, "0", "0"),
        [compact] if compact.len() == 3 || compact.len() == 4 => {
            let (hour, minute) = compact.split_at(compact.len() - 2);
            (hour, minute, "0")
        }
        [hour, minute] => (*hour, *minute, "0"),
        [hour, minute, second] if seconds => (*hour, *minute, *second),
        _ => return None,
    };
    let mut hour = hour.parse::<u8>().ok()?;
    if let Some(pm) = meridiem {
        if !(1..=12).contains(&hour) {
            return None;
        }
        hour = hour % 12 + if pm { 12 } else { 0 };
    }
    TimeOfDay::new(hour, minute.parse().ok()?, second.parse().ok()?)
}

pub struct TimeInput {
    root: Node,
    label: Value<String>,
    value: Binding<Option<TimeOfDay>>,
    seconds: bool,
}

pub fn time_input(label: impl IntoText, value: impl Into<Binding<Option<TimeOfDay>>>) -> TimeInput {
    TimeInput {
        root: node(),
        label: label.into_text_value(),
        value: value.into(),
        seconds: false,
    }
}

impl TimeInput {
    pub fn show_seconds(mut self, show: bool) -> Self {
        self.seconds = show;
        self
    }
}

impl NodeBuilder for TimeInput {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for TimeInput {
    fn into_view(self) -> View {
        let Self {
            root,
            label,
            value,
            seconds,
        } = self;
        let invalid = signal(false);
        let formatted = memo(move || {
            value
                .get()
                .map(|time| {
                    if seconds {
                        format!("{:02}:{:02}:{:02}", time.hour, time.minute, time.second)
                    } else {
                        format!("{:02}:{:02}", time.hour, time.minute)
                    }
                })
                .unwrap_or_default()
        });
        let draft = signal(formatted.get_untracked());
        kivact::effect(move || {
            draft.set(formatted.get());
            invalid.set(false);
        });
        let input = text_input(label, draft)
            .placeholder(if seconds { "HH:MM:SS" } else { "HH:MM" })
            .invalid(move || {
                invalid.get()
                    && !draft.get().trim().is_empty()
                    && parse_time(&draft.get(), seconds).is_none()
            })
            .draft(move |text| {
                if text.trim().is_empty() {
                    value.set(None);
                    invalid.set(false);
                    draft.set(String::new());
                } else if let Some(time) = parse_time(&text, seconds) {
                    value.set(Some(time));
                    invalid.set(false);
                    draft.set(formatted.get_untracked());
                } else {
                    invalid.set(true);
                }
            });
        root.child(
            row((icon("clock").name(message("soft.time.icon", "Time")), input))
                .style(style().min_width(120).align_items(Align::Center).gap(4)),
        )
        .into_view()
    }
}
