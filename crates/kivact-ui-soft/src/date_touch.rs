//! Role: Adaptive date/time sheet surfaces and direct-manipulation picker wheels.
use crate::bottom_sheet::{SheetHeight, bottom_sheet};
use crate::calendar::{Calendar, calendar, month_title};
use crate::recipes_field::frame;
use crate::theme_access::tokens;
use crate::time_of_day::TimeOfDay;
use kivact::control::{Node, NodeBuilder, column, icon, node, row, text};
use kivact::i18n::{CalendarDate, CalendarMonth, message};
use kivact::primitive::{NodeExt, PartState, part};
use kivact::style::{Align, Direction, Justify, pct, style};
use kivact::ui::{ButtonVariant, button};
use kivact::view::{
    Handled, IntoText, IntoValue, IntoView, Key, KeyEvent, NodeRef, PointerEvent, PointerId,
    PointerPhase, Role, WheelEvent, dynamic,
};
use kivact::{Binding, Signal, Value, binding, memo, signal, var};
use std::rc::Rc;

/// The date and time picker surface.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DatePresentation {
    #[default]
    Adaptive,
    Desktop,
    BottomSheet,
}

impl DatePresentation {
    pub(super) fn sheet(self) -> bool {
        match self {
            Self::Adaptive => kivact::platform::capabilities().touch,
            Self::Desktop => false,
            Self::BottomSheet => true,
        }
    }
}

pub(super) fn trigger(
    label: Value<String>,
    shown: impl IntoText,
    glyph: &'static str,
    open: Value<bool>,
    target: NodeRef,
    press: impl Fn() + 'static,
) -> impl IntoView {
    button(label)
        .hide_label(true)
        .variant(ButtonVariant::Ghost)
        .icon(icon(glyph))
        .child(text(shown))
        .role(Role::ComboBox)
        .expanded(open)
        .node_ref(target)
        .on_press(press)
        .on_key(move |event: KeyEvent| {
            if event.key == Key::ArrowDown {
                target.activate();
                Handled::Yes
            } else {
                Handled::No
            }
        })
        .style(move || {
            let t = tokens();
            frame(&t, &PartState::default(), false)
                .min_width(120)
                .height(t.sizes.control_md)
                .padding_x(8)
                .gap(8)
                .align_items(Align::Center)
                .justify_content(Justify::Start)
        })
}

pub(super) fn sheet<V: IntoView>(
    label: impl IntoText,
    open: Signal<bool>,
    target: Option<NodeRef>,
    content: impl Fn() -> V + 'static,
) -> impl IntoView {
    sheet_with_action(
        label,
        open,
        target,
        content,
        message("soft.date_time.save", "Save"),
        move || open.set(false),
    )
}

pub(super) fn sheet_with_action<V: IntoView>(
    label: impl IntoText,
    open: Signal<bool>,
    target: Option<NodeRef>,
    content: impl Fn() -> V + 'static,
    action_label: Value<String>,
    action: impl Fn() + Clone + 'static,
) -> impl IntoView {
    let sheet = bottom_sheet(label, open)
        .height(SheetHeight::Hug)
        .padding(0);
    let sheet = if let Some(target) = target {
        sheet.final_focus(target)
    } else {
        sheet
    };
    sheet.content(move || {
        column((
            content(),
            button(action_label)
                .on_press(action.clone())
                .style(style().width(pct(100))),
        ))
        .style(
            style()
                .gap(8)
                .padding_x(16)
                .padding_top(24)
                .padding_bottom(16),
        )
    })
}

pub(super) type ConfigureCalendar = Rc<dyn Fn(Calendar) -> Calendar>;

pub(super) fn date_panel(
    value: Binding<Option<CalendarDate>>,
    configure: Option<ConfigureCalendar>,
) -> Node {
    let initial = value
        .get_untracked()
        .unwrap_or_else(CalendarDate::today)
        .month_of();
    let base = calendar(value);
    let configured = match &configure {
        Some(configure) => configure(base),
        None => base,
    };
    let (initial, min, max) = configured.sheet_months(initial);
    let month: Binding<CalendarMonth> = signal(initial).into();
    let wheels = signal(false);
    let month_label = month_title(month.into_value());
    let caption = button(month_label)
        .variant(ButtonVariant::Ghost)
        .trailing_icon(icon("chevron-down"))
        .expanded(wheels)
        .on_press(move || wheels.set(!wheels.get_untracked()))
        .style(style().height(36).font_size(17).weight(600));
    let panel = dynamic(
        move || wheels.get(),
        move |wheels_open| {
            if !wheels_open {
                let calendar = calendar(value).displayed_month(month);
                let calendar = match &configure {
                    Some(configure) => configure(calendar),
                    None => calendar,
                };
                return calendar.touch_months(min, max).into_view();
            }
            let month_number = binding(
                move || i32::from(month.get().month()) - 1,
                move |index| {
                    let current = month.get_untracked();
                    month.set(CalendarMonth::new(current.year(), index as u8 + 1).clamp(min, max));
                },
            );
            let year = binding(
                move || month.get().year(),
                move |year| {
                    month.set(
                        CalendarMonth::new(year, month.get_untracked().month()).clamp(min, max),
                    );
                },
            );
            column((
                row((
                    wheel(
                        message("soft.date_time.month", "Month"),
                        month_number,
                        0,
                        11,
                        true,
                        |index| (index + 1).to_string(),
                    ),
                    wheel(
                        message("soft.date_time.year", "Year"),
                        year,
                        min.year(),
                        max.year(),
                        false,
                        |year| year.to_string(),
                    ),
                ))
                .style(style().gap(8)),
                button(message("soft.date_time.done", "Done"))
                    .variant(ButtonVariant::Secondary)
                    .on_press(move || wheels.set(false))
                    .style(style().width(pct(100))),
            ))
            .style(style().gap(8))
            .into_view()
        },
    );
    let reset = button(message("soft.date_time.reset", "Reset"))
        .variant(ButtonVariant::Ghost)
        .on_press(move || {
            value.set(None);
            let today = CalendarDate::today().month_of();
            if (min..=max).contains(&today) {
                month.set(today);
            }
        });
    column((
        row((caption, reset)).style(style().justify_content(Justify::SpaceBetween)),
        panel,
    ))
    .style(style().gap(8))
}

pub(super) fn time_label(time: Option<TimeOfDay>, seconds: bool) -> String {
    time.map_or_else(
        || {
            if seconds {
                "HH:MM:SS".to_owned()
            } else {
                "HH:MM".to_owned()
            }
        },
        |time| {
            let (hour, minute, second) = (time.hour(), time.minute(), time.second());
            if seconds {
                format!("{hour:02}:{minute:02}:{second:02}")
            } else {
                format!("{hour:02}:{minute:02}")
            }
        },
    )
}

pub(super) fn time_panel(value: Binding<Option<TimeOfDay>>, seconds: bool) -> Node {
    let midnight = || TimeOfDay::new(0, 0, 0).expect("midnight is valid");
    let component = move |position: usize| {
        binding(
            move || {
                let time = value.get().unwrap_or_else(midnight);
                i32::from([time.hour(), time.minute(), time.second()][position])
            },
            move |next| {
                let time = value.get_untracked().unwrap_or_else(midnight);
                let mut parts = [time.hour(), time.minute(), time.second()];
                parts[position] = next as u8;
                value.set(TimeOfDay::new(parts[0], parts[1], parts[2]));
            },
        )
    };
    row((
        wheel(
            message("soft.date_time.hour", "Hour"),
            component(0),
            0,
            23,
            true,
            two_digits,
        ),
        wheel(
            message("soft.date_time.minute", "Minute"),
            component(1),
            0,
            59,
            true,
            two_digits,
        ),
        seconds.then(|| {
            wheel(
                message("soft.date_time.second", "Second"),
                component(2),
                0,
                59,
                true,
                two_digits,
            )
        }),
    ))
    .style(style().gap(8).min_height(264))
}

fn two_digits(value: i32) -> String {
    format!("{value:02}")
}

#[derive(Clone, Copy)]
struct Contact {
    pointer: PointerId,
    y: f32,
    stepped: bool,
}

const WHEEL_ROW: f32 = 44.0;

fn wheel(
    label: Value<String>,
    value: Binding<i32>,
    min: i32,
    max: i32,
    wraps: bool,
    format: impl Fn(i32) -> String + Copy + 'static,
) -> Node {
    let normalize = move |value: i32| {
        if wraps {
            min + (value - min).rem_euclid(max - min + 1)
        } else {
            value.clamp(min, max)
        }
    };
    let step = move |delta| {
        let current = value.get_untracked();
        let next = normalize(current + delta);
        if next != current {
            value.set(next);
        }
    };
    let contact = var(None::<Contact>);
    let interaction = part(|input| input)
        .focusable(true)
        .value_text(move || format(value.get()))
        .on_key(move |event: KeyEvent| {
            match event.key {
                Key::ArrowUp => step(-1),
                Key::ArrowDown => step(1),
                Key::Home => value.set(min),
                Key::End => value.set(max),
                _ => return Handled::No,
            }
            Handled::Yes
        })
        .on_wheel(move |event: WheelEvent| {
            if event.delta.y == 0.0 {
                return Handled::No;
            }
            step(event.delta.y.signum() as i32);
            Handled::Yes
        })
        .on_pointer(move |event: PointerEvent| {
            let y = event.window_pos.y;
            match event.phase {
                PointerPhase::Down(_) if event.is_primary => {
                    contact.set(Some(Contact {
                        pointer: event.id,
                        y,
                        stepped: false,
                    }));
                }
                PointerPhase::Move => {
                    let Some(mut drag) = contact.get().filter(|drag| drag.pointer == event.id)
                    else {
                        return Handled::No;
                    };
                    let steps = ((drag.y - y) / WHEEL_ROW).trunc() as i32;
                    if steps != 0 {
                        step(steps);
                        drag.y -= steps as f32 * WHEEL_ROW;
                        drag.stepped = true;
                        contact.set(Some(drag));
                    }
                }
                PointerPhase::Up(_) => {
                    let Some(drag) = contact.get().filter(|drag| drag.pointer == event.id) else {
                        return Handled::No;
                    };
                    contact.set(None);
                    if !drag.stepped {
                        let delta = ((event.pos.y - 132.0) / WHEEL_ROW).round() as i32;
                        step(delta.clamp(-2, 2));
                    }
                }
                PointerPhase::Cancel => {
                    if contact.get().is_some_and(|drag| drag.pointer == event.id) {
                        contact.set(None);
                    }
                }
                _ => return Handled::No,
            }
            Handled::Yes
        });
    let rows = (-2..=2)
        .map(|delta| {
            let shown = memo(move || {
                let raw = value.get() + delta;
                if !wraps && !(min..=max).contains(&raw) {
                    String::new()
                } else {
                    format(normalize(raw))
                }
            });
            node().child(text(shown)).style(move || {
                let t = tokens();
                let base = style()
                    .height(WHEEL_ROW)
                    .width(pct(100))
                    .align_items(Align::Center)
                    .justify_content(Justify::Center)
                    .font_size(20);
                if delta == 0 {
                    base.background(t.colors.accent_muted)
                        .radius(t.radius.element)
                        .weight(600)
                } else {
                    base.color(t.colors.text_secondary)
                        .opacity(if delta.abs() == 2 { 0.5 } else { 0.8 })
                }
            })
        })
        .collect::<Vec<_>>();
    column(rows)
        .role(Role::SpinButton)
        .name(label)
        .part(interaction)
        .style(move || {
            let t = tokens();
            style()
                .direction(Direction::Column)
                .grow(1)
                .min_width(48)
                .height(264)
                .padding_y(22)
                .focus_visible(style().ring_with_offset(2, 3, t.colors.accent))
        })
}
