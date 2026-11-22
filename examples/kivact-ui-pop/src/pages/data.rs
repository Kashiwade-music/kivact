//! Role: The table data: the student fixture, its columns, the toolbar and the footer of the table page.

use super::common::*;
use kivact::prelude::*;
use kivact::view::Node;
use kivact::view::Orientation;
use kivact_ui_pop as ui;
/// One row of the student table.
#[derive(Clone, Copy, PartialEq)]
pub(crate) struct Student {
    pub(crate) id: usize,
    pub(crate) name: &'static str,
    pub(crate) academy: &'static str,
    pub(crate) club: &'static str,
    pub(crate) role: &'static str,
    pub(crate) level: u32,
    pub(crate) bond: u32,
    pub(crate) status: &'static str,
    pub(crate) hue: f64,
}

pub(crate) const STUDENT_FIXTURE: [Student; 24] = [
    Student {
        id: 0,
        name: "Minato Sorasaki",
        academy: "Azure Academy",
        club: "Investigation Club",
        role: "Striker",
        level: 86,
        bond: 20,
        status: "Deployed",
        hue: 205.0,
    },
    Student {
        id: 1,
        name: "Haruka Kasuga",
        academy: "Celeste Girls' Academy",
        club: "Rescue Knights",
        role: "Special",
        level: 78,
        bond: 14,
        status: "Standby",
        hue: 330.0,
    },
    Student {
        id: 2,
        name: "Yuzu Hanaoka",
        academy: "Million Tech High",
        club: "Game Dev Club",
        role: "Striker",
        level: 72,
        bond: 11,
        status: "Resting",
        hue: 45.0,
    },
    Student {
        id: 3,
        name: "Arisa Shiraishi",
        academy: "Azure Academy",
        club: "Discipline Committee",
        role: "Striker",
        level: 90,
        bond: 25,
        status: "Deployed",
        hue: 160.0,
    },
    Student {
        id: 4,
        name: "Koharu Tsukishiro",
        academy: "Celeste Girls' Academy",
        club: "Remedial Class Club",
        role: "Special",
        level: 54,
        bond: 6,
        status: "Standby",
        hue: 265.0,
    },
    Student {
        id: 5,
        name: "Shiori Asanagi",
        academy: "Nox Autonomous Zone",
        club: "Library Committee",
        role: "Special",
        level: 66,
        bond: 9,
        status: "Standby",
        hue: 190.0,
    },
    Student {
        id: 6,
        name: "Rei Takamiya",
        academy: "Million Tech High",
        club: "Engineering Club",
        role: "Striker",
        level: 81,
        bond: 17,
        status: "Deployed",
        hue: 15.0,
    },
    Student {
        id: 7,
        name: "Tsubaki Miyama",
        academy: "Nox Autonomous Zone",
        club: "Training Club",
        role: "Striker",
        level: 75,
        bond: 13,
        status: "Resting",
        hue: 350.0,
    },
    Student {
        id: 8,
        name: "Nonoka Sakurai",
        academy: "Azure Academy",
        club: "Lunch Club",
        role: "Special",
        level: 48,
        bond: 22,
        status: "Standby",
        hue: 120.0,
    },
    Student {
        id: 9,
        name: "Kanade Himuro",
        academy: "Celeste Girls' Academy",
        club: "Student Council",
        role: "Special",
        level: 88,
        bond: 19,
        status: "Deployed",
        hue: 220.0,
    },
    Student {
        id: 10,
        name: "Ibuki Hinata",
        academy: "Million Tech High",
        club: "Veritas",
        role: "Striker",
        level: 69,
        bond: 8,
        status: "Standby",
        hue: 30.0,
    },
    Student {
        id: 11,
        name: "Sena Kurobane",
        academy: "Nox Autonomous Zone",
        club: "Odd Jobs",
        role: "Striker",
        level: 83,
        bond: 16,
        status: "Deployed",
        hue: 280.0,
    },
    Student {
        id: 12,
        name: "Mio Segawa",
        academy: "Azure Academy",
        club: "Broadcast Club",
        role: "Special",
        level: 41,
        bond: 4,
        status: "Resting",
        hue: 175.0,
    },
    Student {
        id: 13,
        name: "Yui Mashiro",
        academy: "Celeste Girls' Academy",
        club: "Tea Party",
        role: "Special",
        level: 90,
        bond: 30,
        status: "Standby",
        hue: 300.0,
    },
    Student {
        id: 14,
        name: "Nagi Wakaba",
        academy: "Million Tech High",
        club: "Seminar",
        role: "Striker",
        level: 62,
        bond: 10,
        status: "Resting",
        hue: 95.0,
    },
    Student {
        id: 15,
        name: "Akane Kujo",
        academy: "Nox Autonomous Zone",
        club: "Night Parade",
        role: "Striker",
        level: 77,
        bond: 15,
        status: "Deployed",
        hue: 5.0,
    },
    Student {
        id: 16,
        name: "Ruka Takanashi",
        academy: "Azure Academy",
        club: "Countermeasures Committee",
        role: "Striker",
        level: 85,
        bond: 21,
        status: "Standby",
        hue: 210.0,
    },
    Student {
        id: 17,
        name: "Saya Mikage",
        academy: "Celeste Girls' Academy",
        club: "Sisterhood",
        role: "Special",
        level: 58,
        bond: 7,
        status: "Deployed",
        hue: 250.0,
    },
    Student {
        id: 18,
        name: "Hinata Amano",
        academy: "Million Tech High",
        club: "Training Squad",
        role: "Striker",
        level: 71,
        bond: 12,
        status: "Standby",
        hue: 55.0,
    },
    Student {
        id: 19,
        name: "Rin Minase",
        academy: "Nox Autonomous Zone",
        club: "Onmyo Club",
        role: "Special",
        level: 64,
        bond: 9,
        status: "Resting",
        hue: 185.0,
    },
    Student {
        id: 20,
        name: "Mako Fujisaki",
        academy: "Azure Academy",
        club: "Gourmet Society",
        role: "Striker",
        level: 52,
        bond: 5,
        status: "Deployed",
        hue: 340.0,
    },
    Student {
        id: 21,
        name: "Ayane Kirishima",
        academy: "Celeste Girls' Academy",
        club: "Justice Committee",
        role: "Striker",
        level: 87,
        bond: 23,
        status: "Standby",
        hue: 0.0,
    },
    Student {
        id: 22,
        name: "Miyu Hoshino",
        academy: "Million Tech High",
        club: "C&C",
        role: "Striker",
        level: 80,
        bond: 18,
        status: "Resting",
        hue: 235.0,
    },
    Student {
        id: 23,
        name: "Kikyo Shinonome",
        academy: "Nox Autonomous Zone",
        club: "Ninjutsu Club",
        role: "Special",
        level: 60,
        bond: 8,
        status: "Standby",
        hue: 150.0,
    },
];

pub(crate) fn contains(field: &str, query: &str) -> bool {
    field.to_lowercase().contains(&query.to_lowercase())
}

pub(crate) fn student_status_column() -> kivact::ui::DataColumn<Student> {
    kivact::ui::DataColumn::new("Status", |s: &Student| s.status.to_owned())
        .width(142.0)
        .cell(|s| {
            kivact::ui::badge(s.status)
                .variant(match s.status {
                    "Deployed" => ui::Hue::Blue,
                    "Standby" => ui::Hue::Mint,
                    _ => ui::Hue::Gray,
                })
                .variant(ui::BadgeVariant::Dot)
        })
}

pub(crate) fn student_bond_column() -> kivact::ui::DataColumn<Student> {
    kivact::ui::DataColumn::new("Bond", |s: &Student| s.bond.to_string())
        .width(220.0)
        .compare(|a, b| a.bond.cmp(&b.bond))
        .cell(|s| {
            row((
                kivact::view::icon("heart").style(style().color(rgb(0xf2559a))),
                text(s.bond.to_string()),
                kivact::ui::progress(Some(f64::from(s.bond) / 30.0))
                    .name("Bond")
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
    kivact::ui::DataColumn::new("Role", |s: &Student| s.role.to_owned())
        .width(220.0)
        .cell(|s| {
            kivact::ui::badge(s.role).variant(if s.role == "Striker" {
                ui::Hue::Red
            } else {
                ui::Hue::Blue
            })
        })
}

pub(crate) fn student_academy_column() -> kivact::ui::DataColumn<Student> {
    kivact::ui::DataColumn::new("Academy", |s: &Student| s.academy.to_owned())
        .width(214.0)
        .compare(|a, b| a.academy.cmp(b.academy))
        .filter(move |s, q| contains(s.academy, q))
}

pub(crate) fn student_name_column() -> kivact::ui::DataColumn<Student> {
    kivact::ui::DataColumn::new("Student", |s: &Student| s.name.to_owned())
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
        "Filter by status",
        vec![
            ListItem::new("all", "All"),
            ListItem::new("deployed", "Deployed"),
            ListItem::new("standby", "Standby"),
            ListItem::new("rest", "Resting"),
        ],
        kivact::state::binding(status, move |next| {
            status.set(next);
            page.set(0);
        }),
    )
    .orientation(Orientation::Horizontal)
    .variant(kivact::ui::RadioGroupVariant::Segmented);
    let actions = student_actions(visible, sheet).style(style().gap(8));
    let search = kivact::ui::text_input(kivact::state::binding(query, move |text: String| {
        query.set(text);
        page.set(0);
    }))
    .label(kivact::i18n::message("pop.table.search", "Search table"))
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
                        style()
                            .size(30, 30)
                            .selected(style().background(color::ACCENT).color(color::ACCENT_TEXT)),
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
        kivact::ui::button("Add student")
            .icon(kivact::view::icon("user-plus"))
            .size(kivact::ui::ControlSize::Small)
            .on_press(move || sheet.set(true)),
    ))
}
