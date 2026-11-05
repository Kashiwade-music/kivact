//! Role: Source catalogue sections built from the public component library.

use super::common::{card, cards_grid, half, muted, section, third, wrap};
use kivact::i18n::{CalendarDate, Weekday};
use kivact::interaction::MenuEntry;
use kivact::prelude::{
    ColorToken, IntoView, Node, NodeBuilder, Signal, column, dynamic, rgb, row, show, signal, text,
};
use kivact::state::Callback;
use kivact::style::GridPlacement;
use kivact::style::TextDecorationLine;
use kivact::style::{Align, Justify, pct, style};
use kivact::view::Placement;
use kivact::view::{rich_text, span as text_span};
use kivact_ui_pop as ui;

pub(super) fn feedback(notify: Callback<(ui::Hue, String, String)>) -> Node {
    let dismissed_info = signal(false);
    let dismissed_success = signal(false);
    section(
        "07",
        "Feedback",
        "アラート・トースト・ローディング・空状態",
        cards_grid((
            half(card(
                "アラート",
                column((
                    show(
                        move || !dismissed_info.get(),
                        move || {
                            let tone = kivact::state::IntoValue::into_value(
                                kivact::interaction::Tone::Info,
                            );
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
                    ),
                    show(
                        move || !dismissed_success.get(),
                        move || {
                            let tone = kivact::state::IntoValue::into_value(
                                kivact::interaction::Tone::Success,
                            );
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
                    ),
                    {
                        let tone = kivact::state::IntoValue::into_value(
                            kivact::interaction::Tone::Warning,
                        );
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
                        let tone =
                            kivact::state::IntoValue::into_value(kivact::interaction::Tone::Danger);
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
            )),
            half(card(
                "トースト通知",
                column((
                    muted("ボタンを押すと画面右下に表示されます。")
                        .style(style().font_size(12).line_height(1.5)),
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
                    ),
                    text("ローディング").style(
                        style()
                            .font_size(15)
                            .weight(600)
                            .line_height(1.2)
                            .min_height(20)
                            .margin_top(20)
                            .margin_bottom(16),
                    ),
                    wrap((
                        kivact::ui::spinner(),
                        kivact::ui::spinner().size(kivact::ui::ControlSize::Large),
                        kivact::ui::spinner().variant(ui::SpinnerLook::Dots),
                        crate::pop::loader::halo_loader(),
                    )),
                    row((
                        kivact::ui::skeleton()
                            .style(style().size(44, 44))
                            .style(style().radius(999)),
                        column((
                            kivact::ui::skeleton().style(style().size(pct(60), 12)),
                            kivact::ui::skeleton().style(style().size(pct(90), 12)),
                            kivact::ui::skeleton().style(style().size(pct(40), 12)),
                        ))
                        .style(style().gap(8).grow(1)),
                    ))
                    .style(
                        style()
                            .margin_top(20)
                            .padding(16)
                            .gap(14)
                            .border(1, ColorToken::Border)
                            .radius(10),
                    ),
                ))
                .style(style().gap(0)),
            )),
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
}

pub(super) fn overlays(
    confirm: Signal<bool>,
    form: Signal<bool>,
    sheet: Signal<bool>,
    command: Signal<bool>,
) -> Node {
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
                            crate::pop::loader::halo_loader(),
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
}

pub(super) fn navigation(notify: Callback<(ui::Hue, String, String)>) -> Node {
    let current = signal(2usize);
    section(
        "09", "Navigation", "パンくず・ステッパー・アコーディオン・タイムライン",
        cards_grid((
            card("パンくず・ステッパー", column((
                ui::breadcrumb(["ホーム", "依頼", "中央区 外郭地区", "REQ-1003"].into_iter().map(str::to_owned).collect::<Vec<_>>()),
                ui::stepper(
                    vec!["依頼受付".into(), "編成".into(), "作戦実行".into(), "報告".into()],
                    current,
                )
                .notes(vec!["10/01".into(), "10/02".into(), "進行中".into(), "—".into()]),
                wrap((
                    kivact::ui::button("戻る").variant(kivact::ui::ButtonVariant::Secondary).size(kivact::ui::ControlSize::Small).icon(kivact::view::icon("chevron-left")).test_id("pop-step-prev").disabled(move || current.get() == 0).on_press(move || current.update(|v| *v = v.saturating_sub(1))),
                    kivact::ui::button("次へ").size(kivact::ui::ControlSize::Small).icon(kivact::view::icon("chevron-right")).test_id("pop-step-next").disabled(move || current.get() == 4).on_press(move || {
                        current.update(|value| *value = (*value + 1).min(4));
                        if current.get() == 4 {
                            notify.call((ui::Hue::Mint, "すべての工程が完了しました".into(), "おつかれさまでした、先生！".into()));
                        }
                    }),
                )),
                text("アコーディオン").style(style().margin_top(4).font_size(15).weight(600)),
                column((
                    kivact::ui::disclosure("依頼の受け方は？", signal(true)).content(|| muted("ダッシュボードの「依頼」から内容を確認し、担当する生徒を選んで受注します。")),
                    kivact::ui::disclosure("活動ポイントの回復速度", signal(false)).content(|| muted("6分ごとに1ポイント回復します。上限を超えた分は受取ボックスに保管されます。")),
                    kivact::ui::disclosure("データのエクスポート", signal(false)).content(|| muted("テーブル右上の「CSV」からいつでも出力できます。")),
                )),
            )).style(style().gap(16))).style(style().grid_column(GridPlacement::span(7))),
            card("アクティビティ", column((
                ui::timeline_item("ミナト が作戦 REQ-1003 に出撃しました").detail("").timestamp("10分前").icon("sword").tone(ui::Hue::Blue)
                    .title_content( rich_text([
                        text_span("ミナト").weight(500).color(ColorToken::Text),
                        text_span(" が作戦 "),
                        text_span("REQ-1003").weight(500).color(ColorToken::Accent)
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
}

fn activity_title(name: &str, activity: &str) -> impl IntoView {
    rich_text([
        text_span(name).weight(500).color(ColorToken::Text),
        text_span(activity),
    ])
}

const SCHEDULE_EVENTS: [(u8, &str, &str, &str, u32, ui::Hue); 11] = [
    (
        1,
        "09:00",
        "朝の定例ミーティング",
        "会議室A · 全員",
        0x1f9bff,
        ui::Hue::Blue,
    ),
    (
        3,
        "10:00",
        "外郭地区パトロール",
        "ミナト・アリサ・セナ",
        0x1f9bff,
        ui::Hue::Blue,
    ),
    (
        3,
        "12:30",
        "ランチ（ミナト）",
        "カフェテリア",
        0xff7cb0,
        ui::Hue::Pink,
    ),
    (
        3,
        "16:00",
        "予算レビュー",
        "ユズ · オンライン",
        0xf2bf0a,
        ui::Hue::Yellow,
    ),
    (
        7,
        "13:00",
        "救護訓練",
        "ハルカ · 体育館",
        0x2fcfa6,
        ui::Hue::Mint,
    ),
    (
        10,
        "18:00",
        "REQ-1003 期限",
        "報告書提出",
        0xff5a6e,
        ui::Hue::Red,
    ),
    (10, "19:00", "打ち上げ", "給食部", 0xff7cb0, ui::Hue::Pink),
    (
        15,
        "14:00",
        "総力戦 作戦会議",
        "作戦室",
        0x9d86ff,
        ui::Hue::Blue,
    ),
    (
        17,
        "23:59",
        "お月見フェス 終了",
        "イベント",
        0xff7cb0,
        ui::Hue::Pink,
    ),
    (
        22,
        "11:00",
        "新入部員 面談",
        "コハル",
        0x2fcfa6,
        ui::Hue::Mint,
    ),
    (
        28,
        "15:00",
        "月次レポート",
        "全学園",
        0x1f9bff,
        ui::Hue::Blue,
    ),
];

pub(super) fn schedule() -> Node {
    let date = signal(CalendarDate::new(2026, 10, 3));
    section(
        "11",
        "Schedule",
        "カレンダー・予定リスト",
        cards_grid((
            kivact::ui::card(
                ui::calendar(date, CalendarDate::new(2026, 10, 3))
                    .weekend(|day| match day {
                        kivact::i18n::Weekday::Sunday => Some(ui::Hue::Red),
                        kivact::i18n::Weekday::Saturday => Some(ui::Hue::Blue),
                        _ => None,
                    })
                    .events(
                        SCHEDULE_EVENTS
                            .into_iter()
                            .map(|(day, _, _, _, color, _)| ui::CalendarEvent {
                                date: CalendarDate::new(2026, 10, day),
                                color: rgb(color),
                            })
                            .collect::<Vec<_>>(),
                    )
                    .test_id("pop-calendar"),
            )
            .style(style().grid_column(GridPlacement::span(5))),
            kivact::ui::card(
                column((
                    row((
                        text(move || {
                            let d = date.get();
                            format!(
                                "{}月{}日（{}）の予定",
                                d.month(),
                                d.day(),
                                ["月", "火", "水", "木", "金", "土", "日"]
                                    [d.weekday().days_since(Weekday::Monday)]
                            )
                        })
                        .style(style().font_size(15).weight(600).grow(1)),
                        kivact::ui::button("予定を追加")
                            .icon(kivact::view::icon("plus"))
                            .size(kivact::ui::ControlSize::Small)
                            .variant(kivact_ui_pop::ButtonTone::Soft),
                    ))
                    .style(style().align_items(Align::Center)),
                    dynamic(date, move |selected_date| {
                        let items = SCHEDULE_EVENTS
                            .into_iter()
                            .filter(|(day, _, _, _, _, _)| {
                                selected_date.year() == 2026
                                    && selected_date.month() == 10
                                    && selected_date.day() == *day
                            })
                            .map(|(_, time, title, subtitle, color, hue)| {
                                ui::agenda_item(time, title)
                                    .description(subtitle)
                                    .tone(hue)
                                    .period(if time < "12:00" { "AM" } else { "PM" })
                                    .on_details(move || {
                                        kivact::interaction::toaster().push(
                                            kivact::interaction::Toast::new(title)
                                                .tone(super::common::tone_of(hue))
                                                .body(subtitle),
                                        );
                                    })
                                    .style(style().border_left(3, rgb(color)))
                            })
                            .collect::<Vec<_>>();
                        if !items.is_empty() {
                            column(items).style(style().gap(8))
                        } else {
                            row((
                                kivact::view::icon("calendar"),
                                muted("予定はありません。ゆっくり休みましょう。"),
                            ))
                            .style(
                                style()
                                    .padding(22)
                                    .gap(8)
                                    .align_items(Align::Center)
                                    .justify_content(Justify::Center)
                                    .radius(8)
                                    .border(1, ColorToken::Border),
                            )
                        }
                    }),
                ))
                .style(style().gap(16)),
            )
            .style(style().grid_column(GridPlacement::span(7))),
        )),
    )
}

pub(super) fn cards(notify: Callback<(ui::Hue, String, String)>) -> Node {
    let items = [
        super::mission::Mission {
            code: "AREA 07".into(),
            title: "外郭地区のパトロール".into(),
            description: "不審なドローンの目撃情報。周辺の安全を確認してください。".into(),
            hue: ui::Hue::Blue,
            rank: "NORMAL".into(),
            rank_tone: ui::Hue::Blue,
            banner_icon: "building-community",
            progress: 66.0,
            progress_label: None,
            progress_value: "2 / 3".into(),
            rewards: vec!["×12,000".into(), "×2".into()],
            reward_icons: vec!["coin", "ticket"],
            footer: "残り 2日 4時間".into(),
            action_label: Some("出撃".into()),
            action_tone: kivact_ui_pop::ButtonTone::Accent,
            locked: false,
        },
        super::mission::Mission {
            code: "EVENT".into(),
            title: "秋のお月見フェスティバル".into(),
            description: "イベント限定ストーリーと報酬を手に入れよう。".into(),
            hue: ui::Hue::Pink,
            rank: "限定".into(),
            rank_tone: ui::Hue::Yellow,
            banner_icon: "gift",
            progress: 43.0,
            progress_label: Some("イベントポイント".into()),
            progress_value: "4,280 pt".into(),
            rewards: vec!["×1,200".into(), "家具セット".into()],
            reward_icons: vec!["sparkles", "gift"],
            footer: "10/17 まで".into(),
            action_label: Some("参加".into()),
            action_tone: kivact_ui_pop::ButtonTone::Pink,
            locked: false,
        },
        super::mission::Mission {
            code: "RAID".into(),
            title: "総力戦：機械仕掛けの大蛇".into(),
            description: "全部隊で挑む高難度コンテンツ。最強の編成で臨め。".into(),
            hue: ui::Hue::Gray,
            rank: "INSANE".into(),
            rank_tone: ui::Hue::Red,
            banner_icon: "crosshair",
            progress: 90.0,
            progress_label: Some("解放条件".into()),
            progress_value: "Lv.80".into(),
            rewards: vec!["称号".into(), "×200,000".into()],
            reward_icons: vec!["star", "coin"],
            footer: "あと 8 レベル".into(),
            action_label: Some("ロック中".into()),
            action_tone: kivact_ui_pop::ButtonTone::Soft,
            locked: true,
        },
    ];
    section(
        "12",
        "Mission Cards",
        "ミッションカード — 依頼・イベント・報酬",
        cards_grid(
            items
                .into_iter()
                .map(|mission| {
                    third(super::mission::mission_card(mission, move || {
                        notify.call((
                            ui::Hue::Yellow,
                            "報酬を獲得！".into(),
                            "クレジット ×12,000 / 活動報告書 ×2".into(),
                        ))
                    }))
                })
                .collect::<Vec<_>>(),
        ),
    )
}
