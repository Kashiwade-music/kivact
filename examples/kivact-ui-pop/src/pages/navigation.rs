//! Role: The Navigation page.

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
/// The page of the Navigation components.
pub fn page() -> Page {
    Page::new("navigation", "Navigation", "Components", include_str!("navigation.rs"))
        .description("Breadcrumbs, pagination, steppers and menus for moving through the app.")
        .example(Example::new(
            "navigation",
            "Navigation",
            "Each control moves between pages, steps or sections.",
            navigation_demo,
        ))
}

fn navigation_demo() -> impl IntoView {
    // example: navigation
    let notify = notify();
    let current = signal(2usize);
    section(
        "09", "Navigation", "パンくず・ステッパー・アコーディオン・タイムライン",
        cards_grid((
            card("パンくず・ステッパー", column((
                kivact::ui::breadcrumb(["ホーム", "依頼", "中央区 外郭地区", "REQ-1003"].into_iter().enumerate().map(|(key,label)|kivact::interaction::ListItem::new(key,label)).collect::<Vec<_>>()).separator(||kivact::view::icon("chevron-right")),
                kivact::ui::stepper(
                    [("依頼受付","10/01"),("編成","10/02"),("作戦実行","進行中"),("報告","—")].into_iter().map(|(label,note)|kivact::ui::StepperItem::new(label).description(note)).collect::<Vec<_>>(),
                    current,
                ).variant(ui::DiamondSteps).min_step_width(0),
                wrap((
                    kivact::ui::button("戻る").variant(kivact::ui::ButtonVariant::Secondary).size(kivact::ui::ControlSize::Small).icon(kivact::view::icon("chevron-left")).test_id("pop-step-prev").disabled(move || current.get() == 0).on_press(move || current.update(|v| *v = v.saturating_sub(1))),
                    kivact::ui::button("次へ").size(kivact::ui::ControlSize::Small).icon(kivact::view::icon("chevron-right")).test_id("pop-step-next").disabled(move || current.get() == 4).on_press(move || {
                        current.update(|value| *value = (*value + 1).min(4));
                        if current.get() == 4 {
                            notify.call((ui::Hue::Mint, "すべての工程が完了しました".into(), "おつかれさまでした、先生！".into()));
                        }
                    }),
                )),
                text("アコーディオン").style(style().margin_top(4).font_size(15).font_weight(600)),
                column((
                    kivact::ui::disclosure("依頼の受け方は？", signal(true)).content(|| muted("ダッシュボードの「依頼」から内容を確認し、担当する生徒を選んで受注します。")),
                    kivact::ui::disclosure("活動ポイントの回復速度", signal(false)).content(|| muted("6分ごとに1ポイント回復します。上限を超えた分は受取ボックスに保管されます。")),
                    kivact::ui::disclosure("データのエクスポート", signal(false)).content(|| muted("テーブル右上の「CSV」からいつでも出力できます。")),
                )),
            )).style(style().gap(16))).style(style().grid_column(GridPlacement::span(7))),
            card("アクティビティ", column((
                ui::timeline_item("ミナト が作戦 REQ-1003 に出撃しました").detail("").timestamp("10分前").icon("sword").tone(ui::Hue::Blue)
                    .title_content( rich_text([
                        text_span("ミナト").font_weight(500).color(color::TEXT),
                        text_span(" が作戦 "),
                        text_span("REQ-1003").font_weight(500).color(color::ACCENT)
                            .text_decoration(TextDecorationLine::UNDERLINE)
                            .text_underline_offset(3),
                        text_span(" に出撃しました"),
                    ])),
                ui::timeline_item("ハルカ が報告書を提出しました").detail("").timestamp("1時間前").icon("circle-check").tone(ui::Hue::Gray)
                    .title_content( activity_title("ハルカ", " が報告書を提出しました")),
                ui::timeline_item("ユズ の予算申請が承認されました").detail("弾薬補給 ¥120,000 × 4件").timestamp("3時間前").icon("coin").tone(ui::Hue::Gray)
                    .title_content( activity_title("ユズ", " の予算申請が承認されました")),
                ui::timeline_item("アリサ の絆ランクが 15 に上がりました").detail("").timestamp("昨日").icon("heart").tone(ui::Hue::Gray)
                    .title_content( activity_title("アリサ", " の絆ランクが 15 に上がりました")),
                ui::timeline_item("コハル が部活に加入しました").detail("").timestamp("2日前").icon("user-plus").tone(ui::Hue::Gray)
                    .title_content( activity_title("コハル", " が部活に加入しました")).style(style().padding_bottom(0)),
            ))).style(style().grid_column(GridPlacement::span(5))),
        )),
    )
    // end example
}

fn activity_title(name: &str, activity: &str) -> impl IntoView {
    rich_text([
        text_span(name).font_weight(500).color(color::TEXT),
        text_span(activity),
    ])
}
