//! Role: Source catalogue sections built from the public component library.
use kivact::state::IntoValue;

use super::common::{card, cards_grid, columns, muted, section, third, wrap};
use kivact::interaction::ListItem;
use kivact::prelude::{ColorToken, Element, Node, Signal, column, node, rgb, row, signal, text};
use kivact::state::{Callback, Value};
use kivact::style::Direction;
use kivact::style::GridPlacement;
use kivact::style::{Align, Justify, pct, style};
use kivact::task::spawn;
use kivact::view::Orientation;
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
        let badge = kivact::ui::badge(s).variant(ui::BadgeHue::new(hue));
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
            third(badge_examples(badges, chip_tags, next_chip)),
            third(avatar_examples()),
            third(progress_examples(live)),
            request_summary(notify).style(style().grid_column(GridPlacement::span(12))),
        )),
    )
}

pub(super) fn table(sheet: Signal<bool>, drawer_title: Signal<String>) -> Node {
    let rows = signal(STUDENT_FIXTURE.to_vec());
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
    section(
        "06",
        "Data Table",
        "テーブル — 検索・並べ替え・選択・ページング",
        column((kivact::ui::data_table(
            visible,
            |s: &Student| s.id,
            vec![
                student_name_column(),
                student_academy_column(),
                student_role_column(),
                student_level_column(),
                student_bond_column(),
                student_status_column(),
            ],
        )
        .row_actions(54.0, move |s: Value<Student>| {
            let s = s.get_untracked();
            let name = s.name.to_owned();
            kivact::ui::button("生徒詳細を開く")
                .variant(kivact::ui::ButtonVariant::Ghost)
                .hide_label(true)
                .icon(kivact::view::icon("dots-vertical"))
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
        .toolbar(move |context| student_toolbar(context, status, page, query, sheet, visible))
        .footer(student_footer),)),
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

const STUDENT_FIXTURE: [Student; 24] = [
    Student {
        id: 0,
        name: "空崎 ミナト",
        academy: "アズール学園",
        club: "調査部",
        role: "ストライカー",
        level: 86,
        bond: 20,
        status: "出撃中",
        hue: 205.0,
    },
    Student {
        id: 1,
        name: "春日 ハルカ",
        academy: "セレスト女学院",
        club: "救護騎士団",
        role: "スペシャル",
        level: 78,
        bond: 14,
        status: "待機",
        hue: 330.0,
    },
    Student {
        id: 2,
        name: "花岡 ユズ",
        academy: "ミリオン工科高校",
        club: "ゲーム開発部",
        role: "ストライカー",
        level: 72,
        bond: 11,
        status: "休息",
        hue: 45.0,
    },
    Student {
        id: 3,
        name: "白石 アリサ",
        academy: "アズール学園",
        club: "風紀委員会",
        role: "ストライカー",
        level: 90,
        bond: 25,
        status: "出撃中",
        hue: 160.0,
    },
    Student {
        id: 4,
        name: "月城 コハル",
        academy: "セレスト女学院",
        club: "補習授業部",
        role: "スペシャル",
        level: 54,
        bond: 6,
        status: "待機",
        hue: 265.0,
    },
    Student {
        id: 5,
        name: "朝凪 シオリ",
        academy: "ノクス自治区",
        club: "図書委員会",
        role: "スペシャル",
        level: 66,
        bond: 9,
        status: "待機",
        hue: 190.0,
    },
    Student {
        id: 6,
        name: "鷹宮 レイ",
        academy: "ミリオン工科高校",
        club: "エンジニア部",
        role: "ストライカー",
        level: 81,
        bond: 17,
        status: "出撃中",
        hue: 15.0,
    },
    Student {
        id: 7,
        name: "深山 ツバキ",
        academy: "ノクス自治区",
        club: "修行部",
        role: "ストライカー",
        level: 75,
        bond: 13,
        status: "休息",
        hue: 350.0,
    },
    Student {
        id: 8,
        name: "桜井 ノノカ",
        academy: "アズール学園",
        club: "給食部",
        role: "スペシャル",
        level: 48,
        bond: 22,
        status: "待機",
        hue: 120.0,
    },
    Student {
        id: 9,
        name: "氷室 カナデ",
        academy: "セレスト女学院",
        club: "生徒会",
        role: "スペシャル",
        level: 88,
        bond: 19,
        status: "出撃中",
        hue: 220.0,
    },
    Student {
        id: 10,
        name: "日向 イブキ",
        academy: "ミリオン工科高校",
        club: "ヴェリタス",
        role: "ストライカー",
        level: 69,
        bond: 8,
        status: "待機",
        hue: 30.0,
    },
    Student {
        id: 11,
        name: "黒羽 セナ",
        academy: "ノクス自治区",
        club: "便利屋",
        role: "ストライカー",
        level: 83,
        bond: 16,
        status: "出撃中",
        hue: 280.0,
    },
    Student {
        id: 12,
        name: "瀬川 ミオ",
        academy: "アズール学園",
        club: "放送部",
        role: "スペシャル",
        level: 41,
        bond: 4,
        status: "休息",
        hue: 175.0,
    },
    Student {
        id: 13,
        name: "真白 ユイ",
        academy: "セレスト女学院",
        club: "ティーパーティー",
        role: "スペシャル",
        level: 90,
        bond: 30,
        status: "待機",
        hue: 300.0,
    },
    Student {
        id: 14,
        name: "若葉 ナギ",
        academy: "ミリオン工科高校",
        club: "セミナー",
        role: "ストライカー",
        level: 62,
        bond: 10,
        status: "休息",
        hue: 95.0,
    },
    Student {
        id: 15,
        name: "九条 アカネ",
        academy: "ノクス自治区",
        club: "百鬼夜行",
        role: "ストライカー",
        level: 77,
        bond: 15,
        status: "出撃中",
        hue: 5.0,
    },
    Student {
        id: 16,
        name: "小鳥遊 ルカ",
        academy: "アズール学園",
        club: "対策委員会",
        role: "ストライカー",
        level: 85,
        bond: 21,
        status: "待機",
        hue: 210.0,
    },
    Student {
        id: 17,
        name: "御影 サヤ",
        academy: "セレスト女学院",
        club: "シスターフッド",
        role: "スペシャル",
        level: 58,
        bond: 7,
        status: "出撃中",
        hue: 250.0,
    },
    Student {
        id: 18,
        name: "天野 ヒナタ",
        academy: "ミリオン工科高校",
        club: "トレーニング部",
        role: "ストライカー",
        level: 71,
        bond: 12,
        status: "待機",
        hue: 55.0,
    },
    Student {
        id: 19,
        name: "水無瀬 リン",
        academy: "ノクス自治区",
        club: "陰陽部",
        role: "スペシャル",
        level: 64,
        bond: 9,
        status: "休息",
        hue: 185.0,
    },
    Student {
        id: 20,
        name: "藤咲 マコ",
        academy: "アズール学園",
        club: "美食研究会",
        role: "ストライカー",
        level: 52,
        bond: 5,
        status: "出撃中",
        hue: 340.0,
    },
    Student {
        id: 21,
        name: "霧島 アヤネ",
        academy: "セレスト女学院",
        club: "正義実現委員会",
        role: "ストライカー",
        level: 87,
        bond: 23,
        status: "待機",
        hue: 0.0,
    },
    Student {
        id: 22,
        name: "星野 ミユ",
        academy: "ミリオン工科高校",
        club: "C&C",
        role: "ストライカー",
        level: 80,
        bond: 18,
        status: "休息",
        hue: 235.0,
    },
    Student {
        id: 23,
        name: "東雲 キキョウ",
        academy: "ノクス自治区",
        club: "忍術研究部",
        role: "スペシャル",
        level: 60,
        bond: 8,
        status: "待機",
        hue: 150.0,
    },
];

fn contains(field: &str, query: &str) -> bool {
    field.to_lowercase().contains(&query.to_lowercase())
}

fn student_status_column() -> kivact::ui::DataColumn<Student> {
    kivact::ui::DataColumn::new("状態", |s: &Student| s.status.to_owned())
        .width(142.0)
        .cell(|s| {
            kivact::ui::badge(s.status)
                .variant(ui::BadgeHue::new(match s.status {
                    "出撃中" => ui::Hue::Blue,
                    "待機" => ui::Hue::Mint,
                    _ => ui::Hue::Gray,
                }))
                .variant(ui::BadgeVariant::Dot)
        })
}

fn student_bond_column() -> kivact::ui::DataColumn<Student> {
    kivact::ui::DataColumn::new("絆", |s: &Student| s.bond.to_string())
        .width(220.0)
        .compare(|a, b| a.bond.cmp(&b.bond))
        .cell(|s| {
            row((
                kivact::view::icon("heart").style(style().color(rgb(0xf2559a))),
                text(s.bond.to_string()),
                kivact::ui::progress(Some(f64::from(s.bond) / 30.0))
                    .name("絆")
                    .variant(ui::Hue::Pink)
                    .style(style().grow(1).height(6)),
            ))
            .style(style().gap(8).align_items(Align::Center).width(pct(100)))
        })
}

fn student_level_column() -> kivact::ui::DataColumn<Student> {
    kivact::ui::DataColumn::new("Lv.", |s: &Student| s.level.to_string())
        .width(70.0)
        .compare(|a, b| a.level.cmp(&b.level))
        .numeric(true)
}

fn student_role_column() -> kivact::ui::DataColumn<Student> {
    kivact::ui::DataColumn::new("役割", |s: &Student| s.role.to_owned())
        .width(220.0)
        .cell(|s| {
            kivact::ui::badge(s.role).variant(ui::BadgeHue::new(if s.role == "ストライカー" {
                ui::Hue::Red
            } else {
                ui::Hue::Blue
            }))
        })
}

fn student_academy_column() -> kivact::ui::DataColumn<Student> {
    kivact::ui::DataColumn::new("所属", |s: &Student| s.academy.to_owned())
        .width(214.0)
        .compare(|a, b| a.academy.cmp(b.academy))
        .filter(move |s, q| contains(s.academy, q))
}

fn student_name_column() -> kivact::ui::DataColumn<Student> {
    kivact::ui::DataColumn::new("生徒", |s: &Student| s.name.to_owned())
        .width(228.0)
        .compare(|a, b| a.name.cmp(b.name))
        .filter(move |s, q| contains(s.name, q) || contains(s.club, q))
        .cell(|s| {
            row((
                super::common::decorated_avatar(
                    kivact::state::IntoText::into_text_value(
                        s.name
                            .split(' ')
                            .nth(1)
                            .unwrap_or("?")
                            .chars()
                            .next()
                            .unwrap_or('?')
                            .to_string(),
                    ),
                    ui::AvatarLook::default().size(32).hue(s.hue),
                ),
                column((
                    text(s.name).style(style().weight(500)),
                    text(s.club).style(style().font_size(12).color(ColorToken::TextMuted)),
                )),
            ))
            .style(style().gap(10).align_items(Align::Center))
        })
}

fn student_toolbar(
    context: kivact::ui::DataTableContext<Student, usize>,
    status: Signal<Option<&'static str>>,
    page: Signal<usize>,
    query: Signal<String>,
    sheet: Signal<bool>,
    visible: impl Fn() -> Vec<Student> + Copy + 'static,
) -> Node {
    let filters = kivact::ui::radio_group(
        "状態で絞り込む",
        vec![
            ListItem::new("all", "すべて"),
            ListItem::new("deployed", "出撃中"),
            ListItem::new("standby", "待機"),
            ListItem::new("rest", "休息"),
        ],
        kivact::state::binding(status, move |next| {
            status.set(next);
            page.set(0);
        }),
    )
    .orientation(Orientation::Horizontal)
    .variant(kivact::ui::RadioGroupVariant::Segmented);
    let actions = student_actions(visible, sheet).style(style().gap(8));
    let search = kivact::ui::text_input(
        kivact::i18n::message("pop.table.search", "Search table"),
        kivact::state::binding(query, move |text: String| {
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
        .map(|selection| kivact::state::IntoValue::into_value(selection));
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
}

fn student_footer(context: kivact::ui::DataTableContext<Student, usize>) -> Node {
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
                    .with_node(|node| node.selected(move || current() == index))
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
}

fn progress_ring(percent: f64, label: &'static str, hue: ui::Hue) -> Node {
    let percent: kivact::state::Value<f64> = percent.into_value();
    let label = kivact::state::IntoText::into_text_value(label);
    column((
        kivact::ui::progress(move || Some(percent.get() / 100.0))
            .name(label)
            .variant(kivact::ui::ProgressVariant::Ring)
            .variant(ui::ProgressHue(hue.into_value())),
        text(label).style(style().font_size(12).color(ColorToken::TextMuted)),
    ))
    .style(style().align_items(Align::Center).gap(6))
}

fn badge_examples(
    badges: impl kivact::view::IntoView,
    chip_tags: Signal<Vec<String>>,
    next_chip: Signal<usize>,
) -> kivact::ui::Card {
    card(
        "バッジ・チップ",
        column((
            wrap(badges),
            wrap((
                kivact::ui::badge("オンライン")
                    .variant(ui::BadgeHue::new(ui::Hue::Mint))
                    .variant(ui::BadgeVariant::Dot),
                kivact::ui::badge("離席中")
                    .variant(ui::BadgeHue::new(ui::Hue::Yellow))
                    .variant(ui::BadgeVariant::Dot),
                kivact::ui::badge("オフライン")
                    .variant(ui::BadgeHue::new(ui::Hue::Gray))
                    .variant(ui::BadgeVariant::Dot),
                kivact::ui::badge("Lv.90").variant(ui::BadgeVariant::Solid),
                kivact::ui::badge("EX").variant(ui::BadgeVariant::Slant),
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
                    chip_tags.update(|items| items.push(options[index % options.len()].into()));
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
    )
}

fn avatar_examples() -> kivact::ui::Card {
    card(
        "アバター・ヘイロー",
        column((
            wrap((
                super::common::decorated_avatar(
                    kivact::state::IntoText::into_text_value("MN"),
                    ui::AvatarLook::default().size(72).halo(true),
                ),
                super::common::decorated_avatar(
                    kivact::state::IntoText::into_text_value("HK"),
                    ui::AvatarLook::default().size(56).hue(330.0).halo(true),
                ),
                super::common::decorated_avatar(
                    kivact::state::IntoText::into_text_value("YZ"),
                    ui::AvatarLook::default().hue(45.0).halo(true),
                ),
                super::common::decorated_avatar(
                    kivact::state::IntoText::into_text_value("AR"),
                    ui::AvatarLook::default().size(32).hue(160.0),
                ),
                super::common::decorated_avatar(
                    kivact::state::IntoText::into_text_value("K"),
                    ui::AvatarLook::default().size(24).hue(265.0),
                ),
            ))
            .style(style().align_items(Align::End)),
            avatar_profile(),
            row((
                super::common::decorated_avatar(
                    kivact::state::IntoText::into_text_value("HK"),
                    ui::AvatarLook::default().hue(330.0).halo(true).online(true),
                ),
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
    )
}

fn progress_examples(live: Signal<f64>) -> kivact::ui::Card {
    card(
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
                    progress_ring(72.0, "出席率", ui::Hue::Blue),
                    progress_ring(45.0, "予算消化", ui::Hue::Yellow),
                    progress_ring(91.0, "達成率", ui::Hue::Mint),
                ),
                3,
            ),
        ))
        .style(style().gap(16)),
    )
}

fn request_summary(notify: Callback<(ui::Hue, String, String)>) -> kivact::ui::Card {
    kivact::ui::card((
        row((
            text("リスト・キーバリュー").style(style().font_size(15).weight(600).grow(1)),
            kivact::ui::button("すべて表示")
                .size(kivact::ui::ControlSize::Small)
                .variant(kivact::ui::ButtonVariant::Ghost)
                .trailing(kivact::view::icon("chevron-right")),
        ))
        .style(style().align_items(Align::Center).margin_bottom(16)),
        columns((document_rows(), request_metadata(notify)), 2),
    ))
}

fn student_actions(
    visible: impl Fn() -> Vec<Student> + Copy + 'static,
    sheet: Signal<bool>,
) -> Node {
    row((
        kivact::ui::button("TSV")
            .variant(kivact::ui::ButtonVariant::Secondary)
            .size(kivact::ui::ControlSize::Small)
            .icon(kivact::view::icon("download"))
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
            .icon(kivact::view::icon("user-plus"))
            .size(kivact::ui::ControlSize::Small)
            .on_press(move || sheet.set(true)),
    ))
}

fn avatar_profile() -> Node {
    row((
        super::common::decorated_avatar(
            kivact::state::IntoText::into_text_value("MN"),
            ui::AvatarLook::default().size(32),
        ),
        super::common::decorated_avatar(
            kivact::state::IntoText::into_text_value("HK"),
            ui::AvatarLook::default().size(32).hue(330.0),
        )
        .style(style().margin_left(-8)),
        super::common::decorated_avatar(
            kivact::state::IntoText::into_text_value("YZ"),
            ui::AvatarLook::default().size(32).hue(45.0),
        )
        .style(style().margin_left(-8)),
        super::common::decorated_avatar(
            kivact::state::IntoText::into_text_value("AR"),
            ui::AvatarLook::default().size(32).hue(160.0),
        )
        .style(style().margin_left(-8)),
        node()
            .child(text("+8").style(style().font_size(11).color(ColorToken::TextMuted)))
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
    .style(style().align_items(Align::Center))
}

fn document_rows() -> kivact::ui::ItemList {
    kivact::ui::item_list((
        kivact::ui::item_row("第3章 作戦報告書")
            .description("更新：10/02 22:14 · ミナト")
            .media(kivact::view::icon("file-text"))
            .variant(ui::ItemHue::new(ui::Hue::Blue))
            .actions({
                row((
                    kivact::ui::badge("承認済")
                        .variant(ui::BadgeHue::new(ui::Hue::Mint.into_value())),
                    kivact::ui::button("報告書メニュー")
                        .variant(kivact::ui::ButtonVariant::Ghost)
                        .hide_label(true)
                        .icon(kivact::view::icon("dots-vertical")),
                ))
                .style(style().gap(8))
            }),
        kivact::ui::item_row("弾薬補給 予算申請")
            .description("更新：10/02 18:40 · ユズ")
            .media(kivact::view::icon("coin"))
            .variant(ui::ItemHue::new(ui::Hue::Gray))
            .actions({
                row((
                    kivact::ui::badge("レビュー待ち")
                        .variant(ui::BadgeHue::new(ui::Hue::Yellow.into_value())),
                    kivact::ui::button("申請メニュー")
                        .variant(kivact::ui::ButtonVariant::Ghost)
                        .hide_label(true)
                        .icon(kivact::view::icon("dots-vertical")),
                ))
                .style(style().gap(8))
            }),
        kivact::ui::item_row("救護所 シフト表")
            .description("更新：10/01 09:03 · ハルカ")
            .media(kivact::view::icon("heart"))
            .variant(ui::ItemHue::new(ui::Hue::Gray))
            .actions({
                row((
                    kivact::ui::badge("下書き").variant(ui::BadgeHue::new(ui::Hue::Gray)),
                    kivact::ui::button("シフトメニュー")
                        .variant(kivact::ui::ButtonVariant::Ghost)
                        .hide_label(true)
                        .icon(kivact::view::icon("dots-vertical")),
                ))
                .style(style().gap(8))
            }),
    ))
}

fn request_metadata(notify: Callback<(ui::Hue, String, String)>) -> kivact::ui::DescriptionList {
    kivact::ui::description_list((
        kivact::ui::description(
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
                    .icon(kivact::view::icon("copy"))
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
        kivact::ui::description(
            "担当",
            row((
                super::common::decorated_avatar(
                    kivact::state::IntoText::into_text_value("M"),
                    ui::AvatarLook::default().size(24),
                ),
                text("空崎 ミナト"),
            ))
            .style(style().gap(8).align_items(Align::Center)),
        ),
        kivact::ui::description(
            "ステータス",
            kivact::ui::badge("進行中").variant(ui::BadgeVariant::Dot),
        ),
        kivact::ui::description("期限", text("2026/10/10 (土) 18:00")),
        kivact::ui::description(
            "エリア",
            row((
                kivact::view::icon("map-pin"),
                text("中央区 外郭地区 第7ブロック"),
            ))
            .style(style().gap(6)),
        ),
    ))
}
