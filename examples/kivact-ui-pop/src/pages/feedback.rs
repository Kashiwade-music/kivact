//! Role: The Feedback page.

use super::common::*;
use catalog_shell::{Example, Page};
use kivact::draw::{LineCap, LineJoin};
use kivact::motion::{Animated, Easing, Repeat, Transition};
use kivact::prelude::*;
use kivact::style::GridPlacement;
use kivact::view::Node;
use kivact_ui_pop as ui;
/// The page of the Feedback components.
pub(crate) fn page() -> Page {
    Page::new(
        "feedback",
        "Feedback",
        "Components",
        include_str!("feedback.rs"),
    )
    .description("Alerts, toasts and notices that report the outcome of an action.")
    .example(Example::new(
        "feedback",
        "Alerts and toasts",
        "Alerts show a state; a toast slides in and expires on its own.",
        feedback_demo,
    ))
}

// xtask-allow(state): independent: dismissed_info and dismissed_success hide the maintenance and success notices; each is set only by its own notice's close button and never cleared
fn feedback_demo() -> impl IntoView {
    // example: feedback
    let notify = notify();
    let dismissed_info = signal(false);
    let dismissed_success = signal(false);
    section(
        "07",
        "Feedback",
        "Alerts, toasts, loading and empty state",
        cards_grid((
            half(alert_examples(notify, dismissed_info, dismissed_success)),
            half(toast_examples(notify)),
            kivact::ui::card(
                kivact::ui::empty_state("No requests yet")
                    .icon(ui::empty_illustration())
                    .description("Create a new request and ask your students for help.")
                    .actions({
                        wrap((
                            kivact::ui::button("Create request")
                                .icon(kivact::view::icon("plus"))
                                .on_press(move || {
                                    notify.call((
                                        ui::Hue::Mint,
                                        "Request created".into(),
                                        "A new request was added.".into(),
                                    ))
                                }),
                            kivact::ui::button("From template")
                                .variant(kivact::ui::ButtonVariant::Ghost),
                        ))
                    }),
            )
            .style(style().grid_column(GridPlacement::span(12))),
        )),
    )
    // end example
}

fn maintenance_notice(dismissed_info: Signal<bool>) -> impl IntoView {
    show(
        move || !dismissed_info.get(),
        move || {
            let tone = kivact::state::IntoValue::into_value(kivact::interaction::Tone::Info);
            kivact::ui::alert("Maintenance notice")
                .tone(tone)
                .icon(kivact::view::icon(move || {
                    kivact::style::Icon::named(match tone.get() {
                        kivact::interaction::Tone::Danger => "alert-circle",
                        kivact::interaction::Tone::Warning => "alert-triangle",
                        kivact::interaction::Tone::Success => "circle-check",
                        _ => "info-circle",
                    })
                }))
                .description("Some features are unavailable from 02:00 to 06:00 on 10/08.")
                .actions({
                    kivact::ui::button("Dismiss notice")
                        .variant(kivact::ui::ButtonVariant::Ghost)
                        .hide_label(true)
                        .icon(kivact::view::icon("x"))
                        .style(style().size(28, 28).margin_top(-4).margin_right(-8))
                        .on_press(move || dismissed_info.set(true))
                })
        },
    )
}

fn success_notice(dismissed_success: Signal<bool>) -> impl IntoView {
    show(
        move || !dismissed_success.get(),
        move || {
            let tone = kivact::state::IntoValue::into_value(kivact::interaction::Tone::Success);
            kivact::ui::alert("Mission success!")
                .tone(tone)
                .icon(kivact::view::icon(move || {
                    kivact::style::Icon::named(match tone.get() {
                        kivact::interaction::Tone::Danger => "alert-circle",
                        kivact::interaction::Tone::Warning => "alert-triangle",
                        kivact::interaction::Tone::Success => "circle-check",
                        _ => "info-circle",
                    })
                }))
                .description("The reward was sent to your inbox.")
                .actions({
                    kivact::ui::button("Dismiss success notice")
                        .variant(kivact::ui::ButtonVariant::Ghost)
                        .hide_label(true)
                        .icon(kivact::view::icon("x"))
                        .style(style().size(28, 28).margin_top(-4).margin_right(-8))
                        .on_press(move || dismissed_success.set(true))
                })
        },
    )
}

fn alert_examples(
    notify: Callback<(ui::Hue, String, String)>,
    dismissed_info: Signal<bool>,
    dismissed_success: Signal<bool>,
) -> kivact::ui::Card {
    card(
        "Alerts",
        column((
            maintenance_notice(dismissed_info),
            success_notice(dismissed_success),
            {
                let tone = kivact::state::IntoValue::into_value(kivact::interaction::Tone::Warning);
                kivact::ui::alert("Activity points are running low")
                    .tone(tone)
                    .icon(kivact::view::icon(move || {
                        kivact::style::Icon::named(match tone.get() {
                            kivact::interaction::Tone::Danger => "alert-circle",
                            kivact::interaction::Tone::Warning => "alert-triangle",
                            kivact::interaction::Tone::Success => "circle-check",
                            _ => "info-circle",
                        })
                    }))
                    .description("12 more reaches the cap. Use them now.")
            },
            {
                let tone = kivact::state::IntoValue::into_value(kivact::interaction::Tone::Danger);
                kivact::ui::alert("Sync failed")
                    .tone(tone)
                    .icon(kivact::view::icon(move || {
                        kivact::style::Icon::named(match tone.get() {
                            kivact::interaction::Tone::Danger => "alert-circle",
                            kivact::interaction::Tone::Warning => "alert-triangle",
                            kivact::interaction::Tone::Success => "circle-check",
                            _ => "info-circle",
                        })
                    }))
                    .description("Check the connection status, then try again.")
                    .actions({
                        kivact::ui::button("Retry")
                            .size(kivact::ui::ControlSize::Small)
                            .variant(kivact::ui::ButtonVariant::Secondary)
                            .on_press(move || {
                                notify.call((
                                    ui::Hue::Blue,
                                    "Syncing".into(),
                                    "Retry started.".into(),
                                ))
                            })
                    })
            },
        ))
        .style(style().gap(16)),
    )
}

fn toast_examples(notify: Callback<(ui::Hue, String, String)>) -> kivact::ui::Card {
    card(
        "Toast notifications",
        column((
            muted("Press a button to show it at the bottom right of the screen.")
                .style(style().font_size(12).line_height(1.5)),
            toast_action_buttons(notify),
            text("Loading").style(
                style()
                    .font_size(15)
                    .font_weight(600)
                    .line_height(1.2)
                    .min_height(20)
                    .margin_top(20)
                    .margin_bottom(16),
            ),
            wrap((
                kivact::ui::spinner(),
                kivact::ui::spinner().size(kivact::ui::ControlSize::Large),
                kivact::ui::spinner().variant(ui::SpinnerVariant::Dots),
                halo_loader(),
            )),
            row((
                kivact::ui::skeleton()
                    .style(style().size(44, 44))
                    .style(style().border_radius(999)),
                column((
                    kivact::ui::skeleton().style(style().size(pct(60), 12)),
                    kivact::ui::skeleton().style(style().size(pct(90), 12)),
                    kivact::ui::skeleton().style(style().size(pct(40), 12)),
                ))
                .style(style().gap(8).flex_grow(1)),
            ))
            .style(
                style()
                    .margin_top(20)
                    .padding(16)
                    .gap(14)
                    .border(1, color::BORDER)
                    .border_radius(10),
            ),
        ))
        .style(style().gap(0)),
    )
}

fn toast_action_buttons(notify: Callback<(ui::Hue, String, String)>) -> Node {
    wrap(
        [
            (ui::Hue::Mint, "Success"),
            (ui::Hue::Blue, "Info"),
            (ui::Hue::Yellow, "Warning"),
            (ui::Hue::Red, "Error"),
            (ui::Hue::Yellow, "Reward"),
        ]
        .into_iter()
        .map(|(hue, s)| {
            kivact::ui::button(s)
                .variant(if s == "Reward" {
                    kivact_ui_pop::ButtonTone::Accent
                } else {
                    kivact_ui_pop::ButtonTone::Soft
                })
                .icon(kivact::view::icon(match s {
                    "Success" => "circle-check",
                    "Info" => "info-circle",
                    "Warning" => "alert-triangle",
                    "Error" => "alert-circle",
                    _ => "gift",
                }))
                .on_press(move || {
                    notify.call((
                        hue,
                        match s {
                            "Success" => "Saved",
                            "Info" => "New notice",
                            "Warning" => "Deadline approaching",
                            "Error" => "Send failed",
                            _ => "Reward earned!",
                        }
                        .into(),
                        match s {
                            "Success" => "Request changes are saved.",
                            "Info" => "A new all-out battle season has started.",
                            "Warning" => "6 hours left until the REQ-1003 deadline.",
                            "Error" => "Check your network connection.",
                            _ => "Credits ×12,000 / activity report ×2",
                        }
                        .into(),
                    ))
                })
        })
        .collect::<Vec<_>>(),
    )
}

pub(crate) fn halo_loader() -> Node {
    let pulse = Animated::from(
        0.0f64,
        1.0f64,
        Transition::tween(800, Easing::EaseInOut)
            .repeat(Repeat::Forever)
            .alternate()
            .essential(),
    );
    node()
        .role(Role::Status)
        .name(message("spinner.loading", "Loading"))
        .style(style().size(48, 48))
        .draw(move |draw, _| {
            let phase = pulse.get();
            let mut ring = Path::new();
            for i in 0..=64 {
                let a = std::f64::consts::TAU * i as f64 / 64.0;
                let point = Point::new(24.0 + 20.0 * a.cos(), 20.0 - phase * 6.0 + 8.0 * a.sin());
                ring = if i == 0 {
                    ring.move_to(point)
                } else {
                    ring.line_to(point)
                };
            }
            draw.stroke_path(
                &ring,
                &Stroke::new(3.0).cap(LineCap::Round).join(LineJoin::Round),
                rgb(0x5cbde8).mix(ui::Hue::Yellow.accent(), phase, draw.theme()),
            );
            draw.rounded_rect(
                Rect {
                    x: 14.0 + phase * 3.0,
                    y: 30.0,
                    width: 20.0 - phase * 6.0,
                    height: 8.0,
                },
                4.0,
                ui::Hue::Blue
                    .accent()
                    .mix(Color::TRANSPARENT, 0.78 + phase * 0.11, draw.theme()),
            );
        })
}
