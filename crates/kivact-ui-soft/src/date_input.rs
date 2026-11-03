//! Role: Typed date field and its attached calendar.
use crate::calendar::{Calendar, calendar};
use kivact::control::{Node, NodeBuilder, icon, node, row};
use kivact::i18n::{CalendarDate, message};
use kivact::primitive::NodeExt;
use kivact::style::{Align, Length, style};
use kivact::ui::{ButtonVariant, DialogKind, button, dialog, text_input};
use kivact::view::{Handled, IntoText, IntoView, Key, KeyEvent, View, node_ref};
use kivact::{Binding, Value, memo, signal};
use std::rc::Rc;

pub struct DateInput {
    root: Node,
    label: Value<String>,
    value: Binding<Option<CalendarDate>>,
    configure: Option<Rc<dyn Fn(Calendar) -> Calendar>>,
}

/// An editable date with an attached month calendar.
pub fn date_input(
    label: impl IntoText,
    value: impl Into<Binding<Option<CalendarDate>>>,
) -> DateInput {
    DateInput {
        root: node(),
        label: label.into_text_value(),
        value: value.into(),
        configure: None,
    }
}

impl DateInput {
    pub fn calendar(mut self, configure: impl Fn(Calendar) -> Calendar + 'static) -> Self {
        self.configure = Some(Rc::new(configure));
        self
    }
}

impl NodeBuilder for DateInput {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

pub(super) fn parse_date(text: &str) -> Option<CalendarDate> {
    let fields = text
        .trim()
        .split(['-', '/', '.', ' '])
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>();
    let [a, b, c] = fields.as_slice() else {
        return None;
    };
    if a.len() == 4 {
        CalendarDate::checked(a.parse().ok()?, b.parse().ok()?, c.parse().ok()?)
    } else {
        let month = [
            "jan", "feb", "mar", "apr", "may", "jun", "jul", "aug", "sep", "oct", "nov", "dec",
        ];
        let name = |s: &str| {
            month
                .iter()
                .position(|name| s.to_ascii_lowercase().starts_with(name))
                .map(|index| index as u8 + 1)
        };
        let (month, day) = if let Some(month) = name(a) {
            (month, b.trim_end_matches(',').parse().ok()?)
        } else if let Some(month) = name(b) {
            (month, a.parse().ok()?)
        } else {
            (a.parse().ok()?, b.parse().ok()?)
        };
        CalendarDate::checked(c.parse().ok()?, month, day)
    }
}

impl IntoView for DateInput {
    fn into_view(self) -> View {
        let Self {
            root,
            label,
            value,
            configure,
        } = self;
        let open = signal(false);
        let invalid = signal(false);
        let opened_from_input = signal(false);
        let input_ref = node_ref();
        let formatted = memo(move || value.get().map(|date| date.to_string()).unwrap_or_default());
        let draft = signal(formatted.get_untracked());
        kivact::effect(move || {
            draft.set(formatted.get());
            invalid.set(false);
        });
        let input = text_input(label, draft)
            .placeholder("YYYY-MM-DD")
            .expanded(open)
            .forward_keys(vec![Key::ArrowDown])
            .on_key(move |event: KeyEvent| {
                if event.key == Key::ArrowDown {
                    opened_from_input.set(true);
                    open.set(true);
                    Handled::Yes
                } else {
                    Handled::No
                }
            })
            .invalid(move || {
                invalid.get()
                    && !draft.get().trim().is_empty()
                    && parse_date(&draft.get()).is_none()
            })
            .draft(move |text| {
                if text.trim().is_empty() {
                    value.set(None);
                    open.set(false);
                    invalid.set(false);
                    draft.set(String::new());
                } else if let Some(date) = parse_date(&text) {
                    value.set(Some(date));
                    open.set(false);
                    invalid.set(false);
                    draft.set(formatted.get_untracked());
                } else {
                    invalid.set(true);
                }
            })
            .node_ref(input_ref);
        let toggle = button(message("soft.date.open", "Choose date"))
            .variant(ButtonVariant::Ghost)
            .hide_label(true)
            .icon(icon("calendar"))
            .expanded(open)
            .on_press(move || {
                opened_from_input.set(false);
                open.set(!open.get_untracked());
            });
        let trigger =
            root.child(row((toggle, input)).style(style().align_items(Align::Center).gap(4)));
        dialog(label, open)
            .kind(DialogKind::Popover)
            .hide_title(true)
            .width(Length::Auto)
            .content(move || {
                if opened_from_input.get_untracked() {
                    kivact::on_mount(move || input_ref.focus());
                }
                let calendar = calendar(value).on_commit(move || {
                    invalid.set(false);
                    draft.set(
                        value
                            .get_untracked()
                            .map(|date| date.to_string())
                            .unwrap_or_default(),
                    );
                    open.set(false);
                    input_ref.restore_focus();
                });
                match &configure {
                    Some(configure) => configure(calendar),
                    None => calendar,
                }
            })
            .attach(trigger)
    }
}
