//! Role: Adaptive date/time sheet surfaces and direct-manipulation picker wheels.
use crate::bottom_sheet::{SheetHeight, bottom_sheet};
use crate::calendar::{Calendar, calendar, month_title};
use crate::recipes_field::frame;

use crate::time_of_day::TimeOfDay;
use kivact::custom::PartState;
use kivact::custom::clock_now;
use kivact::i18n::{CalendarDate, CalendarMonth, message};
use kivact::interaction::DragVelocity;
use kivact::interaction::nearest_snap;
use kivact::interaction::project_release;
use kivact::motion::Spring;
use kivact::motion::Transition;
use kivact::motion::animated;
use kivact::state::Binding;
use kivact::state::IntoBinding;
use kivact::state::IntoText;
use kivact::state::IntoValue;
use kivact::state::Signal;
use kivact::state::Value;
use kivact::state::binding;
use kivact::state::memo;
use kivact::state::signal;
use kivact::state::var;
use kivact::style::Align;
use kivact::style::Direction;
use kivact::style::Justify;
use kivact::style::Position;
use kivact::style::pct;
use kivact::style::style;
use kivact::ui::ButtonVariant;
use kivact::ui::button;
use kivact::view::A11yAction;
use kivact::view::Handled;
use kivact::view::IntoView;
use kivact::view::Key;
use kivact::view::KeyEvent;
use kivact::view::Node;
use kivact::view::NodeBuilder;
use kivact::view::NodeRef;
use kivact::view::PointerEvent;
use kivact::view::PointerId;
use kivact::view::PointerPhase;
use kivact::view::Role;
use kivact::view::WheelEvent;
use kivact::view::column;
use kivact::view::dynamic;
use kivact::view::icon;
use kivact::view::node;
use kivact::view::row;
use kivact::view::text;
use std::rc::Rc;
use std::time::Duration;

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
            Self::Adaptive => kivact::platform::capabilities().get().touch,
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
        .with_node(move |node| node.role(Role::ComboBox))
        .with_node(move |node| node.with_node(move |node| node.expanded(open)))
        .node_ref(target)
        .on_press(press)
        .on_key(move |event: KeyEvent| {
            if event.key() == Key::ArrowDown {
                target.activate();
                Handled::Yes
            } else {
                Handled::No
            }
        })
        .style(move || {
            let t = kivact::style::use_tokens::<crate::Tokens>();
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
    let month: Binding<CalendarMonth> = signal(initial).into_binding();
    let wheels = signal(false);
    let month_label = month_title(month.into_value());
    let caption = button(month_label)
        .variant(ButtonVariant::Ghost)
        .trailing(icon("chevron-down"))
        .with_node(move |node| node.with_node(move |node| node.expanded(wheels)))
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
    y: f64,
    dragging: bool,
}

const WHEEL_ROW: f64 = 28.0;
const WHEEL_HEIGHT: f64 = 264.0;
const RELEASE_STOPS: [f64; 17] = [
    -224.0, -196.0, -168.0, -140.0, -112.0, -84.0, -56.0, -28.0, 0.0, 28.0, 56.0, 84.0, 112.0,
    140.0, 168.0, 196.0, 224.0,
];

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
    let velocity = var(DragVelocity::new(Duration::from_millis(100)));
    let release = signal(0u64);
    let offset = animated(
        move || {
            release.get();
            0.0f64
        },
        Transition::spring(Spring {
            stiffness: 246.7,
            damping: 31.4,
            mass: 1.0,
        }),
    );
    let settle = move |from: f64| {
        if kivact::platform::is_motion_reduced().get() {
            offset.jump_to(0.0);
        } else {
            offset.jump_to(from.clamp(-112.0, 112.0));
            release.update(|release| *release = release.wrapping_add(1));
        }
    };
    // The band stays fixed while the labels move. Extra labels prevent a
    // projected release from exposing the ends of the rendered strip.
    let band = node().hit_testable(false).style(move || {
        let t = kivact::style::use_tokens::<crate::Tokens>();
        style()
            .position(Position::Absolute)
            .left(0)
            .right(0)
            .top((WHEEL_HEIGHT - WHEEL_ROW) / 2.0)
            .height(WHEEL_ROW)
            .radius(t.radius.element)
            .background(t.colors.neutral)
    });
    let rows = (-8i32..=8)
        .map(|delta| {
            let shown = memo(move || {
                let raw = value.get() + delta;
                if !wraps && !(min..=max).contains(&raw) {
                    String::new()
                } else {
                    format(normalize(raw))
                }
            });
            node()
                .hit_testable(false)
                .child(text(shown))
                .style(move || {
                    let t = kivact::style::use_tokens::<crate::Tokens>();
                    let distance = delta as f64 * WHEEL_ROW + offset.get();
                    let falloff = (distance.abs() / 146.0).min(1.0);
                    let selected = distance.abs() <= WHEEL_ROW / 2.0;
                    let base = style()
                        .position(Position::Absolute)
                        .left(0)
                        .right(0)
                        .top((WHEEL_HEIGHT - WHEEL_ROW) / 2.0 + delta as f64 * WHEEL_ROW)
                        .height(WHEEL_ROW)
                        .translate_y(offset.get())
                        .align_items(Align::Center)
                        .justify_content(Justify::Center)
                        .font_size(17)
                        .opacity(1.0 - 0.7 * falloff)
                        .weight(if selected { 600 } else { 400 })
                        .color(if selected {
                            t.colors.text_accent
                        } else {
                            t.colors.text_primary
                        });
                    if kivact::platform::is_motion_reduced().get() {
                        base
                    } else {
                        let scale = 1.0 - 0.14 * falloff;
                        base.scale_xy(scale, scale * (52.0 * falloff).to_radians().cos())
                    }
                })
        })
        .collect::<Vec<_>>();
    node()
        .child(band)
        .child(rows)
        .with_node(move |node| node.role(Role::SpinButton))
        .name(label)
        .focusable(true)
        .value_min(f64::from(min))
        .value_max(f64::from(max))
        .value_now(move || f64::from(value.get()))
        .value_text(move || format(value.get()))
        .a11y_action(A11yAction::Increment, move |_| step(1))
        .a11y_action(A11yAction::Decrement, move |_| step(-1))
        .on_key(move |event: KeyEvent| {
            if event.modifiers().ctrl || event.modifiers().meta || event.modifiers().alt {
                return Handled::No;
            }
            match event.key() {
                Key::ArrowUp => step(-1),
                Key::ArrowDown => step(1),
                Key::PageUp => step(-10),
                Key::PageDown => step(10),
                Key::Home => value.set(min),
                Key::End => value.set(max),
                _ => return Handled::No,
            }
            offset.jump_to(0.0);
            Handled::Yes
        })
        .on_wheel(move |event: WheelEvent| {
            if !event.delta().y.is_finite() || event.delta().y == 0.0 {
                return Handled::No;
            }
            step(event.delta().y.signum() as i32);
            offset.jump_to(0.0);
            Handled::Yes
        })
        .on_pointer(move |event: PointerEvent| {
            let pos = event.window_pos();
            let now = clock_now();
            if !pos.y.is_finite() {
                return Handled::No;
            }
            match event.phase() {
                PointerPhase::Down(_) if event.is_primary() => {
                    event.reserve_touch_gesture();
                    offset.jump_to(0.0);
                    velocity.update(|velocity| velocity.reset(now, pos));
                    contact.set(Some(Contact {
                        pointer: event.id(),
                        y: pos.y,
                        dragging: false,
                    }));
                }
                PointerPhase::Move => {
                    let Some(mut drag) = contact.get().filter(|drag| drag.pointer == event.id())
                    else {
                        return Handled::No;
                    };
                    let travel = drag.y - pos.y;
                    drag.dragging |= travel.abs() >= 4.0;
                    velocity.update(|velocity| velocity.record(now, pos));
                    if drag.dragging {
                        let steps = (travel / WHEEL_ROW).trunc() as i32;
                        if steps != 0 {
                            step(steps);
                            drag.y -= steps as f64 * WHEEL_ROW;
                        }
                        offset.jump_to(pos.y - drag.y);
                    }
                    contact.set(Some(drag));
                }
                PointerPhase::Up(_) => {
                    let Some(drag) = contact.get().filter(|drag| drag.pointer == event.id()) else {
                        return Handled::No;
                    };
                    contact.set(None);
                    if drag.dragging {
                        velocity.update(|velocity| velocity.record(now, pos));
                        let speed = velocity.with(|velocity| velocity.pixels_per_ms(now).y);
                        let residual = drag.y - pos.y;
                        let projected =
                            project_release(residual, -speed, Duration::from_millis(499));
                        let steps = nearest_snap(projected, &RELEASE_STOPS)
                            .map_or(0, |index| index as i32 - 8);
                        let before = value.get_untracked();
                        step(steps);
                        let after = value.get_untracked();
                        let applied = if before == after {
                            0
                        } else if wraps {
                            steps
                        } else {
                            after - before
                        };
                        settle(applied as f64 * WHEEL_ROW - residual);
                    } else {
                        let delta =
                            ((event.pos().y - WHEEL_HEIGHT / 2.0) / WHEEL_ROW).round() as i32;
                        let steps = delta.clamp(-4, 4);
                        let before = value.get_untracked();
                        step(steps);
                        if before != value.get_untracked() {
                            settle(steps as f64 * WHEEL_ROW);
                        }
                    }
                }
                PointerPhase::Cancel => {
                    if contact.get().is_some_and(|drag| drag.pointer == event.id()) {
                        contact.set(None);
                        settle(offset.get_untracked());
                    }
                }
                _ => return Handled::No,
            }
            Handled::Yes
        })
        .style(move || {
            let t = kivact::style::use_tokens::<crate::Tokens>();
            style()
                .position(Position::Relative)
                .direction(Direction::Column)
                .grow(1)
                .min_width(48)
                .height(WHEEL_HEIGHT)
                .clip(true)
                .focus_visible(style().ring_with_offset(2, 3, t.colors.accent))
        })
}
