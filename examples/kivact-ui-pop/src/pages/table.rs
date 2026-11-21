//! Role: The data table page: a searchable, sortable and paged table of students with a row drawer.

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
use super::data::*;
use catalog_shell::{Example, Page};
use super::common::*;
/// The page of the data table.
pub fn page() -> Page {
    Page::new("table", "Table", "Components", include_str!("table.rs"))
        .description("A data table with search, sorting, selection and paging.")
        .example(Example::new(
            "table",
            "Student table",
            "Press a row's button to open its detail drawer.",
            table_demo,
        ))
}

// xtask-allow(state): independent: sort is written by the column headers, whose binding resets page to 0 as the status filter and the search do; page is then moved by the footer pager, so any sort pairs with any page; sheet is the student drawer's binding, opened by the row press and closed by the drawer
fn table_demo() -> impl IntoView {
    // example: table
    let sheet = signal(false);
    let drawer_title = signal(String::new());
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
    // end example
}


// xtask-allow(state): independent: pin_student and auto_skill back the drawer's two switches (pin to main team, auto skill), each written only by its own switch
fn student_drawer(
    drawer_title: Signal<String>,
    sheet: Signal<bool>,
    notify: kivact::state::Callback<(ui::Hue, String, String)>,
) -> kivact::ui::Dialog {
    let pin_student = signal(true);
    let auto_skill = signal(false);
    kivact::ui::dialog(drawer_title, sheet)
        .mode(kivact::interaction::DialogMode::Drawer(
            kivact::geometry::Edge::Right,
        ))
        .content(move || {
            column((
                student_identity(),
                kivact::ui::description_list((
                    kivact::ui::description("レベル", text("Lv.86")),
                    kivact::ui::description("絆ランク", text("Rank 20")),
                    kivact::ui::description(
                        "攻撃タイプ",
                        kivact::ui::badge("爆発").variant(ui::Hue::Red),
                    ),
                    kivact::ui::description(
                        "防御タイプ",
                        kivact::ui::badge("重装甲").variant(ui::Hue::Yellow),
                    ),
                    text("個別設定").style(style().font_size(15).font_weight(600).margin_top(8)),
                ))
                .style(style().gap(12)),
                row((
                    text("メイン編成に固定").style(style().flex_grow(1)),
                    kivact::ui::switch("メイン編成に固定", pin_student)
                        .hide_label(true)
                        .style(style().width(32).padding_left(0)),
                ))
                .style(style().align_items(Align::Center)),
                row((
                    text("スキル自動使用").style(style().flex_grow(1)),
                    kivact::ui::switch("スキル自動使用", auto_skill)
                        .hide_label(true)
                        .style(style().width(32).padding_left(0)),
                ))
                .style(style().align_items(Align::Center)),
            ))
            .style(style().gap(16))
        })
        .footer(move || {
            row((
                kivact::ui::button("閉じる")
                    .variant(kivact::ui::ButtonVariant::Secondary)
                    .on_press(move || sheet.set(false)),
                kivact::ui::button("保存")
                    .test_id("pop-drawer-save")
                    .on_press(move || {
                        sheet.set(false);
                        notify.call((
                            ui::Hue::Mint,
                            "保存しました".into(),
                            "個別設定を保存しました。".into(),
                        ));
                    }),
            ))
            .style(style().gap(8))
        })
}

fn student_identity() -> Node {
    row((
        decorated_avatar(
            kivact::state::IntoText::into_text_value("MN"),
            AvatarLook::default().size(72).halo(true),
        ),
        column((
            text("アズール学園 · 調査部").style(style().font_weight(600)),
            text("★★★").style(style().color(rgb(0xf2bf0a))),
            wrap((
                kivact::ui::badge("ストライカー"),
                kivact::ui::badge("後衛").variant(ui::Hue::Gray),
            )),
        ))
        .style(style().gap(6)),
    ))
    .style(
        style()
            .padding(16)
            .padding_top(24)
            .gap(18)
            .align_items(Align::Center)
            .border_radius(10)
            .border(1, color::BORDER)
            .background(color::SURFACE_ALT),
    )
}
