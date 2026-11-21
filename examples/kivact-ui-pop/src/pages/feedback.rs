//! Role: The Feedback page.

use kivact::prelude::*;
use kivact::style::{Align, Color, GridPlacement, Justify, Track, color, pct, rgb, rgba, style};
use kivact::view::Node;
use kivact_ui_pop as ui;
use kivact::draw::{LineCap, LineJoin, Path, Stroke};
use kivact::geometry::Point;
use kivact::i18n::{CalendarDate, Weekday};
use kivact::interaction::{ListItem, MenuEntry, ScalarRange};
use kivact::motion::{Animated, Easing, Repeat, Transition};
use kivact::state::{Callback, IntoHandler, IntoText, IntoValue, Machine, MachineHandle, Signal, Step, Value, callback, machine};
use kivact::style::{Direction, Filter, Shadow, Shape, TextDecorationLine, linear_gradient};
use kivact::ui::RadioGroup;
use kivact::view::{Orientation, Placement, rich_text, span as text_span};
use kivact_ui_pop::Hue;
use std::time::Duration;
use catalog_shell::{Example, Page};
use super::common::*;
/// The page of the Feedback components.
pub fn page() -> Page {
    Page::new("feedback", "Feedback", "Components", include_str!("feedback.rs"))
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
        "アラート・トースト・ローディング・空状態",
        cards_grid((
            half(alert_examples(notify, dismissed_info, dismissed_success)),
            half(toast_examples(notify)),
            kivact::ui::card(
                kivact::ui::empty_state("まだ依頼がありません")
                    .icon(ui::empty_illustration())
                    .description("新しい依頼を作成して、生徒たちに仕事をお願いしましょう。")
                    .actions({
                        wrap((
                            kivact::ui::button("依頼を作成")
                                .icon(kivact::view::icon("plus"))
                                .on_press(move || {
                                    notify.call((
                                        ui::Hue::Mint,
                                        "依頼を作成しました".into(),
                                        "新しい依頼が追加されました。".into(),
                                    ))
                                }),
                            kivact::ui::button("テンプレートから")
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
            kivact::ui::alert("メンテナンスのお知らせ")
                .tone(tone)
                .icon(kivact::view::icon(move || {
                    kivact::style::Icon::named(match tone.get() {
                        kivact::interaction::Tone::Danger => "alert-circle",
                        kivact::interaction::Tone::Warning => "alert-triangle",
                        kivact::interaction::Tone::Success => "circle-check",
                        _ => "info-circle",
                    })
                }))
                .description("10/08 02:00〜06:00 の間、一部機能が利用できません。")
                .actions({
                    kivact::ui::button("お知らせを閉じる")
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
            kivact::ui::alert("作戦成功！")
                .tone(tone)
                .icon(kivact::view::icon(move || {
                    kivact::style::Icon::named(match tone.get() {
                        kivact::interaction::Tone::Danger => "alert-circle",
                        kivact::interaction::Tone::Warning => "alert-triangle",
                        kivact::interaction::Tone::Success => "circle-check",
                        _ => "info-circle",
                    })
                }))
                .description("報酬が受取ボックスに送られました。")
                .actions({
                    kivact::ui::button("成功通知を閉じる")
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
        "アラート",
        column((
            maintenance_notice(dismissed_info),
            success_notice(dismissed_success),
            {
                let tone = kivact::state::IntoValue::into_value(kivact::interaction::Tone::Warning);
                kivact::ui::alert("活動ポイントが残りわずかです")
                    .tone(tone)
                    .icon(kivact::view::icon(move || {
                        kivact::style::Icon::named(match tone.get() {
                            kivact::interaction::Tone::Danger => "alert-circle",
                            kivact::interaction::Tone::Warning => "alert-triangle",
                            kivact::interaction::Tone::Success => "circle-check",
                            _ => "info-circle",
                        })
                    }))
                    .description("あと 12 で上限に達します。今すぐ使う")
            },
            {
                let tone = kivact::state::IntoValue::into_value(kivact::interaction::Tone::Danger);
                kivact::ui::alert("同期に失敗しました")
                    .tone(tone)
                    .icon(kivact::view::icon(move || {
                        kivact::style::Icon::named(match tone.get() {
                            kivact::interaction::Tone::Danger => "alert-circle",
                            kivact::interaction::Tone::Warning => "alert-triangle",
                            kivact::interaction::Tone::Success => "circle-check",
                            _ => "info-circle",
                        })
                    }))
                    .description("通信状態を確認してから再試行してください。")
                    .actions({
                        kivact::ui::button("再試行")
                            .size(kivact::ui::ControlSize::Small)
                            .variant(kivact::ui::ButtonVariant::Secondary)
                            .on_press(move || {
                                notify.call((
                                    ui::Hue::Blue,
                                    "同期しています".into(),
                                    "再試行を開始しました。".into(),
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
        "トースト通知",
        column((
            muted("ボタンを押すと画面右下に表示されます。")
                .style(style().font_size(12).line_height(1.5)),
            toast_action_buttons(notify),
            text("ローディング").style(
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
            (ui::Hue::Mint, "成功"),
            (ui::Hue::Blue, "情報"),
            (ui::Hue::Yellow, "警告"),
            (ui::Hue::Red, "エラー"),
            (ui::Hue::Yellow, "報酬"),
        ]
        .into_iter()
        .map(|(hue, s)| {
            kivact::ui::button(s)
                .variant(if s == "報酬" {
                    kivact_ui_pop::ButtonTone::Accent
                } else {
                    kivact_ui_pop::ButtonTone::Soft
                })
                .icon(kivact::view::icon(match s {
                    "成功" => "circle-check",
                    "情報" => "info-circle",
                    "警告" => "alert-triangle",
                    "エラー" => "alert-circle",
                    _ => "gift",
                }))
                .on_press(move || {
                    notify.call((
                        hue,
                        match s {
                            "成功" => "保存しました",
                            "情報" => "新しいお知らせ",
                            "警告" => "期限が近づいています",
                            "エラー" => "送信に失敗しました",
                            _ => "報酬を獲得！",
                        }
                        .into(),
                        match s {
                            "成功" => "依頼の変更が反映されました。",
                            "情報" => "総力戦の新シーズンが始まりました。",
                            "警告" => "REQ-1003 の期限まで残り 6 時間です。",
                            "エラー" => "ネットワーク接続を確認してください。",
                            _ => "クレジット ×12,000 / 活動報告書 ×2",
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
            let p = draw.theme().ext::<Tokens>();
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
                p.cyan.mix_in(p.yellow, phase, draw.theme()),
            );
            draw.rounded_rect(
                Rect {
                    x: 14.0 + phase * 3.0,
                    y: 30.0,
                    width: 20.0 - phase * 6.0,
                    height: 8.0,
                },
                4.0,
                p.blue
                    .mix_in(Color::TRANSPARENT, 0.78 + phase * 0.11, draw.theme()),
            );
        })
}
