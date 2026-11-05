//! Role: data specimens for the operation-console catalog.
use kivact::view::IntoView;

use super::common::{cards, line, list, segmented, specimen};
use super::state::{Demo, Record};
use kivact::interaction::{MenuEntry, Toast, Tone, toaster};
use kivact::platform;
use kivact::prelude::{
    ColorToken, Node, NodeBuilder, column, node, rgb, row, signal, spacer, text,
};
use kivact::style::{Align, Justify, pct, style};
use kivact::style::{Position, TextAlign};
use kivact::task::spawn;
use kivact::view::{rich_text, span};
use kivact_ui_game as g;
use std::time::Duration;

fn kpi_heading(icon: &'static str, label: &str) -> Node {
    row((
        kivact::view::icon(icon).style(style().size(13, 13)),
        text(label.to_owned()).style(style().font_size(11).weight(600)),
    ))
    .style(
        style()
            .gap(6)
            .align_items(Align::Center)
            .color(g::ColorName::Text3),
    )
}

pub(super) fn display(scene: Demo) -> Node {
    list((
        specimen(
            "DSP-01",
            "KPI タイル",
            cards(
                (
                    node().child({ let animated = kivact::motion::Animated::from(0.0, 1284., kivact::motion::Transition::tween(900, kivact::motion::Easing::CubicBezier(0.2,0.7,0.2,1.)));
    let formatted = kivact::state::memo(move || (move |value| format_count(value, 0))(animated.get()));
    let unit = kivact::state::IntoText::into_text_value("");
    kivact::ui::stat("完了案件", move || format!("{}{}", formatted.get(), unit.get()))
        .variant(g::StatVariant::Kpi).variant(kivact::interaction::Tone::Info)
        .value_content(row((text(formatted), text(unit).style(style().font_size(16).color(ColorToken::TextMuted)))).style(style().gap(2).align_items(Align::Baseline)))
.label_content(kpi_heading("target", "完了案件"))
.delta("↗ 12.4%")
.chart(g::sparkline(vec![0.24, 0.36, 0.3, 0.66, 0.56, 0.75, 0.58, 0.88])) })
                        .child(watermark("01")),
                    node().child({ let animated = kivact::motion::Animated::from(0.0, 99.97, kivact::motion::Transition::tween(900, kivact::motion::Easing::CubicBezier(0.2,0.7,0.2,1.)));
    let formatted = kivact::state::memo(move || (move |value| format_count(value, 2))(animated.get()));
    let unit = kivact::state::IntoText::into_text_value("%");
    kivact::ui::stat("稼働率", move || format!("{}{}", formatted.get(), unit.get()))
        .variant(g::StatVariant::Kpi).variant(kivact::interaction::Tone::Success)
        .value_content(row((text(formatted), text(unit).style(style().font_size(16).color(ColorToken::TextMuted)))).style(style().gap(2).align_items(Align::Baseline)))
.label_content(kpi_heading("activity", "稼働率"))
.delta("↗ 0.02pt")
.chart(g::sparkline(vec![0.24, 0.36, 0.3, 0.66, 0.56, 0.75, 0.58, 0.88])) })
                        .child(watermark("02")),
                    node().child({ let animated = kivact::motion::Animated::from(0.0, 4.2, kivact::motion::Transition::tween(900, kivact::motion::Easing::CubicBezier(0.2,0.7,0.2,1.)));
    let formatted = kivact::state::memo(move || (move |value| format_count(value, 1))(animated.get()));
    let unit = kivact::state::IntoText::into_text_value("h");
    kivact::ui::stat("平均処理時間", move || format!("{}{}", formatted.get(), unit.get()))
        .variant(g::StatVariant::Kpi).variant(kivact::interaction::Tone::Warning)
        .value_content(row((text(formatted), text(unit).style(style().font_size(16).color(ColorToken::TextMuted)))).style(style().gap(2).align_items(Align::Baseline)))
.label_content(kpi_heading("clock", "平均処理時間"))
.delta("↘ +0.6h")
.chart(g::sparkline(vec![0.24, 0.36, 0.3, 0.66, 0.56, 0.75, 0.58, 0.88])) })
                        .child(watermark("03")),
                    node().child({ let animated = kivact::motion::Animated::from(0.0, 7., kivact::motion::Transition::tween(900, kivact::motion::Easing::CubicBezier(0.2,0.7,0.2,1.)));
    let formatted = kivact::state::memo(move || (move |value| format_count(value, 0))(animated.get()));
    let unit = kivact::state::IntoText::into_text_value("");
    kivact::ui::stat("未解決インシデント", move || format!("{}{}", formatted.get(), unit.get()))
        .variant(g::StatVariant::Kpi).variant(kivact::interaction::Tone::Danger)
        .value_content(row((text(formatted), text(unit).style(style().font_size(16).color(ColorToken::TextMuted)))).style(style().gap(2).align_items(Align::Baseline)))
.label_content(kpi_heading("alert-octagon", "未解決インシデント"))
.delta("↗ +2")
.chart(g::sparkline(vec![0.24, 0.36, 0.3, 0.66, 0.56, 0.75, 0.58, 0.88])) })
                        .child(watermark("04")),
                ),
                220.,
            ).style(style().gap(12)),
        ),
        cards(
            (
                specimen(
                    "DSP-02",
                    "バッジ / ステータス",
                    line((
                        kivact::ui::badge("RUNNING").tone(kivact::interaction::Tone::Info),
                        kivact::ui::badge("PASSED").tone(kivact::interaction::Tone::Success),
                        kivact::ui::badge("PENDING").tone(kivact::interaction::Tone::Warning),
                        kivact::ui::badge("FAILED").tone(kivact::interaction::Tone::Danger),
                        kivact::ui::badge("ARCHIVED").tone(kivact::interaction::Tone::Neutral) .variant(g::BadgeVariant::Plain),
                        kivact::ui::badge("V2.4.0").tone(kivact::interaction::Tone::Info) .variant(g::BadgeVariant::Solid),
                        kivact::ui::badge("BETA").tone(kivact::interaction::Tone::Accent) .variant(g::BadgeVariant::Plain)
                            .style(style().color(rgb(0x13233a))),
                        kivact::ui::badge("PROD").tone(kivact::interaction::Tone::Info) .variant(g::BadgeVariant::Outline),
                        row((
                            text("受信箱"),
                            kivact::ui::badge("24").tone(kivact::interaction::Tone::Danger) .variant(g::BadgeVariant::Solid)
                                .style(style().padding_x(5)),
                        ))
                        .style(style().gap(6)),
                    )),
                ),
                specimen(
                    "DSP-03",
                    "プログレス",
                    list((
                        row((
                            text("同期").style(style().width(96).font_size(12)),
                            kivact::ui::progress(Some(0.68))
                                .name("同期")
                                .variant(kivact::interaction::Tone::Info)
                                .variant(g::Striped)
                                .moving(true)
                                .style(style().grow(1)),
                            g::mono("68%").style(style().width(44).text_align(TextAlign::End)),
                        ))
                        .style(style().gap(10).align_items(Align::Center)),
                        progress_row("検証", 100., kivact::interaction::Tone::Success),
                        progress_row("ストレージ", 86., kivact::interaction::Tone::Warning),
                        row((
                            text("解析中").style(style().width(96).font_size(12)),
                            kivact::ui::progress(None::<f64>)
                                .name("解析中")
                                .variant(kivact::interaction::Tone::Info)
                                .size(kivact::ui::ControlSize::Small)
                                .style(style().grow(1)),
                            g::mono("—").style(style().width(44).text_align(TextAlign::End)),
                        ))
                        .style(style().gap(10).align_items(Align::Center)),
                    )),
                ),
                specimen(
                    "DSP-04",
                    "ゲージ / リング",
                    row((
                        list((
                            line((
                                { let total: kivact::state::Value<usize> = kivact::state::IntoValue::into_value(10);
    let value: kivact::state::Value<usize> = kivact::state::IntoValue::into_value(7_usize);
    kivact::ui::progress(move || Some(if total.get()==0 {0.0} else {value.get() as f64/total.get() as f64}))
        .name("容量").variant(kivact::ui::ProgressVariant::Gauge)
        .variant(g::GaugeSegments(total)).variant(kivact::interaction::Tone::Info) },
                                text("7/10").style(
                                    style()
                                        .font_family("Barlow Condensed")
                                        .font_size(22)
                                        .italic(true)
                                        .weight(700),
                                ),
                            )),
                            line((
                                { let total: kivact::state::Value<usize> = kivact::state::IntoValue::into_value(10);
    let value: kivact::state::Value<usize> = kivact::state::IntoValue::into_value(2_usize);
    kivact::ui::progress(move || Some(if total.get()==0 {0.0} else {value.get() as f64/total.get() as f64}))
        .name("容量不足").variant(kivact::ui::ProgressVariant::Gauge)
        .variant(g::GaugeSegments(total)).variant(kivact::interaction::Tone::Danger) },
                                text("2/10").style(
                                    style()
                                        .font_family("Barlow Condensed")
                                        .font_size(22)
                                        .italic(true)
                                        .weight(700),
                                ),
                            )),
                            g::mono("CAPACITY UNITS").style(style().color(ColorToken::TextMuted)),
                        )),
                        { let value: kivact::state::Value<f64> = kivact::state::IntoValue::into_value(76.);
    kivact::ui::progress(move || Some(value.get()/100.0)).name("SLA")
        .variant(kivact::ui::ProgressVariant::Ring).variant(kivact::interaction::Tone::Info)
        .with_node(|node| node.child(g::mono("SLA").style(style().font_size(9).color(ColorToken::TextMuted)))) },
                    ))
                    .style(
                        style()
                            .justify_content(Justify::SpaceBetween)
                            .width(pct(100))
                            .align_items(Align::Center),
                    ),
                ),
                specimen(
                    "DSP-05",
                    "アバター",
                    list((
                        line((
                            kivact::ui::avatar("KM").fallback_content(|| text("KM")).variant(g::AvatarVariant { side: 40., color: rgb(0x1283f0) }).status(Some(kivact::interaction::Tone::Success)),
                            kivact::ui::avatar("SA").fallback_content(|| text("SA")).variant(g::AvatarVariant { side: 32., color: rgb(0x0c1f36) }).status(Some(kivact::interaction::Tone::Warning)),
                            kivact::ui::avatar("TN").fallback_content(|| text("TN")).variant(g::AvatarVariant { side: 24., color: rgb(0x12a06e) }).status(Some(kivact::interaction::Tone::Neutral)),
                            kivact::ui::avatar_group(vec![
                                ("HI", rgb(0x1283f0)), ("YU", rgb(0xd98300)), ("RK", rgb(0xe0364d)),
                            ].into_iter().map(|(name,color)| {
                                kivact::ui::avatar(name).fallback_content(move || text(name))
                                    .variant(g::AvatarVariant { side:32., color })
                                    .style(style().border(2,ColorToken::Surface))
                            }).collect::<Vec<_>>()).total(8usize).max(3usize),
                        )),
                        row((
                            kivact::ui::avatar("MS").fallback_content(|| text("MS")).variant(g::AvatarVariant { side: 32., color: rgb(0x0a62c2) }).status(None),
                            column((
                                text("真柴 ソウ").style(style().weight(700)),
                                g::mono("ADMIN · 整備部")
                                    .style(style().font_size(10).color(ColorToken::TextMuted)),
                            )),
                        ))
                        .style(style().gap(10).align_items(Align::Center)),
                    )),
                ),
                specimen(
                    "DSP-06",
                    "アクティビティ",
                    g::activity(vec![
                        g::ActivityEntry::new("09:40 · NOW", rich_text(vec![span("デプロイ #418").weight(700), span(" を production に開始")]), kivact::interaction::Tone::Info),
                        g::ActivityEntry::new("09:12", rich_text(vec![span("検証").weight(700), span(" 全 214 項目に合格")]), kivact::interaction::Tone::Success),
                        g::ActivityEntry::new("08:55", rich_text(vec![span("worker-2").weight(700), span(" がタイムアウト（自動復旧）")]), kivact::interaction::Tone::Danger),
                        g::ActivityEntry::new("08:30", rich_text(vec![span("真柴").weight(700), span(" が OPS-2041 を承認")]), kivact::interaction::Tone::Neutral),
                    ]),
                ),
                specimen(
                    "DSP-07",
                    "プロパティリスト",
                    kivact::ui::description_list((vec![
                        (
                            "ID".into(),
                            row((
                                g::mono("ops_01HZX4K9"),
                                spacer(),
                                g::tool_button("ID をコピー")
                                    .icon(kivact::view::icon("copy")).hide_label(true)
                                    .on_press(|| {
                                        spawn(platform::copy_text("ops_01HZX4K9"));
                                    }),
                            ))
                            .style(style().align_items(Align::Center)).into_view(),
                        ),
                        (
                            "STATUS".into(),
                            kivact::ui::badge("ACTIVE").tone(kivact::interaction::Tone::Success).into_view(),
                        ),
                        ("OWNER".into(), text("整備部 / 真柴").into_view()),
                        ("CREATED".into(), g::mono("2026-09-28 14:02").into_view()),
                    ]).into_iter().map(|(term, detail): (String, _)| kivact::ui::description(term, detail)).collect::<Vec<_>>()),
                ),
                specimen(
                    "DSP-08",
                    "機能カード",
                    crate::game::feature::feature_card(
                        "常時監視",
                        "全拠点のセンサーを 30 秒間隔で収集し、閾値を超えた時点でオンコールへ通知します。",
                        kivact::ui::button("ルールを編集").variant(kivact_ui_game::Slant)
                            .size(kivact::ui::ControlSize::Small)
                            .variant(kivact::ui::ButtonVariant::Secondary)
                            .on_press(move || scene.drawer.set(true)),
                    ).child(kivact::ui::badge("ON").tone(kivact::interaction::Tone::Success) .variant(g::BadgeVariant::Plain)
                        .style(style().position(Position::Absolute).right(16).top(16))),
                ),
            ),
            330.,
        ),
    ))
}
fn watermark(number: &str) -> kivact::view::Node {
    // A decoration that bleeds out of its card on purpose; the layout check
    // of the catalog ignores it by this id.
    text(number.to_owned())
        .style(
            style()
                .position(Position::Absolute)
                .right(-4)
                .top(-10)
                .font_family("Barlow Condensed")
                .font_size(64)
                .weight(800)
                .italic(true)
                .opacity(0.035),
        )
        .test_id(format!("deco-number-{number}"))
}
pub(super) fn progress_row(label: &str, value: f64, tone: kivact::interaction::Tone) -> Node {
    row((
        text(label.to_owned()).style(style().width(96).font_size(12)),
        kivact::ui::progress(Some(value / 100.0))
            .name(label.to_owned())
            .variant(tone)
            .style(style().grow(1)),
        g::mono(format!("{value:.0}%"))
            .style(style().width(44).text_align(kivact::style::TextAlign::End)),
    ))
    .style(style().align_items(Align::Center).gap(10))
}
pub(super) fn records() -> Vec<Record> {
    [
        (
            "OPS-2041",
            "第 3 区画 定期点検",
            "配電盤 12 基 / 冷却 4 基",
            "真柴 ソウ",
            "RUNNING",
            kivact::interaction::Tone::Info,
            2,
            68.,
            "10-03 09:38",
        ),
        (
            "OPS-2040",
            "資材庫 棚卸し（Q3）",
            "在庫 2,184 品目",
            "久遠 ミオ",
            "PASSED",
            kivact::interaction::Tone::Success,
            1,
            100.,
            "10-03 08:12",
        ),
        (
            "OPS-2039",
            "外周センサー 交換",
            "北側 6 基",
            "早瀬 リク",
            "PENDING",
            kivact::interaction::Tone::Warning,
            3,
            20.,
            "10-02 22:47",
        ),
        (
            "OPS-2038",
            "決済ゲートウェイ移行",
            "upstream v3 → v4",
            "白石 ナギ",
            "FAILED",
            kivact::interaction::Tone::Danger,
            3,
            54.,
            "10-02 19:05",
        ),
        (
            "OPS-2037",
            "監査ログ保管期間の延長",
            "180 日 → 400 日",
            "真柴 ソウ",
            "RUNNING",
            kivact::interaction::Tone::Info,
            2,
            41.,
            "10-02 16:30",
        ),
        (
            "OPS-2036",
            "夜間巡回ルート最適化",
            "8 ルート",
            "東雲 カイ",
            "PENDING",
            kivact::interaction::Tone::Warning,
            1,
            0.,
            "10-01 11:20",
        ),
        (
            "OPS-2035",
            "通信中継機 ファーム更新",
            "fw 4.2.1",
            "久遠 ミオ",
            "PASSED",
            kivact::interaction::Tone::Success,
            2,
            100.,
            "09-30 17:44",
        ),
        (
            "OPS-2034",
            "旧拠点データのアーカイブ",
            "1.8TB",
            "早瀬 リク",
            "ARCHIVED",
            kivact::interaction::Tone::Neutral,
            1,
            100.,
            "09-28 09:02",
        ),
    ]
    .into_iter()
    .map(
        |(id, title, description, owner, status, tone, priority, progress, updated)| Record {
            id: id.into(),
            title: title.into(),
            description: description.into(),
            owner: owner.into(),
            status: status.into(),
            tone,
            priority,
            progress,
            updated: updated.into(),
        },
    )
    .collect()
}

/// The owner's avatar color.
fn owner_color(owner: &str) -> kivact::style::Color {
    rgb(match owner {
        "久遠 ミオ" => 0x12a06e,
        "早瀬 リク" => 0xd98300,
        "白石 ナギ" => 0xe0364d,
        "東雲 カイ" => 0x0c1f36,
        _ => 0x0a62c2,
    })
}

/// Whether `text` contains the query, ignoring case.
fn contains(text: &str, query: &str) -> bool {
    text.to_lowercase().contains(&query.to_lowercase())
}

/// The columns of the table specimen. Their widths match CSS table
/// intrinsic minima and share the spare width: the title's border-box
/// minimum is 220 px and the progress wrapper adds 24 px of cell padding.
fn columns() -> Vec<kivact::ui::DataColumn<Record>> {
    vec![
        kivact::ui::DataColumn::new("ID", |r: &Record| r.id.clone())
            .width(80.)
            .cell(|r: &Record| g::GameCellVariant::Identifier(r.id.clone()))
            .compare(|a: &Record, b: &Record| a.id.cmp(&b.id))
            .filter(|r: &Record, q: &str| contains(&r.id, q)),
        kivact::ui::DataColumn::new("件名", |r: &Record| {
            format!("{} {}", r.title, r.description)
        })
        .width(220.)
        .cell(|r: &Record| g::GameCellVariant::Detail {
            title: r.title.clone(),
            description: r.description.clone(),
        })
        .compare(|a: &Record, b: &Record| a.title.cmp(&b.title))
        .filter(|r: &Record, q: &str| contains(&r.title, q) || contains(&r.description, q)),
        kivact::ui::DataColumn::new("担当", |r: &Record| r.owner.clone())
            .width(124.)
            .cell(|r: &Record| g::GameCellVariant::User {
                name: r.owner.clone(),
                initials: r.owner.chars().take(1).collect(),
                color: owner_color(&r.owner),
            })
            .compare(|a: &Record, b: &Record| a.owner.cmp(&b.owner))
            .filter(|r: &Record, q: &str| contains(&r.owner, q)),
        kivact::ui::DataColumn::new("状態", |r: &Record| r.status.clone())
            .width(118.)
            .cell(|r: &Record| g::GameCellVariant::Badge {
                label: r.status.clone(),
                tone: r.tone,
            })
            .compare(|a: &Record, b: &Record| a.status.cmp(&b.status))
            .filter(|r: &Record, q: &str| contains(&r.status, q)),
        kivact::ui::DataColumn::new("優先度", |r: &Record| format!("P{}", r.priority))
            .width(70.)
            .cell(|r: &Record| g::GameCellVariant::Badge {
                label: format!("P{}", r.priority),
                tone: if r.priority == 3 {
                    kivact::interaction::Tone::Danger
                } else {
                    kivact::interaction::Tone::Accent
                },
            })
            .compare(|a: &Record, b: &Record| a.priority.cmp(&b.priority)),
        kivact::ui::DataColumn::new("進捗", |r: &Record| format!("{:.0}%", r.progress))
            .width(144.)
            .cell(|r: &Record| g::GameCellVariant::Progress {
                value: r.progress,
                tone: if r.tone == kivact::interaction::Tone::Danger {
                    kivact::interaction::Tone::Danger
                } else if r.progress == 100. {
                    kivact::interaction::Tone::Success
                } else {
                    kivact::interaction::Tone::Info
                },
            })
            .compare(|a: &Record, b: &Record| a.progress.total_cmp(&b.progress)),
        kivact::ui::DataColumn::new("更新", |r: &Record| r.updated.clone())
            .width(110.)
            .compare(|a: &Record, b: &Record| a.updated.cmp(&b.updated)),
    ]
}

pub(super) fn table(scene: Demo) -> impl kivact::view::Component {
    let query = signal(String::new());
    let filter = signal(Some("all".to_owned()));
    let page_size = signal(Some(25_usize));
    let rows = move || {
        let wanted = filter.get().unwrap_or_else(|| "all".to_owned());
        scene
            .rows
            .get()
            .into_iter()
            .filter(|r| wanted == "all" || r.status == wanted)
            .collect::<Vec<_>>()
    };
    g::hud(column((
        row((
            g::mono("TBL-01").style(style().color(ColorToken::Accent)),
            text("案件テーブル").style(style().weight(700)),
            spacer(),
            g::mono(move || format!("{} selected", scene.selected.with(Vec::len)))
                .style(style().font_size(10.5).color(g::ColorName::Text3)),
        ))
        .style(
            style()
                .height(36)
                .padding_x(14)
                .gap(10)
                .align_items(Align::Center)
                .background(ColorToken::SurfaceAlt)
                .border_bottom(1, ColorToken::Border),
        ),
        row((
            kivact::ui::input_group(
                "",
                (kivact::ui::text_input("案件を検索", query)
                    .test_id("game-table-search")
                    .placeholder("ID・件名・担当で絞り込み")
                    .style(style().width(pct(100)).padding_left(34).padding_right(52)),),
            )
            .variant(g::GameInputGroupVariant::Adorned)
            .start(kivact::view::icon("search").style(style().size(15, 15)))
            .end(())
            .style(style().width(260)),
            segmented(
                "Status",
                super::controls::choice_options(&[
                    ("all", "すべて"),
                    ("RUNNING", "進行中"),
                    ("PENDING", "保留"),
                    ("FAILED", "失敗"),
                ]),
                filter,
            ),
            spacer(),
            kivact::ui::button("CSV")
                .variant(kivact_ui_game::Slant)
                .icon(kivact::view::icon("download"))
                .variant(kivact::ui::ButtonVariant::Ghost)
                .size(kivact::ui::ControlSize::Small)
                .on_press(move || {
                    toaster().push(
                        Toast::new("CSV エクスポート")
                            .body("表示中の案件を CSV に出力します。")
                            .tone(Tone::Info)
                            .duration(Some(Duration::from_secs(4))),
                    );
                }),
            kivact::ui::button("新規案件")
                .variant(kivact_ui_game::Slant)
                .icon(kivact::view::icon("plus"))
                .size(kivact::ui::ControlSize::Small)
                .on_press(move || scene.create.set(true)),
        ))
        .style(
            style()
                .padding_y(10)
                .padding_x(12)
                .gap(8)
                .wrap(true)
                .align_items(Align::Center)
                .border_bottom(1, ColorToken::Border),
        ),
        kivact::ui::data_table(rows, |r: &Record| r.id.clone(), columns())
            .selection(scene.selected)
            .style(style().border(0, ColorToken::Border))
            .sort(signal(Some((
                6,
                kivact::interaction::SortOrder::Descending,
            ))))
            .filter(query)
            .page(signal(0_usize), move || page_size.get().unwrap_or(25))
            .row_label(|r: &Record| r.id.clone())
            .footer(move |context| {
                let shown = context.model.keys();
                let total = context.model.row_count();
                let selected = context
                    .selection
                    .map(|selection| kivact::state::IntoValue::into_value(selection));
                row((
                    g::mono(kivact::i18n::message_with(
                        "game.table.shown",
                        "{shown} shown of {total}",
                        move || {
                            [
                                ("shown", shown.with(Vec::len).to_string()),
                                ("total", total.get().to_string()),
                            ]
                        },
                    ))
                    .style(style().grow(1)),
                    g::mono(kivact::i18n::message_with(
                        "game.table.selected",
                        "{count} selected",
                        move || {
                            [(
                                "count",
                                selected.map_or(0, |s| s.with(Vec::len)).to_string(),
                            )]
                        },
                    )),
                    row((
                        text("表示件数").style(style().font_size(11.5).color(g::ColorName::Text3)),
                        kivact::ui::select(
                            "Select",
                            vec![
                                kivact::interaction::ListItem::new(25_usize, "25"),
                                kivact::interaction::ListItem::new(50, "50"),
                                kivact::interaction::ListItem::new(100, "100"),
                            ],
                            page_size,
                        )
                        .placeholder("")
                        .name("表示件数")
                        .test_id("game-table-page-size")
                        .style(style().width(70).height(28)),
                    ))
                    .style(style().gap(10).align_items(Align::Center)),
                ))
                .style(
                    style()
                        .padding_x(12)
                        .gap(16)
                        .height(48)
                        .align_items(Align::Center),
                )
            })
            .row_actions(42., move |record| {
                g::tool_button(move || format!("{} の詳細", record.get().id))
                    .icon(kivact::view::icon("chevron-right"))
                    .hide_label(true)
                    .style(
                        style()
                            .opacity(0.)
                            .group_hover(style().opacity(1.))
                            .focus_visible(style().opacity(1.)),
                    )
                    .on_press(move || scene.drawer.set(true))
            })
            .toolbar(move |context| {
                let selection = context.selection.expect("the table has a selection");
                let selected = kivact::state::IntoValue::into_value(selection);
                row((
                    g::mono(kivact::i18n::message_with(
                        "game.table.selected",
                        "{count} selected",
                        move || [("count", selected.with(Vec::len).to_string())],
                    ))
                    .style(style().grow(1).font_size(18).color(g::ColorName::Cyan)),
                    row((
                        kivact::ui::button("一括承認")
                            .variant(kivact_ui_game::Slant)
                            .test_id("game-bulk-approve")
                            .icon(kivact::view::icon("check"))
                            .size(kivact::ui::ControlSize::Small)
                            .variant(kivact::ui::ButtonVariant::Ghost)
                            .style(style().color(ColorToken::AccentText))
                            .on_press(move || {
                                let selected = selected.get();
                                scene.rows.update(|rows| {
                                    for row in rows {
                                        if selected.contains(&row.id) {
                                            row.approve();
                                        }
                                    }
                                });
                                toaster().push(
                                    Toast::new("処理が完了しました")
                                        .body(format!("{} 件を承認しました。", selected.len()))
                                        .tone(Tone::Success)
                                        .duration(Some(Duration::from_secs(4))),
                                );
                            }),
                        kivact::ui::menu("担当変更", vec![MenuEntry::item("soh", "真柴 ソウ")])
                            .on_activate(move |_: &'static str| {
                                scene.rows.update(|rows| {
                                    for row in rows {
                                        if selected.with_untracked(|ids| ids.contains(&row.id)) {
                                            row.owner = "真柴 ソウ".into();
                                        }
                                    }
                                })
                            })
                            .offset(0.)
                            .attach(
                                kivact::ui::button("担当変更")
                                    .variant(kivact_ui_game::Slant)
                                    .variant(kivact::ui::ButtonVariant::Secondary),
                            ),
                        kivact::ui::button("削除")
                            .variant(kivact_ui_game::Slant)
                            .icon(kivact::view::icon("trash"))
                            .size(kivact::ui::ControlSize::Small)
                            .variant(kivact::ui::ButtonVariant::Danger)
                            .on_press(move || scene.delete.set(true)),
                    ))
                    .style(style().gap(10).align_items(Align::Center)),
                    kivact::ui::button(kivact::i18n::message(
                        "game.table.clear",
                        "Clear selection",
                    ))
                    .variant(kivact_ui_game::Slant)
                    .size(kivact::ui::ControlSize::Small)
                    .variant(kivact::ui::ButtonVariant::Ghost)
                    .style(style().color(ColorToken::AccentText))
                    .on_press(move || selection.set(Vec::new())),
                ))
                .hidden(move || selected.with(Vec::is_empty))
                .role(kivact::view::Role::Group)
                .name(kivact::i18n::message("game.table.bulk", "Selected records"))
                .live_region(kivact::view::LiveRegion::Polite)
                .style(
                    style()
                        .padding_x(12)
                        .padding_y(8)
                        .align_items(Align::Center),
                )
                .style(move || {
                    let t = &kivact::style::use_theme().get();
                    style()
                        .background(t.ext::<kivact_ui_game::Tokens>().navy)
                        .color(t.colors().accent_text)
                })
            }),
    )))
    .test_id("TBL-01")
}

fn format_count(value: f64, decimals: usize) -> String {
    let plain = format!("{:.*}", decimals.min(9), value);
    let (whole, fraction) = plain
        .split_once('.')
        .map_or((plain.as_str(), None), |(a, b)| (a, Some(b)));
    let mut grouped = String::new();
    for (index, character) in whole.chars().enumerate() {
        if index > 0
            && (whole.len() - index).is_multiple_of(3)
            && character.is_ascii_digit()
            && whole.as_bytes()[index - 1].is_ascii_digit()
        {
            grouped.push(',');
        }
        grouped.push(character);
    }
    if let Some(fraction) = fraction {
        grouped.push('.');
        grouped.push_str(fraction);
    }
    grouped
}
