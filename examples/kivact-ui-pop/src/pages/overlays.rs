//! Role: The Overlays page.

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
/// The page of the Overlays components.
pub fn page() -> Page {
    Page::new("overlays", "Overlays", "Components", include_str!("overlays.rs"))
        .description("Dialogs, drawers, popovers, menus and palettes that open above the page.")
        .example(Example::new(
            "overlays",
            "Overlays",
            "Each overlay opens from its trigger and closes on its own action.",
            overlays_demo,
        ))
}

fn overlays_demo() -> impl IntoView {
    // example: overlays
    let confirm = signal(false);
    let form = signal(false);
    let sheet = signal(false);
    let command = signal(false);
    let popover_open = signal(false);
    section(
        "08",
        "Overlays",
        "モーダル・ドロワー・メニュー・ツールチップ",
        cards_grid((
            third(card(
                "ダイアログ",
                column((
                    kivact::ui::button("確認ダイアログ").icon(kivact::view::icon("sword")).on_press(move || confirm.set(true)),
                    kivact::ui::button("フォームダイアログ").icon(kivact::view::icon("pencil"))
                        .variant(kivact::ui::ButtonVariant::Secondary).on_press(move || form.set(true)),
                    kivact::ui::button("ドロワーを開く").icon(kivact::view::icon("layout-sidebar-left-collapse"))
                        .variant(kivact_ui_pop::ButtonTone::Soft)
                        .on_press(move || sheet.set(true)),
                    kivact::ui::button("コマンドパレット　 Ctrl K").icon(kivact::view::icon("command"))
                        .variant(kivact::ui::ButtonVariant::Ghost)
                        .on_press(move || command.set(true)),
                ))
                .style(style().gap(16)),
            )),
            third(card(
                "ドロップダウン・ポップオーバー",
                wrap((
                    kivact::ui::menu("アクション", vec![
                            MenuEntry::item("rename", "名前を変更").with_shortcut("F2"),
                            MenuEntry::item("copy", "複製").with_shortcut("Ctrl D"),
                            MenuEntry::item("archive", "アーカイブ"),
                            MenuEntry::item("delete", "削除"),
                        ]).on_activate(|_: &'static str| {}),
                    kivact::ui::dialog("ヘイローとは？", popover_open).mode(kivact::interaction::DialogMode::Popover).content(move || {
                            column((
                            halo_loader(),
                            muted("キャラクターの頭上に浮かぶ光輪。このUIではアバターやローダーの装飾モチーフとして使っています。"),
                            row((
                                kivact::ui::button("わかった").size(kivact::ui::ControlSize::Small).on_press(move || popover_open.set(false)),
                                kivact::ui::button("詳しく").variant(kivact::ui::ButtonVariant::Ghost).size(kivact::ui::ControlSize::Small),
                            )).style(style().gap(8)),
                        ))
                        .style(style().gap(6))
                        }).attach(kivact::ui::button("ポップオーバー")
                        .variant(kivact_ui_pop::ButtonTone::Soft)),
                ))
                .style(style().gap(8).align_items(Align::Start)),
            )),
            third(card(
                "ツールチップ",
                wrap((
                    kivact::ui::tooltip("上に表示").placement(kivact::view::Placement::Below).align(kivact::view::OverlayAlign::Center).offset(6.0).placement(Placement::Above).attach(kivact::ui::button("上").variant(kivact::ui::ButtonVariant::Secondary)),
                    kivact::ui::tooltip("下に表示").placement(kivact::view::Placement::Below).align(kivact::view::OverlayAlign::Center).offset(6.0).placement(Placement::Below).attach(kivact::ui::button("下").variant(kivact::ui::ButtonVariant::Secondary)),
                    kivact::ui::tooltip("右に表示").placement(kivact::view::Placement::Below).align(kivact::view::OverlayAlign::Center).offset(6.0).placement(Placement::End).attach(kivact::ui::button("右").variant(kivact::ui::ButtonVariant::Secondary)),
                    muted("保存　Ctrl　S").style(style().font_size(12)),
                )),
            )),
        )),
    )
    // end example
}

fn request_dialog(
    form_dialog: Signal<bool>,
    notify: kivact::state::Callback<(ui::Hue, String, String)>,
) -> kivact::ui::Dialog {
    let request_title = signal(String::new());
    let request_priority = signal(Some("normal"));
    let request_detail = signal(String::new());
    let request_date = signal(CalendarDate::new(2026, 10, 10));
    kivact::ui::dialog("新しい依頼を作成", form_dialog)
        .content(move || {
            request_fields(
                request_title,
                request_priority,
                request_date,
                request_detail,
                form_dialog,
                notify,
            )
        })
        .footer(move || {
            row((
                kivact::ui::button("キャンセル")
                    .variant(kivact::ui::ButtonVariant::Ghost)
                    .on_press(move || form_dialog.set(false)),
                kivact::ui::button("作成する")
                    .disabled(move || request_title.get().trim().is_empty())
                    .test_id("pop-create-request")
                    .on_press(move || {
                        form_dialog.set(false);
                        notify.call((
                            ui::Hue::Mint,
                            "依頼を作成しました".into(),
                            "生徒たちに通知が送られます。".into(),
                        ));
                    }),
            ))
            .style(style().gap(8).justify_content(Justify::End))
        })
}

fn operation_confirmation(
    confirm: Signal<bool>,
    notify: kivact::state::Callback<(ui::Hue, String, String)>,
) -> kivact::ui::Dialog {
    let remember_confirmation = signal(false);
    kivact::ui::dialog("作戦を開始しますか？", confirm)
        .content(move || {
            column((
                text("活動ポイントを 20 消費して「外郭地区のパトロール」を開始します。"),
                row((
                    decorated_avatar(
                        kivact::state::IntoText::into_text_value("MN"),
                        AvatarLook::default().halo(true),
                    ),
                    decorated_avatar(
                        kivact::state::IntoText::into_text_value("HK"),
                        AvatarLook::default().hue(330.0).halo(true),
                    ),
                    decorated_avatar(
                        kivact::state::IntoText::into_text_value("YZ"),
                        AvatarLook::default().hue(45.0).halo(true),
                    ),
                    decorated_avatar(
                        kivact::state::IntoText::into_text_value("AR"),
                        AvatarLook::default().hue(160.0).halo(true),
                    ),
                ))
                .style(
                    style()
                        .gap(14)
                        .padding(20)
                        .justify_content(Justify::Center)
                        .background(color::SURFACE_ALT)
                        .border_radius(10),
                ),
                kivact::ui::checkbox("今後この確認を表示しない", remember_confirmation),
            ))
            .style(style().gap(16))
        })
        .footer(move || {
            row((
                kivact::ui::button("キャンセル")
                    .variant(kivact::ui::Slant)
                    .variant(kivact::ui::ButtonVariant::Secondary)
                    .on_press(move || confirm.set(false)),
                kivact::ui::button("出撃する")
                    .variant(kivact::ui::Slant)
                    .variant(kivact_ui_pop::ButtonTone::Accent)
                    .test_id("pop-confirm-start")
                    .on_press(move || {
                        confirm.set(false);
                        notify.call((
                            ui::Hue::Blue,
                            "作戦開始".into(),
                            "部隊が外郭地区へ出撃しました。".into(),
                        ));
                    }),
            ))
            .style(style().gap(8).justify_content(Justify::End))
        })
}

fn request_fields(
    request_title: Signal<String>,
    request_priority: Signal<Option<&'static str>>,
    request_date: Signal<CalendarDate>,
    request_detail: Signal<String>,
    form_dialog: Signal<bool>,
    notify: kivact::state::Callback<(ui::Hue, String, String)>,
) -> Node {
    column((
        kivact::ui::field(
            "依頼タイトル *",
            kivact::ui::text_input("依頼タイトル", request_title)
                .on_submit(move || {
                    if !request_title.get_untracked().trim().is_empty() {
                        form_dialog.set(false);
                        notify.call((
                            ui::Hue::Mint,
                            "依頼を作成しました".into(),
                            "生徒たちに通知が送られます。".into(),
                        ));
                    }
                })
                .placeholder("例：図書館の蔵書整理")
                .test_id("pop-request-title"),
        ),
        row((
            kivact::ui::field(
                "優先度",
                kivact::ui::select(
                    "優先度",
                    vec![
                        ListItem::new("normal", "通常"),
                        ListItem::new("high", "高"),
                        ListItem::new("urgent", "緊急"),
                    ],
                    request_priority,
                ),
            )
            .style(style().flex_grow(1)),
            kivact::ui::field(
                "期限",
                {
                    let value: kivact::state::Signal<kivact::i18n::CalendarDate> = request_date;
                    let value = kivact::state::binding(
                        move || Some(value.get()),
                        move |day| {
                            if let Some(day) = day {
                                value.set(day);
                            }
                        },
                    );
                    kivact::ui::date_input("依頼期限", value)
                        .presentation(kivact::ui::DatePresentation::Desktop)
                        .calendar(move |calendar| {
                            calendar
                                .today(CalendarDate::new(2026, 10, 3))
                                .month_title(|date| format!("{}.{:02}", date.year(), date.month()))
                        })
                }
                .style(style().width(180)),
            ),
        ))
        .style(style().gap(16)),
        kivact::ui::field(
            "詳細",
            kivact::ui::text_area("依頼詳細", request_detail).placeholder("依頼の内容を入力…"),
        ),
    ))
    .style(style().gap(16))
}
