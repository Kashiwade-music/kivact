//! Role: Date range field and its two-month popover.
use crate::calendar::{Calendar, calendar_range};
use crate::recipes_field::frame;
use crate::theme_access::tokens;
use kivact::control::{Node, NodeBuilder, icon, node, text};
use kivact::i18n::{DateRange, message};
use kivact::primitive::PartState;
use kivact::style::{Align, Direction, Length};
use kivact::ui::{ButtonVariant, DialogKind, button, dialog};
use kivact::view::{IntoText, IntoView, View, node_ref};
use kivact::{Binding, Value, signal};
use std::rc::Rc;

pub struct DateRangeInput {
    root: Node,
    label: Value<String>,
    value: Binding<Option<DateRange>>,
    configure: Option<Rc<dyn Fn(Calendar) -> Calendar>>,
}

pub fn date_range_input(
    label: impl IntoText,
    value: impl Into<Binding<Option<DateRange>>>,
) -> DateRangeInput {
    DateRangeInput {
        root: node(),
        label: label.into_text_value(),
        value: value.into(),
        configure: None,
    }
}

impl DateRangeInput {
    pub fn calendar(mut self, configure: impl Fn(Calendar) -> Calendar + 'static) -> Self {
        self.configure = Some(Rc::new(configure));
        self
    }
}

impl NodeBuilder for DateRangeInput {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for DateRangeInput {
    fn into_view(self) -> View {
        let Self {
            root,
            label,
            value,
            configure,
        } = self;
        let open = signal(false);
        let trigger_ref = node_ref();
        let placeholder = message("soft.date_range.placeholder", "Select date range");
        let shown = move || {
            value
                .get()
                .map(|range| format!("{} – {}", range.start(), range.end()))
                .unwrap_or_else(|| placeholder.get())
        };
        let trigger = button(label)
            .hide_label(true)
            .variant(ButtonVariant::Ghost)
            .child(icon("calendar"))
            .child(text(shown))
            .node_ref(trigger_ref)
            .style(move || {
                let t = tokens();
                frame(&t, &PartState::default(), false)
                    .height(t.sizes.control_md)
                    .direction(Direction::Row)
                    .align_items(Align::Center)
                    .gap(8)
                    .padding_x(8)
                    .color(if value.get().is_some() {
                        t.colors.text_primary
                    } else {
                        t.colors.text_secondary
                    })
            });
        let picker = dialog(label, open)
            .kind(DialogKind::Popover)
            .hide_title(true)
            .width(Length::Auto)
            .content(move || {
                let calendar = calendar_range(value)
                    .number_of_months(2)
                    .autofocus(true)
                    .on_commit(move || {
                        open.set(false);
                        trigger_ref.restore_focus();
                    });
                match &configure {
                    Some(configure) => configure(calendar),
                    None => calendar,
                }
            })
            .attach(trigger);
        root.child(picker).into_view()
    }
}
