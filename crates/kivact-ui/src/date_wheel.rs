//! Role: Date and wall-clock wheels share scalar keyboard, pointer and assistive editing.
use crate::layer::with_recipe_style;
use crate::variants::Variants;
use kivact_core::custom::PartState;
use kivact_core::i18n::{TimeOfDay, message};
use kivact_core::state::{Binding, IntoValue, Value, binding, notifier};
use kivact_core::view::{
    EditPhase, Handled, Node, NodeBuilder, Role, WheelEvent, column, node, row, text,
};
use kivact_interaction::{
    DragCoordinates, DragPhase, ScalarRange, project_release, use_drag, use_scalar,
};
/// A single discrete wheel; the scalar owns drag cancellation and bounds.
pub(crate) fn wheel(
    label: Value<String>,
    value: Binding<f64>,
    min: f64,
    max: impl IntoValue<f64>,
    time: bool,
) -> Node {
    let release = notifier();
    let max = max.into_value();
    let range = move || {
        ScalarRange::new(min, max.get())
            .expect("invariant: picker bounds are ordered")
            .with_step(1.0)
            .expect("invariant: one is a positive step")
    };
    let settle_offset = kivact_core::motion::animated(
        move || {
            release.track();
            0.0f64
        },
        kivact_core::motion::Transition::spring(kivact_core::motion::Spring {
            stiffness: 246.7,
            damping: 31.4,
            mass: 1.0,
            rest: 1e-3,
        }),
    );
    let scalar = use_scalar(value, range).on_edit(move |phase| {
        if matches!(phase, EditPhase::Commit | EditPhase::Cancel) {
            release.notify();
        }
    });
    let row_height = move || wheel_row_height(time);
    let normalize = move |next: f64| {
        if time {
            min + (next - min).rem_euclid(max.get_untracked() - min + 1.0)
        } else {
            next.clamp(min, max.get_untracked())
        }
    };
    let drag = wheel_drag(scalar, row_height, normalize, settle_offset, release);
    let control = node()
        .part(scalar.input())
        .part(drag.source_with(move || value.get_untracked()))
        .name(label)
        .focusable(true)
        .on_key(move |event: kivact_core::view::KeyEvent| {
            use kivact_core::view::Key;
            let steps = match event.key() {
                Key::ArrowDown => -1.0,
                Key::ArrowUp => 1.0,
                Key::PageDown => -10.0,
                Key::PageUp => 10.0,
                _ => return Handled::No,
            };
            scalar.once(|| scalar.set(normalize(value.get_untracked() + steps)));
            Handled::Yes
        })
        .child(wheel_rows(scalar, min, max, time, settle_offset, release))
        .on_wheel(move |event: WheelEvent| {
            let delta = event.delta().y;
            if !delta.is_finite() || delta == 0.0 {
                return Handled::No;
            }
            scalar.once(|| scalar.set(normalize(value.get_untracked() + delta.signum())));
            release.notify();
            Handled::Yes
        });
    let pick: fn(&crate::recipes::UiRecipes) -> &crate::recipes::Recipe<PartState> = if time {
        |r| r.time_input_recipes().wheel_recipe()
    } else {
        |r| r.date_input_recipes().wheel_recipe()
    };
    with_recipe_style(
        node().with_node(|n| n.role(Role::Group)).child(control),
        pick,
        &Variants::default(),
        PartState::default,
    )
}
pub(crate) fn time_panel(value: Binding<Option<TimeOfDay>>, seconds: Value<bool>) -> Node {
    let time = binding(
        move || {
            value.get().unwrap_or(TimeOfDay {
                hour: 0,
                minute: 0,
                second: 0,
            })
        },
        move |time| value.set(Some(time)),
    );
    let hour = binding(
        move || f64::from(time.get().hour),
        move |hour| {
            let mut next = time.get_untracked();
            next.hour = hour as u8;
            time.set(next);
        },
    );
    let minute = binding(
        move || f64::from(time.get().minute),
        move |minute| {
            let mut next = time.get_untracked();
            next.minute = minute as u8;
            time.set(next);
        },
    );
    let second = binding(
        move || f64::from(time.get().second),
        move |second| {
            let mut next = time.get_untracked();
            next.second = second as u8;
            time.set(next);
        },
    );
    let wheels = kivact_core::view::dynamic(
        move || seconds.get(),
        move |show| {
            let mut wheels = vec![
                wheel(message("time.hour", "Hour"), hour, 0.0, 23.0, true),
                wheel(message("time.minute", "Minute"), minute, 0.0, 59.0, true),
            ];
            if show {
                wheels.push(wheel(
                    message("time.second", "Second"),
                    second,
                    0.0,
                    59.0,
                    true,
                ));
            }
            row(wheels)
        },
    );
    column((wheels,))
}

/// Render the five visible wheel rows from the authoritative scalar and settle offset.
fn wheel_rows(
    scalar: kivact_interaction::ScalarApi,
    min: f64,
    max: Value<f64>,
    time: bool,
    settle_offset: kivact_core::motion::Animated<f64>,
    release: kivact_core::state::Notifier,
) -> Node {
    column(
        (-2i32..=2)
            .map(move |offset| {
                text(move || {
                    release.track();
                    let next = scalar.value().get() + f64::from(offset);
                    if next < min || next > max.get() {
                        String::new()
                    } else {
                        format!("{next:02.0}")
                    }
                })
                .hit_testable(false)
                .style(move || {
                    let theme = kivact_core::style::use_theme().get();
                    let tokens = theme.ext::<crate::UiTokens>();
                    let tokens = if time {
                        &tokens.time_input
                    } else {
                        &tokens.date_input
                    };
                    kivact_core::style::style()
                        .height(tokens.wheel_row_height)
                        .translate_y(settle_offset.get())
                        .opacity(if offset == 0 { 1.0 } else { 0.45 })
                        .weight(if offset == 0 { 600 } else { 400 })
                })
            })
            .collect::<Vec<_>>(),
    )
}

/// Apply scalar scrubbing and release projection to the discrete wheel.
fn wheel_drag(
    scalar: kivact_interaction::ScalarApi,
    row_height: impl Fn() -> f64 + Copy + 'static,
    normalize: impl Fn(f64) -> f64 + Copy + 'static,
    settle_offset: kivact_core::motion::Animated<f64>,
    release: kivact_core::state::Notifier,
) -> kivact_interaction::DragApi<f64> {
    scalar
        .scrub(
            use_drag::<f64>(DragCoordinates::Window)
                .slop(6.0)
                .axis_lock()
                .track_velocity(),
        )
        .on_update(move |update| {
            let raw = update.payload - update.delta.y / row_height();
            scalar.set(normalize(raw));
            settle_offset.jump_to((raw - raw.round()) * row_height());
        })
        .on_end(move |update| {
            if update.phase == DragPhase::End {
                let shift = if kivact_core::platform::is_motion_reduced().get() {
                    update.delta.y
                } else {
                    project_release(
                        update.delta.y,
                        update.velocity.y,
                        std::time::Duration::from_millis(180),
                    )
                };
                scalar.set(normalize(update.payload - shift / row_height()));
            }
            release.notify();
        })
}

fn wheel_row_height(time: bool) -> f64 {
    let theme = kivact_core::style::use_theme().get();
    let tokens = theme.ext::<crate::UiTokens>();
    if time {
        tokens.time_input.wheel_row_height
    } else {
        tokens.date_input.wheel_row_height
    }
}
