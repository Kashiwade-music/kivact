//! Role: The table data: the student fixture, its columns, the toolbar and the footer of the table page.

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
/// One row of the student table.
#[derive(Clone, Copy, PartialEq)]
pub(crate) struct Student {
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

pub(crate) const STUDENT_FIXTURE: [Student; 24] = [
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

pub(crate) fn contains(field: &str, query: &str) -> bool {
    field.to_lowercase().contains(&query.to_lowercase())
}

pub(crate) fn student_status_column() -> kivact::ui::DataColumn<Student> {
    kivact::ui::DataColumn::new("状態", |s: &Student| s.status.to_owned())
        .width(142.0)
        .cell(|s| {
            kivact::ui::badge(s.status)
                .variant(ui::Hue::from_badge(match s.status {
                    "出撃中" => ui::Hue::Blue,
                    "待機" => ui::Hue::Mint,
                    _ => ui::Hue::Gray,
                }))
                .variant(ui::BadgeVariant::Dot)
        })
}

pub(crate) fn student_bond_column() -> kivact::ui::DataColumn<Student> {
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
                    .style(style().flex_grow(1).height(6)),
            ))
            .style(style().gap(8).align_items(Align::Center).width(pct(100)))
        })
}

pub(crate) fn student_level_column() -> kivact::ui::DataColumn<Student> {
    kivact::ui::DataColumn::new("Lv.", |s: &Student| s.level.to_string())
        .width(70.0)
        .compare(|a, b| a.level.cmp(&b.level))
        .numeric(true)
}

pub(crate) fn student_role_column() -> kivact::ui::DataColumn<Student> {
    kivact::ui::DataColumn::new("役割", |s: &Student| s.role.to_owned())
        .width(220.0)
        .cell(|s| {
            kivact::ui::badge(s.role).variant(ui::Hue::from_badge(if s.role == "ストライカー" {
                ui::Hue::Red
            } else {
                ui::Hue::Blue
            }))
        })
}

pub(crate) fn student_academy_column() -> kivact::ui::DataColumn<Student> {
    kivact::ui::DataColumn::new("所属", |s: &Student| s.academy.to_owned())
        .width(214.0)
        .compare(|a, b| a.academy.cmp(b.academy))
        .filter(move |s, q| contains(s.academy, q))
}

pub(crate) fn student_name_column() -> kivact::ui::DataColumn<Student> {
    kivact::ui::DataColumn::new("生徒", |s: &Student| s.name.to_owned())
        .width(228.0)
        .compare(|a, b| a.name.cmp(b.name))
        .filter(move |s, q| contains(s.name, q) || contains(s.club, q))
        .cell(|s| {
            row((
                decorated_avatar(
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
                    AvatarLook::default().size(32).hue(s.hue),
                ),
                column((
                    text(s.name).style(style().font_weight(500)),
                    text(s.club).style(style().font_size(12).color(color::TEXT_MUTED)),
                )),
            ))
            .style(style().gap(10).align_items(Align::Center))
        })
}

pub(crate) fn student_toolbar(
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
    .style(style().flex_grow(1).font_size(13).color(color::TEXT_MUTED));
    row((search, filters, count, actions)).style(
        style()
            .padding(16)
            .gap(8)
            .align_items(Align::Center)
            .border_bottom(1, color::BORDER),
    )
}

pub(crate) fn student_footer(context: kivact::ui::DataTableContext<Student, usize>) -> Node {
    let kivact::ui::Paging::Local {
        page,
        page_size: size,
    } = context.paging
    else {
        panic!("the table pages");
    };
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
        text(range).style(style().flex_grow(1).font_size(13).color(color::TEXT_MUTED)),
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
                                .background(color::ACCENT)
                                .color(color::ACCENTText),
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

pub(crate) fn student_actions(
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
