//! Role: Source catalogue sections built from the public component library.

use super::common::{card, cards_grid, columns, muted, section, third, wrap};
use kivact::prelude::{
    ColorToken, Node, NodeBuilder, Signal, column, node, rgb, row, signal, text,
};
use kivact::style::Direction;
use kivact::style::GridPlacement;
use kivact::style::{Align, Justify, pct, style};
use kivact::task::spawn;
use kivact::ui::ListItem;
use kivact::view::Orientation;
use kivact::{Callback, Value};
use kivact_ui_pop as ui;
use std::time::Duration;

pub(super) fn display(notify: Callback<(ui::Hue, String, String)>) -> Node {
    let badges = [
        ui::Hue::Blue,
        ui::Hue::Mint,
        ui::Hue::Yellow,
        ui::Hue::Red,
        ui::Hue::Pink,
        ui::Hue::Gray,
    ]
    .into_iter()
    .zip(["進行中", "完了", "保留", "期限切れ", "NEW", "下書き"])
    .map(|(hue, s)| {
        let badge = ui::badge(s).hue(hue);
        // Full-width labels retain their 12 px em width, 8 px side padding
        // and 1 px borders when the source's inline row wraps.
        if s.is_ascii() {
            badge
        } else {
            badge.style(style().min_width(s.chars().count() as f64 * 12.0 + 18.0))
        }
    })
    .collect::<Vec<_>>();
    let chip_tags = signal(vec!["爆発".to_string(), "貫通".into(), "神秘".into()]);
    let next_chip = signal(0usize);
    let live = signal(35.0f64);
    spawn::<()>(async move {
        loop {
            kivact::task::sleep(Duration::from_millis(900)).await;
            live.update(|n| {
                *n = if *n >= 100.0 {
                    8.0
                } else {
                    (*n + 7.0).min(100.0)
                }
            });
        }
    });
    section(
        "05",
        "Data Display",
        "バッジ・アバター・プログレス",
        cards_grid((
            third(card(
                "バッジ・チップ",
                column((
                    wrap(badges),
                    wrap((
                        ui::badge("オンライン")
                            .hue(ui::Hue::Mint)
                            .variant(ui::BadgeVariant::Dot),
                        ui::badge("離席中")
                            .hue(ui::Hue::Yellow)
                            .variant(ui::BadgeVariant::Dot),
                        ui::badge("オフライン")
                            .hue(ui::Hue::Gray)
                            .variant(ui::BadgeVariant::Dot),
                        ui::badge("Lv.90").variant(ui::BadgeVariant::Solid),
                        ui::badge("EX").variant(ui::BadgeVariant::Slant),
                    )),
                    ui::chip_list(chip_tags)
                        .on_add(move || {
                            let options = [
                                "振動",
                                "分解",
                                "重装甲",
                                "軽装備",
                                "特殊装甲",
                                "市街地",
                                "屋内",
                                "野外",
                            ];
                            let index = next_chip.get();
                            chip_tags
                                .update(|items| items.push(options[index % options.len()].into()));
                            next_chip.set(index + 1);
                        })
                        .on_remove(move |index| {
                            chip_tags.update(|items| {
                                items.remove(index);
                            })
                        })
                        .test_id("pop-chip-list"),
                ))
                .style(style().gap(20)),
            )),
            third(card(
                "アバター・ヘイロー",
                column((
                    wrap((
                        ui::avatar("MN").size(72).halo(true),
                        ui::avatar("HK").size(56).hue(330.0).halo(true),
                        ui::avatar("YZ").hue(45.0).halo(true),
                        ui::avatar("AR").size(32).hue(160.0),
                        ui::avatar("K").size(24).hue(265.0),
                    ))
                    .style(style().align_items(Align::End)),
                    row((
                        ui::avatar("MN").size(32),
                        ui::avatar("HK")
                            .size(32)
                            .hue(330.0)
                            .style(style().margin_left(-8)),
                        ui::avatar("YZ")
                            .size(32)
                            .hue(45.0)
                            .style(style().margin_left(-8)),
                        ui::avatar("AR")
                            .size(32)
                            .hue(160.0)
                            .style(style().margin_left(-8)),
                        node()
                            .child(
                                text("+8")
                                    .style(style().font_size(11).color(ColorToken::TextMuted)),
                            )
                            .style(
                                style()
                                    .size(32, 32)
                                    .radius(999)
                                    .margin_left(-8)
                                    .background(ColorToken::SurfaceAlt)
                                    .direction(Direction::Row)
                                    .align_items(Align::Center)
                                    .justify_content(Justify::Center),
                            ),
                        muted("12名が参加中").style(style().font_size(12).margin_left(8)),
                    ))
                    .style(style().align_items(Align::Center)),
                    row((
                        ui::avatar("HK").hue(330.0).halo(true).online(true),
                        column((
                            text("春日 ハルカ").style(style().weight(600)),
                            muted("セレスト女学院 · 救護騎士団").style(style().font_size(12)),
                            text("★★★").style(style().color(rgb(0xf2bf0a))),
                        ))
                        .style(style().grow(1)),
                        kivact::ui::button("プロフィール")
                            .size(kivact::ui::ControlSize::Small)
                            .variant(kivact_ui_pop::ButtonTone::Soft),
                    ))
                    .style(
                        style()
                            .padding(14)
                            .gap(14)
                            .align_items(Align::Center)
                            .background(ColorToken::SurfaceAlt)
                            .border(1, ColorToken::Border)
                            .radius(10),
                    ),
                ))
                .style(style().gap(20)),
            )),
            third(card(
                "プログレス",
                column((
                    column((
                        row((muted("EXP").style(style().grow(1)), text("7,820 / 10,000"))),
                        kivact::ui::progress(Some(0.78)).name("EXP"),
                    ))
                    .style(style().gap(8)),
                    column((
                        row((
                            muted("同期中…").style(style().grow(1)),
                            text(move || {
                                if live.get() >= 100.0 {
                                    "完了".to_string()
                                } else {
                                    format!("{:.0}%", live.get())
                                }
                            }),
                        )),
                        kivact::ui::progress(move || Some(live.get() / 100.0))
                            .name("同期中")
                            .variant(ui::Striped),
                    ))
                    .style(style().gap(8)),
                    column((
                        row((muted("絆ランク").style(style().grow(1)), text("Rank 14"))),
                        kivact::ui::progress(Some(0.6))
                            .name("絆ランク")
                            .variant(ui::Hue::Pink)
                            .variant(ui::Segments(10)),
                    ))
                    .style(style().gap(8)),
                    columns(
                        (
                            ui::progress_ring(72.0, "出席率", ui::Hue::Blue),
                            ui::progress_ring(45.0, "予算消化", ui::Hue::Yellow),
                            ui::progress_ring(91.0, "達成率", ui::Hue::Mint),
                        ),
                        3,
                    ),
                ))
                .style(style().gap(16)),
            )),
            ui::panel((
                row((
                    text("リスト・キーバリュー").style(style().font_size(15).weight(600).grow(1)),
                    kivact::ui::button("すべて表示")
                        .size(kivact::ui::ControlSize::Small)
                        .variant(kivact::ui::ButtonVariant::Ghost)
                        .trailing_icon(kivact::control::icon("chevron-right")),
                ))
                .style(style().align_items(Align::Center).margin_bottom(16)),
                columns(
                    (
                        column((
                            ui::list_item("第3章 作戦報告書")
                                .subtitle("更新：10/02 22:14 · ミナト")
                                .icon("file-text")
                                .tone(ui::Hue::Blue)
                                .trailing(move || {
                                    row((
                                        ui::badge("承認済").hue(ui::Hue::Mint),
                                        kivact::ui::button("報告書メニュー")
                                            .variant(kivact::ui::ButtonVariant::Ghost)
                                            .hide_label(true)
                                            .icon(kivact::control::icon("dots-vertical")),
                                    ))
                                    .style(style().gap(8))
                                }),
                            ui::list_item("弾薬補給 予算申請")
                                .subtitle("更新：10/02 18:40 · ユズ")
                                .icon("coin")
                                .tone(ui::Hue::Gray)
                                .trailing(move || {
                                    row((
                                        ui::badge("レビュー待ち").hue(ui::Hue::Yellow),
                                        kivact::ui::button("申請メニュー")
                                            .variant(kivact::ui::ButtonVariant::Ghost)
                                            .hide_label(true)
                                            .icon(kivact::control::icon("dots-vertical")),
                                    ))
                                    .style(style().gap(8))
                                }),
                            ui::list_item("救護所 シフト表")
                                .subtitle("更新：10/01 09:03 · ハルカ")
                                .icon("heart")
                                .tone(ui::Hue::Gray)
                                .trailing(move || {
                                    row((
                                        ui::badge("下書き").hue(ui::Hue::Gray),
                                        kivact::ui::button("シフトメニュー")
                                            .variant(kivact::ui::ButtonVariant::Ghost)
                                            .hide_label(true)
                                            .icon(kivact::control::icon("dots-vertical")),
                                    ))
                                    .style(style().gap(8))
                                }),
                        )),
                        column((
                            ui::key_value(
                                "依頼ID",
                                row((
                                    text("REQ-2026-1003").style(
                                        style()
                                            .background(ColorToken::SurfaceAlt)
                                            .font_family("monospace")
                                            .padding_x(4)
                                            .radius(4),
                                    ),
                                    kivact::ui::button("依頼IDをコピー")
                                        .variant(kivact::ui::ButtonVariant::Ghost)
                                        .hide_label(true)
                                        .icon(kivact::control::icon("copy"))
                                        .on_press(move || {
                                            spawn(kivact::platform::copy_text("REQ-2026-1003"));
                                            notify.call((
                                                ui::Hue::Blue,
                                                "コピーしました".into(),
                                                "REQ-2026-1003".into(),
                                            ));
                                        }),
                                )),
                            ),
                            ui::key_value(
                                "担当",
                                row((ui::avatar("M").size(24), text("空崎 ミナト")))
                                    .style(style().gap(8).align_items(Align::Center)),
                            ),
                            ui::key_value(
                                "ステータス",
                                ui::badge("進行中").variant(ui::BadgeVariant::Dot),
                            ),
                            ui::key_value("期限", text("2026/10/10 (土) 18:00")),
                            ui::key_value(
                                "エリア",
                                row((
                                    kivact::control::icon("map-pin"),
                                    text("中央区 外郭地区 第7ブロック"),
                                ))
                                .style(style().gap(6)),
                            ),
                        )),
                    ),
                    2,
                ),
            ))
            .style(style().grid_column(GridPlacement::span(12))),
        )),
    )
}

pub(super) fn table(sheet: Signal<bool>, drawer_title: Signal<String>) -> Node {
    let students = [
        (
            "空崎 ミナト",
            "アズール学園",
            "調査部",
            "ストライカー",
            86,
            20,
            "出撃中",
            205.0,
        ),
        (
            "春日 ハルカ",
            "セレスト女学院",
            "救護騎士団",
            "スペシャル",
            78,
            14,
            "待機",
            330.0,
        ),
        (
            "花岡 ユズ",
            "ミリオン工科高校",
            "ゲーム開発部",
            "ストライカー",
            72,
            11,
            "休息",
            45.0,
        ),
        (
            "白石 アリサ",
            "アズール学園",
            "風紀委員会",
            "ストライカー",
            90,
            25,
            "出撃中",
            160.0,
        ),
        (
            "月城 コハル",
            "セレスト女学院",
            "補習授業部",
            "スペシャル",
            54,
            6,
            "待機",
            265.0,
        ),
        (
            "朝凪 シオリ",
            "ノクス自治区",
            "図書委員会",
            "スペシャル",
            66,
            9,
            "待機",
            190.0,
        ),
        (
            "鷹宮 レイ",
            "ミリオン工科高校",
            "エンジニア部",
            "ストライカー",
            81,
            17,
            "出撃中",
            15.0,
        ),
        (
            "深山 ツバキ",
            "ノクス自治区",
            "修行部",
            "ストライカー",
            75,
            13,
            "休息",
            350.0,
        ),
        (
            "桜井 ノノカ",
            "アズール学園",
            "給食部",
            "スペシャル",
            48,
            22,
            "待機",
            120.0,
        ),
        (
            "氷室 カナデ",
            "セレスト女学院",
            "生徒会",
            "スペシャル",
            88,
            19,
            "出撃中",
            220.0,
        ),
        (
            "日向 イブキ",
            "ミリオン工科高校",
            "ヴェリタス",
            "ストライカー",
            69,
            8,
            "待機",
            30.0,
        ),
        (
            "黒羽 セナ",
            "ノクス自治区",
            "便利屋",
            "ストライカー",
            83,
            16,
            "出撃中",
            280.0,
        ),
        (
            "瀬川 ミオ",
            "アズール学園",
            "放送部",
            "スペシャル",
            41,
            4,
            "休息",
            175.0,
        ),
        (
            "真白 ユイ",
            "セレスト女学院",
            "ティーパーティー",
            "スペシャル",
            90,
            30,
            "待機",
            300.0,
        ),
        (
            "若葉 ナギ",
            "ミリオン工科高校",
            "セミナー",
            "ストライカー",
            62,
            10,
            "休息",
            95.0,
        ),
        (
            "九条 アカネ",
            "ノクス自治区",
            "百鬼夜行",
            "ストライカー",
            77,
            15,
            "出撃中",
            5.0,
        ),
        (
            "小鳥遊 ルカ",
            "アズール学園",
            "対策委員会",
            "ストライカー",
            85,
            21,
            "待機",
            210.0,
        ),
        (
            "御影 サヤ",
            "セレスト女学院",
            "シスターフッド",
            "スペシャル",
            58,
            7,
            "出撃中",
            250.0,
        ),
        (
            "天野 ヒナタ",
            "ミリオン工科高校",
            "トレーニング部",
            "ストライカー",
            71,
            12,
            "待機",
            55.0,
        ),
        (
            "水無瀬 リン",
            "ノクス自治区",
            "陰陽部",
            "スペシャル",
            64,
            9,
            "休息",
            185.0,
        ),
        (
            "藤咲 マコ",
            "アズール学園",
            "美食研究会",
            "ストライカー",
            52,
            5,
            "出撃中",
            340.0,
        ),
        (
            "霧島 アヤネ",
            "セレスト女学院",
            "正義実現委員会",
            "ストライカー",
            87,
            23,
            "待機",
            0.0,
        ),
        (
            "星野 ミユ",
            "ミリオン工科高校",
            "C&C",
            "ストライカー",
            80,
            18,
            "休息",
            235.0,
        ),
        (
            "東雲 キキョウ",
            "ノクス自治区",
            "忍術研究部",
            "スペシャル",
            60,
            8,
            "待機",
            150.0,
        ),
    ];
    let rows = signal(
        students
            .iter()
            .enumerate()
            .map(|(id, s)| Student {
                id,
                name: s.0,
                academy: s.1,
                club: s.2,
                role: s.3,
                level: s.4,
                bond: s.5,
                status: s.6,
                hue: s.7,
            })
            .collect::<Vec<_>>(),
    );
    let status = signal(Some("all"));
    let page = signal(0usize);
    let sort = signal(None::<(usize, kivact::interaction::SortOrder)>);
    let selected = signal(Vec::<usize>::new());
    let query = signal(String::new());
    let visible = move || {
        let expected = match status.get() {
            Some("deployed") => Some("出撃中"),
            Some("standby") => Some("待機"),
            Some("rest") => Some("休息"),
            _ => None,
        };
        rows.get()
            .into_iter()
            .filter(|s| expected.is_none_or(|expected| s.status == expected))
            .collect::<Vec<_>>()
    };
    let contains = |field: &str, query: &str| field.to_lowercase().contains(&query.to_lowercase());
    section(
        "06",
        "Data Table",
        "テーブル — 検索・並べ替え・選択・ページング",
        column((kivact::ui::data_table(
            visible,
            |s: &Student| s.id,
            vec![
                kivact::ui::DataColumn::new("生徒", |s: &Student| s.name.to_owned())
                    .width(228.0)
                    .compare(|a, b| a.name.cmp(b.name))
                    .filter(move |s, q| contains(s.name, q) || contains(s.club, q))
                    .cell(|s| {
                        row((
                            ui::avatar(
                                s.name
                                    .split(' ')
                                    .nth(1)
                                    .unwrap_or("?")
                                    .chars()
                                    .next()
                                    .unwrap_or('?')
                                    .to_string(),
                            )
                            .size(32)
                            .hue(s.hue),
                            column((
                                text(s.name).style(style().weight(500)),
                                text(s.club)
                                    .style(style().font_size(12).color(ColorToken::TextMuted)),
                            )),
                        ))
                        .style(style().gap(10).align_items(Align::Center))
                    }),
                kivact::ui::DataColumn::new("所属", |s: &Student| s.academy.to_owned())
                    .width(214.0)
                    .compare(|a, b| a.academy.cmp(b.academy))
                    .filter(move |s, q| contains(s.academy, q)),
                kivact::ui::DataColumn::new("役割", |s: &Student| s.role.to_owned())
                    .width(220.0)
                    .cell(|s| {
                        ui::badge(s.role).hue(if s.role == "ストライカー" {
                            ui::Hue::Red
                        } else {
                            ui::Hue::Blue
                        })
                    }),
                kivact::ui::DataColumn::new("Lv.", |s: &Student| s.level.to_string())
                    .width(70.0)
                    .compare(|a, b| a.level.cmp(&b.level))
                    .numeric(true),
                kivact::ui::DataColumn::new("絆", |s: &Student| s.bond.to_string())
                    .width(220.0)
                    .compare(|a, b| a.bond.cmp(&b.bond))
                    .cell(|s| {
                        row((
                            kivact::control::icon("heart").style(style().color(rgb(0xf2559a))),
                            text(s.bond.to_string()),
                            kivact::ui::progress(Some(f64::from(s.bond) / 30.0))
                                .name("絆")
                                .variant(ui::Hue::Pink)
                                .style(style().grow(1).height(6)),
                        ))
                        .style(style().gap(8).align_items(Align::Center).width(pct(100)))
                    }),
                kivact::ui::DataColumn::new("状態", |s: &Student| s.status.to_owned())
                    .width(142.0)
                    .cell(|s| {
                        ui::badge(s.status)
                            .hue(match s.status {
                                "出撃中" => ui::Hue::Blue,
                                "待機" => ui::Hue::Mint,
                                _ => ui::Hue::Gray,
                            })
                            .variant(ui::BadgeVariant::Dot)
                    }),
            ],
        )
        .row_actions(54.0, move |s: Value<Student>| {
            let s = s.get_untracked();
            let name = s.name.to_owned();
            kivact::ui::button("生徒詳細を開く")
                .variant(kivact::ui::ButtonVariant::Ghost)
                .hide_label(true)
                .icon(kivact::control::icon("dots-vertical"))
                .test_id(format!("pop-student-detail-{}", s.id))
                .on_press(move || {
                    drawer_title.set(name.clone());
                    sheet.set(true);
                })
        })
        .selection(selected)
        .sort(sort)
        .filter(query)
        .page(page, 6)
        .toolbar(move |context| {
            let filters = kivact::ui::radio_group(
                "状態で絞り込む",
                vec![
                    ListItem::new("all", "すべて"),
                    ListItem::new("deployed", "出撃中"),
                    ListItem::new("standby", "待機"),
                    ListItem::new("rest", "休息"),
                ],
                kivact::binding(status, move |next| {
                    status.set(next);
                    page.set(0);
                }),
            )
            .orientation(Orientation::Horizontal)
            .variant(ui::Segmented);
            let actions = row((
                kivact::ui::button("TSV")
                    .variant(kivact::ui::ButtonVariant::Secondary)
                    .size(kivact::ui::ControlSize::Small)
                    .icon(kivact::control::icon("download"))
                    .on_press(move || {
                        let lines = visible()
                            .into_iter()
                            .map(|s| {
                                [
                                    s.name.to_owned(),
                                    s.academy.to_owned(),
                                    s.role.to_owned(),
                                    s.level.to_string(),
                                    s.bond.to_string(),
                                    s.status.to_owned(),
                                    s.club.to_owned(),
                                ]
                                .join("	")
                            })
                            .collect::<Vec<_>>()
                            .join(
                                "
",
                            );
                        kivact::task::spawn(kivact::platform::copy_text(lines));
                    }),
                kivact::ui::button("生徒を追加")
                    .icon(kivact::control::icon("user-plus"))
                    .size(kivact::ui::ControlSize::Small)
                    .on_press(move || sheet.set(true)),
            ))
            .style(style().gap(8));
            let search = kivact::ui::text_input(
                kivact::i18n::message("pop.table.search", "Search table"),
                kivact::binding(query, move |text: String| {
                    query.set(text);
                    page.set(0);
                }),
            )
            .placeholder(kivact::i18n::message(
                "pop.table.search_placeholder",
                "Filter rows…",
            ))
            .style(style().width(260));
            let picked = context
                .selection
                .map(|selection| kivact::view::IntoValue::into_value(selection));
            let selected =
                kivact::i18n::message_with("pop.table.selected", "{count} selected", move || {
                    [(
                        "count",
                        picked.map_or(0, |picked| picked.with(Vec::len)).to_string(),
                    )]
                });
            let count = text(move || {
                if picked.is_none_or(|picked| picked.with(Vec::is_empty)) {
                    String::new()
                } else {
                    selected.get()
                }
            })
            .style(style().grow(1).font_size(13).color(ColorToken::TextMuted));
            row((search, filters, count, actions)).style(
                style()
                    .padding(16)
                    .gap(8)
                    .align_items(Align::Center)
                    .border_bottom(1, ColorToken::Border),
            )
        })
        .footer(|context| {
            let (page, size) = context.page.expect("the table pages");
            let (count, pages) = (context.model.row_count(), context.model.page_count());
            let current = move || page.get().min(pages.get().saturating_sub(1));
            let range = kivact::i18n::message_with(
                "pop.table.range",
                "Showing {from}–{to} of {total}",
                move || {
                    let (n, p, size) = (count.get(), current(), size.get().max(1));
                    [
                        ("from", (if n == 0 { 0 } else { p * size + 1 }).to_string()),
                        ("to", ((p + 1) * size).min(n).to_string()),
                        ("total", n.to_string()),
                    ]
                },
            );
            row((
                text(range).style(style().grow(1).font_size(13).color(ColorToken::TextMuted)),
                kivact::ui::button("‹")
                    .variant(kivact::ui::ButtonVariant::Ghost)
                    .disabled(move || current() == 0)
                    .on_press(move || page.set(current().saturating_sub(1))),
                kivact::view::for_index(
                    move || (0..if count.get() == 0 { 0 } else { pages.get() }).collect::<Vec<_>>(),
                    move |index, _| {
                        kivact::ui::button((index + 1).to_string())
                            .variant(kivact::ui::ButtonVariant::Ghost)
                            .selected(move || current() == index)
                            .style(
                                style().size(30, 30).selected(
                                    style()
                                        .background(ColorToken::Accent)
                                        .color(ColorToken::AccentText),
                                ),
                            )
                            .on_press(move || page.set(index))
                    },
                ),
                kivact::ui::button("›")
                    .variant(kivact::ui::ButtonVariant::Ghost)
                    .disabled(move || current() + 1 >= pages.get())
                    .on_press(move || page.set(current() + 1)),
            ))
            .style(
                style()
                    .padding_x(16)
                    .padding_y(12)
                    .gap(4)
                    .align_items(Align::Center),
            )
        }),)),
    )
}

/// One row of the student table.
#[derive(Clone, Copy, PartialEq)]
struct Student {
    id: usize,
    name: &'static str,
    academy: &'static str,
    club: &'static str,
    role: &'static str,
    level: u32,
    bond: u32,
    status: &'static str,
    hue: f64,
}
