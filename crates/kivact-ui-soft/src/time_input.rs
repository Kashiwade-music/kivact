//! Role: Typed wall-clock time, independent of time zone.
use crate::date_touch::{self, DatePresentation};
use crate::time_of_day::TimeOfDay;
use kivact::i18n::message;
use kivact::state::Binding;
use kivact::state::IntoBinding;
use kivact::state::IntoText;
use kivact::state::IntoValue;
use kivact::state::Value;
use kivact::state::memo;
use kivact::state::signal;
use kivact::style::Align;
use kivact::style::style;
use kivact::ui::text_input;
use kivact::view::IntoView;
use kivact::view::Node;
use kivact::view::NodeBuilder;
use kivact::view::View;
use kivact::view::icon;
use kivact::view::node;
use kivact::view::node_ref;
use kivact::view::row;

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
    presentation: DatePresentation,
    value: Binding<Option<TimeOfDay>>,
    seconds: Value<bool>,
}

pub fn time_input(label: impl IntoText, value: impl IntoBinding<Option<TimeOfDay>>) -> TimeInput {
    TimeInput {
        root: node(),
        label: label.into_text_value(),
        value: value.into_binding(),
        presentation: DatePresentation::Adaptive,
        seconds: (false).into_value(),
    }
}

impl TimeInput {
    /// Choose how the picker opens.
    pub fn presentation(mut self, presentation: DatePresentation) -> Self {
        self.presentation = presentation;
        self
    }

    pub fn show_seconds(mut self, show: impl IntoValue<bool>) -> Self {
        self.seconds = show.into_value();
        self
    }
}

impl NodeBuilder for TimeInput {
    #[doc(hidden)]
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
            presentation,
        } = self;
        if presentation.sheet() {
            let open = signal(false);
            let target = node_ref();
            let trigger = date_touch::trigger(
                label,
                move || date_touch::time_label(value.get(), seconds.get()),
                "clock",
                open.into(),
                target,
                move || open.set(true),
            );
            let sheet = date_touch::sheet(label, open, Some(target), move || {
                date_touch::time_panel(value, seconds.get())
            });
            return root.child((trigger, sheet)).into_view();
        }
        let invalid = signal(false);
        let formatted = memo(move || {
            value
                .get()
                .map(|time| {
                    if seconds.get() {
                        format!(
                            "{:02}:{:02}:{:02}",
                            time.hour(),
                            time.minute(),
                            time.second()
                        )
                    } else {
                        format!("{:02}:{:02}", time.hour(), time.minute())
                    }
                })
                .unwrap_or_default()
        });
        let draft = signal(formatted.get_untracked());
        kivact::state::effect(move || {
            draft.set(formatted.get());
            invalid.set(false);
        });
        let input = text_input(label, draft)
            .placeholder(if seconds.get() { "HH:MM:SS" } else { "HH:MM" })
            .invalid(move || {
                invalid.get()
                    && !draft.get().trim().is_empty()
                    && parse_time(&draft.get(), seconds.get()).is_none()
            })
            .on_draft_commit(move |text| {
                if text.trim().is_empty() {
                    value.set(None);
                    invalid.set(false);
                    draft.set(String::new());
                } else if let Some(time) = parse_time(&text, seconds.get()) {
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
