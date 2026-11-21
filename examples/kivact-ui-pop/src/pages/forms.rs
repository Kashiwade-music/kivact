//! Role: The forms page: text inputs, selection editors, checks, switches, sliders and uploads.

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
/// The page of the form controls.
pub fn page() -> Page {
    Page::new("forms", "Forms", "Components", include_str!("forms.rs"))
        .description("Inputs, selections, checks, switches, sliders and uploads with their labels, hints and errors.")
        .example(Example::new(
            "forms",
            "Form controls",
            "Each control is named by its label and reports its state.",
            forms_demo,
        ))
}

#[derive(Clone, PartialEq)]
struct UploadEntry {
    id: u64,
    name: String,
    bytes: Option<u64>,
    progress: f64,
}

fn forms_demo() -> impl IntoView {
    // example: forms
    section(
        "03",
        "Form Controls",
        "フォーム — 入力・選択・アップロード",
        cards_grid((
            half(text_inputs()),
            half(selection_editors()),
            third(check_controls()),
            third(switch_controls()),
            third(upload_controls()),
        )),
    )
    // end example
}


struct IconInput {
    root: Node,
    input: kivact::ui::TextInput,
    glyph: &'static str,
    trailing: View,
}
impl Element for IconInput {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl IntoView for IconInput {
    fn into_view(self) -> View {
        let Self {
            mut root,
            mut input,
            glyph,
            trailing,
        } = self;
        if let Some(read) = root.accessible_name() {
            input = input.name(move || read.get());
        }
        if let Some(read) = root.accessible_disabled() {
            let own = input.node_mut().accessible_disabled();
            input = input.disabled(move || read.get() || own.as_ref().is_some_and(|f| f.get()));
        }
        if let Some(read) = root.accessible_invalid() {
            let own = input.node_mut().accessible_invalid();
            input = input.invalid(move || read.get() || own.as_ref().is_some_and(|f| f.get()));
        }
        for reference in root.accessible_labelled_by() {
            input = input.with_node(|node| node.labelled_by(reference));
        }
        for reference in root.accessible_described_by() {
            input = input.with_node(|node| node.described_by(reference));
        }
        root.child((
            kivact::view::icon(glyph).style(style().color(color::TEXT_MUTED)),
            input,
            trailing,
        ))
        .into_view()
    }
}

fn input_with_icon(
    glyph: &'static str,
    input: kivact::ui::TextInput,
    trailing: impl IntoView,
    invalid: bool,
) -> IconInput {
    let input_ref = node_ref();
    let focused = input_ref.focus_within();
    let input = input.with_node(|node| node.node_ref(input_ref)).style(
        style()
            .flex_grow(1)
            .height(34)
            .min_width(40)
            .padding_x(0)
            .background(Color::TRANSPARENT)
            .border(0, Color::TRANSPARENT)
            .shadow(Shadow::new(0, 0, 0, Color::TRANSPARENT)),
    );
    let root = row(())
        .style(
            style()
                .height(36)
                .gap(8)
                .padding_x(12)
                .align_items(Align::Center)
                .background(color::SURFACE)
                .border_radius(8),
        )
        .style(move || {
            let theme = &use_theme().get();
            let p = theme.ext::<ui::Tokens>();
            style()
                .border(
                    1,
                    if focused.get() {
                        p.blue
                    } else if invalid {
                        p.red
                    } else {
                        p.line
                    },
                )
                .ring(if focused.get() { 3 } else { 0 }, color::FOCUS)
        });
    IconInput {
        root,
        input,
        glyph,
        trailing: trailing.into_view(),
    }
}

fn switch_row(label: &'static str, hint: &'static str, checked: bool, hue: ui::Hue) -> Node {
    row((
        column((
            text(label).style(style().font_weight(500)),
            muted(hint).style(style().font_size(13)),
        ))
        .style(style().flex_grow(1).gap(2)),
        toned(hue, move || {
            kivact::ui::switch(label, signal(checked))
                .hide_label(true)
                .style(style().width(32).padding_left(0))
        }),
    ))
    .style(style().align_items(Align::Center))
}

/// `child` drawn with `hue` as the accent: the kit's blue and the core
/// accent of the theme in effect become the hue's color.
fn toned<V: IntoView>(hue: ui::Hue, child: impl FnOnce() -> V + 'static) -> View {
    let inherited = memo(|| kivact::style::use_theme().get());
    kivact::style::provide_theme(
        move || {
            let theme = inherited.get();
            let mut tokens = (*theme.ext::<ui::Tokens>()).clone();
            tokens.blue = tokens.accent(hue);
            let colors = kivact::style::theme::Palette {
                accent: tokens.blue,
                ..theme.colors().clone()
            };
            theme.with_colors(colors).with(tokens)
        },
        child,
    )
}

fn upload_controls() -> kivact::ui::Card {
    let files = signal(vec![UploadEntry {
        id: 0,
        name: "作戦報告書_第3章.pdf".into(),
        bytes: Some(2_516_582),
        progress: 100.0,
    }]);
    let file_serial = signal(1u64);
    card(
        "ファイルアップロード",
        column((
            kivact::ui::dropzone("ここにファイルをドロップ")
                .multiple(true)
                .pick_on_press(true)
                .icon(kivact::view::icon("cloud-upload").style(style().size(24, 24)))
                .hint(kivact::i18n::message(
                    "pop.dropzone.hint",
                    "or click to choose · PNG, PDF up to 10 MB",
                ))
                .filters(vec![(
                    "Images and documents".into(),
                    vec!["png".into(), "pdf".into()],
                )])
                .on_files(move |paths| {
                    for file in paths {
                        let path = file.path;
                        let Some(name) = path.file_name() else {
                            continue;
                        };
                        let id = file_serial.get();
                        file_serial.set(id + 1);
                        let entry = UploadEntry {
                            id,
                            name: name.to_string_lossy().into_owned(),
                            bytes: std::fs::metadata(&path).ok().map(|metadata| metadata.len()),
                            progress: 0.0,
                        };
                        files.update(|entries| entries.insert(0, entry));
                        spawn(async move {
                            for _ in 0..6 {
                                kivact::task::sleep(Duration::from_millis(180)).await;
                                files.update(|entries| {
                                    if let Some(entry) =
                                        entries.iter_mut().find(|entry| entry.id == id)
                                    {
                                        entry.progress = (entry.progress + 18.0).min(100.0);
                                    }
                                });
                            }
                        });
                    }
                }),
            for_each(
                files,
                |entry| entry.id,
                |entry| {
                    ui::upload_row(
                        move || entry.get().name,
                        move || entry.get().bytes,
                        move || entry.get().progress,
                    )
                    .test_id(format!("pop-upload-{}", entry.get_untracked().id))
                },
            ),
        ))
        .style(style().gap(12)),
    )
}

// xtask-allow(state): independent: volume and difficulty hold the values of the BGM volume slider and the difficulty slider, each written only by its own slider
fn switch_controls() -> kivact::ui::Card {
    let volume = signal(64.0f64);
    let difficulty = signal(2.0f64);
    card(
        "スイッチ・スライダー",
        column((
            switch_row("オートバトル", "戦闘を自動で進行", true, ui::Hue::Blue),
            switch_row("倍速モード", "演出を短縮", false, ui::Hue::Blue),
            switch_row("ボイス", "生徒のボイスを再生", true, ui::Hue::Pink),
            column((
                row((
                    text("BGM 音量"),
                    text(move || volume.get().to_string()).style(style().color(color::ACCENT)),
                )),
                kivact::ui::slider("BGM音量", volume)
                    .range(
                        ScalarRange::new(0.0, 100.0)
                            .and_then(|range| range.with_step(1.0))
                            .expect("finite ordered range"),
                    )
                    .test_id("pop-volume"),
            ))
            .style(
                style()
                    .gap(6)
                    .padding_top(12)
                    .border_top(1, color::BORDER),
            ),
            column((
                row((
                    text("難易度補正"),
                    text(move || difficulty.get().to_string()).style(style().color(rgb(0xe8b90f))),
                )),
                toned(ui::Hue::Yellow, move || {
                    kivact::ui::slider("難易度補正", difficulty).range(
                        ScalarRange::new(0.0, 5.0)
                            .and_then(|range| range.with_step(1.0))
                            .expect("finite ordered range"),
                    )
                }),
                row((
                    muted("0"),
                    muted("1"),
                    muted("2"),
                    muted("3"),
                    muted("4"),
                    muted("5"),
                ))
                .style(style().justify_content(Justify::SpaceBetween).font_size(12)),
            ))
            .style(style().gap(6)),
        ))
        .style(style().gap(12)),
    )
}

// xtask-allow(state): independent: notifications and reports hold the values of the notifications and weekly-report checkboxes, each toggled only by its own checkbox
fn check_controls() -> kivact::ui::Card {
    let notifications = signal(true);
    let reports = signal(false);
    let mode = signal(Some("normal".to_owned()));
    let modes = signal(vec![
        ListItem::new("normal".to_owned(), "ノーマル"),
        ListItem::new("hard".to_owned(), "ハード"),
        ListItem::new("extreme".to_owned(), "エクストリーム").disabled(true),
    ]);
    card(
        "チェック・ラジオ",
        column((
            column((
                kivact::ui::checkbox("通知を受け取る", notifications),
                muted("依頼の更新をお知らせします").style(style().font_size(13).margin_left(28)),
            ))
            .style(style().gap(2)),
            column((
                kivact::ui::checkbox("週次レポート", reports),
                muted("毎週月曜 9:00 に送信").style(style().font_size(13).margin_left(28)),
            ))
            .style(style().gap(2)),
            column((
                kivact::ui::checkbox("部分選択（indeterminate）", kivact::state::signal(false))
                    .indeterminate(true),
                muted("子項目の一部が選択済み").style(style().font_size(13).margin_left(28)),
            ))
            .style(style().gap(2)),
            node().style(style().height(1).background(color::BORDER)),
            kivact::ui::radio_group("難易度", modes, mode),
            muted("エクストリームの解放条件：Lv.80").style(style().font_size(13)),
            kivact::ui::button("ハードの表示を切り替え").on_press(move || {
                modes.update(|options| {
                    if options.iter().any(|option| option.key() == "hard") {
                        options.retain(|option| option.key() != "hard");
                    } else {
                        options.insert(1, ListItem::new("hard".to_owned(), "ハード"));
                    }
                })
            }),
        ))
        .style(style().gap(12)),
    )
}

fn selection_editors() -> kivact::ui::Card {
    let memo = signal(String::new());
    let count = signal(4.0_f64);
    let tags = signal(vec!["護衛".into(), "調査".into()]);
    let chosen = signal(Some("azure".to_owned()));
    let schools = signal(vec![
        ListItem::new("azure".to_owned(), "アズール学園"),
        ListItem::new("celeste".to_owned(), "セレスト女学院"),
        ListItem::new("million".to_owned(), "ミリオン工科高校"),
        ListItem::new("nox".to_owned(), "ノクス自治区"),
    ]);
    card(
        "セレクト・テキストエリア",
        column((
            kivact::ui::field(
                "所属学園",
                kivact::ui::select("所属学園", schools, chosen).icon(kivact::view::icon("school")),
            ),
            kivact::ui::button("学園一覧を並べ替え")
                .on_press(move || schools.update(|options| options.reverse())),
            column((
                row((
                    text("タグ").style(style().flex_grow(1).font_weight(500).line_height(1.0)),
                    text("Enterで追加").style(style().font_size(12).color(color::TEXT_MUTED)),
                )),
                kivact::ui::tag_input("タグを追加", tags)
                    .on_add(move |tag| tags.update(|items| items.push(tag)))
                    .on_remove(move |index| {
                        tags.update(|items| {
                            items.remove(index);
                        })
                    }),
            ))
            .style(style().gap(8)),
            column((
                row((
                    text("依頼メモ").style(style().flex_grow(1).font_weight(500).line_height(1.0)),
                    text(move || format!("{} / 140", memo.get().chars().count()))
                        .style(style().font_size(12).color(color::TEXT_MUTED)),
                )),
                ui::resizable_text_area(
                    kivact::ui::text_area("依頼メモ", memo)
                        .max_length(140)
                        .placeholder("生徒への連絡事項を書いてください…"),
                    80.0,
                    72.0,
                )
                .test_id("pop-memo-resize"),
            ))
            .style(style().gap(8)),
            columns(
                (
                    kivact::ui::field(
                        "人数",
                        kivact::ui::number_input(
                            "人数",
                            count,
                            kivact::interaction::ScalarRange::new(1.0, 12.0)
                                .and_then(|range| range.with_step(1.0))
                                .expect("invariant: the bounds and the step are finite"),
                        ),
                    ),
                    date_editor(),
                ),
                2,
            ),
        ))
        .style(style().gap(20)),
    )
}

fn text_inputs() -> kivact::ui::Card {
    let name = signal("空崎 ミナト".to_string());
    let mail = signal("minato@halo".to_string());
    let password = signal("kivo-secret".to_string());
    let reveal = signal(false);
    card(
        "テキスト入力",
        column((
            kivact::ui::field(
                "生徒名 *",
                input_with_icon(
                    "user",
                    kivact::ui::text_input("生徒名", name).placeholder("例：空崎 ミナト"),
                    (),
                    false,
                ),
            ),
            kivact::ui::field(
                "メールアドレス",
                input_with_icon(
                    "mail",
                    kivact::ui::text_input("メールアドレス", mail).invalid(true),
                    kivact::view::icon("alert-circle").style(style().color(rgb(0xef4458))),
                    true,
                ),
            )
            .error(Some("有効なメールアドレスを入力してください。".to_owned())),
            kivact::ui::field(
                "アクセスキー",
                input_with_icon(
                    "lock",
                    kivact::ui::text_input("アクセスキー", password).purpose(move || {
                        if reveal.get() {
                            kivact::view::InputPurpose::Text
                        } else {
                            kivact::view::InputPurpose::Password
                        }
                    }),
                    kivact::ui::button("表示切替")
                        .variant(kivact::ui::ButtonVariant::Ghost)
                        .hide_label(true)
                        .icon(kivact::view::icon("eye"))
                        .test_id("pop-password-reveal")
                        .style(style().size(20, 20))
                        .on_press(move || reveal.update(|v| *v = !*v)),
                    false,
                ),
            )
            .hint(|| "8文字以上・英数字を含めてください。"),
            reward_editor(),
        ))
        .style(style().gap(20)),
    )
}

fn date_editor() -> impl kivact::view::Element {
    let date = signal(CalendarDate::new(2026, 10, 3));
    kivact::ui::field(
        "実施日",
        {
            let value: kivact::state::Signal<kivact::i18n::CalendarDate> = date;
            let value = kivact::state::binding(
                move || Some(value.get()),
                move |day| {
                    if let Some(day) = day {
                        value.set(day);
                    }
                },
            );
            kivact::ui::date_input("実施日", value)
                .presentation(kivact::ui::DatePresentation::Desktop)
                .calendar(move |calendar| {
                    calendar
                        .today(CalendarDate::new(2026, 10, 3))
                        .month_title(|date| format!("{}.{:02}", date.year(), date.month()))
                })
        }
        .test_id("pop-date-input"),
    )
}

fn reward_editor() -> impl kivact::view::Element {
    kivact::ui::field(
        "報酬額",
        row((
            row((text("¥"),)).style(
                style()
                    .height(36)
                    .padding_x(12)
                    .align_items(Align::Center)
                    .justify_content(Justify::Center)
                    .background(color::SURFACE_ALT)
                    .border_right(1, color::BORDER),
            ),
            kivact::ui::text_input("報酬額", signal("120,000".to_string())).style(
                style()
                    .height(34)
                    .flex_grow(1)
                    .border(0, Color::TRANSPARENT)
                    .shadow(Shadow::new(0, 0, 0, Color::TRANSPARENT)),
            ),
            row((text("/ 件"),)).style(
                style()
                    .height(36)
                    .padding_x(12)
                    .align_items(Align::Center)
                    .justify_content(Justify::Center)
                    .background(color::SURFACE_ALT)
                    .border_left(1, color::BORDER),
            ),
        ))
        .style(
            style()
                .height(36)
                .align_items(Align::Center)
                .border(1, color::BORDER)
                .border_radius(8)
                .overflow(kivact::style::Overflow::Hidden),
        ),
    )
}

