//! Role: Source catalogue sections built from the public component library.

use super::common::{card, cards_grid, columns, half, muted, section, third, wrap};
use super::components_helpers::{cut_corner_sample, shape_samples, type_samples};
use kivact::draw::Shadow;
use kivact::i18n::CalendarDate;
use kivact::interaction::{ListItem, MenuEntry, ScalarRange};
use kivact::prelude::{
    ColorToken, Element, IntoView, Node, Role, column, for_each, grid, memo, node, rgb, row,
    signal, text,
};
use kivact::state::{Callback, Machine, Step, machine};
use kivact::style::use_theme;
use kivact::style::{Align, Color, Justify, rgba, style};
use kivact::style::{GridPlacement, Track};
use kivact::task::spawn;
use kivact::view::{View, node_ref};
use kivact_ui_pop as ui;
use std::time::Duration;

#[derive(Clone, PartialEq)]
struct UploadEntry {
    id: u64,
    name: String,
    bytes: Option<u64>,
    progress: f64,
}

/// The sync button: idle, or busy until the delayed finish event arrives.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum SyncPhase {
    #[default]
    Idle,
    Running,
}

enum SyncEvent {
    Start,
    Finish,
}

/// What the sync machine asks the page to do after a transition.
enum SyncEffect {
    Finished,
}

struct SyncMachine;

impl Machine for SyncMachine {
    type State = SyncPhase;
    type Event = SyncEvent;
    type Effect = SyncEffect;
    fn transition(
        &self,
        state: &SyncPhase,
        event: SyncEvent,
    ) -> Step<SyncPhase, SyncEvent, SyncEffect> {
        match (state, event) {
            (SyncPhase::Idle, SyncEvent::Start) => {
                Step::to(SyncPhase::Running).after(Duration::from_millis(1600), SyncEvent::Finish)
            }
            (SyncPhase::Running, SyncEvent::Finish) => {
                Step::to(SyncPhase::Idle).with(SyncEffect::Finished)
            }
            _ => Step::stay(),
        }
    }
}

pub(super) fn tokens(notify: Callback<(ui::Hue, String, String)>) -> Node {
    let swatches = color_swatches(notify);
    let type_scale = type_samples();
    let mut shapes = shape_samples();
    shapes.push(cut_corner_sample());
    section(
        "01",
        "Design Tokens",
        "デザイントークン — スウォッチをクリックでコピー",
        cards_grid((
            card(
                "カラーパレット",
                grid(swatches).style(
                    style()
                        .grid_template_columns(Track::repeat_auto_fill(Track::minmax(
                            116,
                            Track::fr(1),
                        )))
                        .gap(12),
                ),
            )
            .style(style().grid_column(GridPlacement::span(8))),
            card("タイポグラフィ", column(type_scale).style(style().gap(12)))
                .style(style().grid_column(GridPlacement::span(4))),
            card(
                "角丸・影・エレベーション",
                grid(shapes).style(
                    style()
                        .grid_template_columns(Track::repeat_auto_fill(Track::minmax(
                            112,
                            Track::fr(1),
                        )))
                        .gap(16),
                ),
            )
            .style(style().grid_column(GridPlacement::span(12))),
        )),
    )
}

pub(super) fn buttons(notify: Callback<(ui::Hue, String, String)>) -> Node {
    use kivact::ui::ButtonVariant;
    use kivact_ui_pop::ButtonTone;
    let variants = [
        ("プライマリ", ButtonVariant::Primary, None),
        (
            "アクセント",
            ButtonVariant::Primary,
            Some(ButtonTone::Accent),
        ),
        (
            "セカンダリ",
            ButtonVariant::Secondary,
            Some(ButtonTone::Soft),
        ),
        ("アウトライン", ButtonVariant::Secondary, None),
        ("ゴースト", ButtonVariant::Ghost, None),
        ("削除する", ButtonVariant::Danger, None),
    ]
    .into_iter()
    .map(|(label, v, hue)| {
        let button = kivact::ui::button(label).variant(v);
        let button = match hue {
            Some(hue) => button.variant(hue),
            None => button,
        };
        button.on_press(move || notify.call((ui::Hue::Blue, "アクション".into(), label.into())))
    })
    .chain(std::iter::once(kivact::ui::button("無効").disabled(true)))
    .collect::<Vec<_>>();
    section(
        "02",
        "Buttons",
        "ボタン — 主要アクションは青、特別な瞬間は黄色",
        cards_grid((
            half(card("バリエーション", wrap(variants))),
            half(card(
                "スラント（ゲームUI調）",
                wrap((
                    kivact::ui::button("編成する").variant(kivact_ui_pop::Slant),
                    kivact::ui::button("出撃")
                        .variant(kivact_ui_pop::ButtonTone::Accent)
                        .variant(kivact_ui_pop::Slant)
                        .icon(kivact::view::icon("sword")),
                    kivact::ui::button("キャンセル")
                        .variant(kivact::ui::ButtonVariant::Secondary)
                        .variant(kivact_ui_pop::Slant),
                    kivact::ui::button("詳細")
                        .variant(kivact_ui_pop::ButtonTone::Soft)
                        .size(kivact::ui::ControlSize::Small)
                        .variant(kivact_ui_pop::Slant),
                )),
            )),
            half(button_sizes()),
            half(button_groups(notify)),
            half(card(
                "標準部品（kivact::ui）— テーマの recipe でこのキットの見た目になる",
                column((
                    wrap((
                        kivact::ui::button("標準ボタン").test_id("standard-button"),
                        kivact::ui::button("標準ゴースト")
                            .variant(kivact::ui::ButtonVariant::Ghost),
                        kivact::ui::button("標準無効").disabled(true),
                    )),
                    wrap((
                        kivact::ui::checkbox("標準チェック", signal(true)),
                        kivact::ui::switch("標準スイッチ", signal(true)),
                    )),
                    kivact::ui::text_input("標準入力", signal("標準の入力欄".to_string()))
                        .style(style().width(240)),
                ))
                .style(style().gap(16)),
            )),
        )),
    )
}

pub(super) fn forms() -> Node {
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
            kivact::view::icon(glyph).style(style().color(ColorToken::TextMuted)),
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
            .grow(1)
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
                .background(ColorToken::Surface)
                .radius(8),
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
                .ring(if focused.get() { 3 } else { 0 }, ColorToken::Focus)
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
            text(label).style(style().weight(500)),
            muted(hint).style(style().font_size(13)),
        ))
        .style(style().grow(1).gap(2)),
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

fn color_swatches(notify: Callback<(ui::Hue, String, String)>) -> Vec<Node> {
    [
        ("Pop Blue", 0x2b8bd8),
        ("Deep Blue", 0x1a66a8),
        ("Sky Cyan", 0x5cbde8),
        ("Signal Yellow", 0xffdc3c),
        ("Talk Pink", 0xff7cb0),
        ("Mint", 0x2fcfa6),
        ("Alert Red", 0xef4458),
        ("Orange", 0xffa53c),
        ("Lavender", 0x9d86ff),
        ("Navy Ink", 0x1d2b4a),
        ("Slate 50", 0xf8fafc),
        ("Night", 0x0b1120),
    ]
    .into_iter()
    .map(|(name, hex)| {
        column((
            node().style(style().height(52).background(rgb(hex))),
            column((
                text(name).style(style().font_size(12.5).weight(500)),
                text(format!("#{hex:06X}")).style(
                    style()
                        .font_family("monospace")
                        .font_size(11)
                        .color(ColorToken::TextMuted),
                ),
            ))
            .style(style().padding_x(10).padding_top(8).padding_bottom(10)),
        ))
        .role(Role::Button)
        .name(name)
        .focusable(true)
        .on_press(move || {
            spawn(kivact::platform::copy_text(format!("#{hex:06X}")));
            notify.call((
                ui::Hue::Blue,
                "コピーしました".into(),
                format!("#{hex:06X}"),
            ));
        })
        .style(
            style()
                .border(1, ColorToken::Border)
                .radius(10)
                .clip(true)
                .hover(style().shadow(Shadow::new(0, 4, 6, ColorToken::Shadow)))
                .focus(style().ring(3, ColorToken::Focus)),
        )
    })
    .collect::<Vec<_>>()
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
                .multiple()
                .pick_on_press()
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

fn switch_controls() -> kivact::ui::Card {
    // xtask-allow(state-fields): independent: bound only to the BGM volume slider; the difficulty slider has its own signal, so any volume pairs with any difficulty
    let volume = signal(64.0f64);
    // xtask-allow(state-fields): independent: bound only to the difficulty slider; the volume slider has its own signal, so any difficulty pairs with any volume
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
                    text(move || volume.get().to_string()).style(style().color(ColorToken::Accent)),
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
                    .border_top(1, ColorToken::Border),
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

fn check_controls() -> kivact::ui::Card {
    // xtask-allow(state-fields): independent: bound only to the notifications checkbox; the weekly report checkbox has its own signal, so both, either or neither can be checked
    let notifications = signal(true);
    // xtask-allow(state-fields): independent: bound only to the weekly report checkbox; the notifications checkbox has its own signal, so both, either or neither can be checked
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
            node().style(style().height(1).background(ColorToken::Border)),
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
                    text("タグ").style(style().grow(1).weight(500).line_height(1.0)),
                    text("Enterで追加").style(style().font_size(12).color(ColorToken::TextMuted)),
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
                    text("依頼メモ").style(style().grow(1).weight(500).line_height(1.0)),
                    text(move || format!("{} / 140", memo.get().chars().count()))
                        .style(style().font_size(12).color(ColorToken::TextMuted)),
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

fn button_sizes() -> kivact::ui::Card {
    card(
        "サイズとアイコン",
        column((
            wrap((
                kivact::ui::button("Small")
                    .size(kivact::ui::ControlSize::Small)
                    .icon(kivact::view::icon("plus")),
                kivact::ui::button("Medium").icon(kivact::view::icon("plus")),
                kivact::ui::button("Large")
                    .size(kivact::ui::ControlSize::Large)
                    .icon(kivact::view::icon("plus")),
            )),
            wrap((
                kivact::ui::button("エクスポート")
                    .variant(kivact::ui::ButtonVariant::Secondary)
                    .trailing(kivact::view::icon("download")),
                kivact::ui::tooltip("追加")
                    .placement(kivact::view::Placement::Below)
                    .align(kivact::view::OverlayAlign::Center)
                    .offset(6.0)
                    .attach(
                        kivact::ui::button("追加")
                            .variant(kivact::ui::ButtonVariant::Ghost)
                            .hide_label(true)
                            .icon(kivact::view::icon("plus"))
                            .style(style().background(ColorToken::Accent).color(Color::WHITE)),
                    ),
                kivact::ui::tooltip("編集")
                    .placement(kivact::view::Placement::Below)
                    .align(kivact::view::OverlayAlign::Center)
                    .offset(6.0)
                    .attach(
                        kivact::ui::button("編集")
                            .variant(kivact::ui::ButtonVariant::Ghost)
                            .hide_label(true)
                            .icon(kivact::view::icon("pencil"))
                            .style(style().background(ColorToken::SurfaceAlt)),
                    ),
                kivact::ui::tooltip("コピー")
                    .placement(kivact::view::Placement::Below)
                    .align(kivact::view::OverlayAlign::Center)
                    .offset(6.0)
                    .attach(
                        kivact::ui::button("コピー")
                            .variant(kivact::ui::ButtonVariant::Ghost)
                            .hide_label(true)
                            .icon(kivact::view::icon("copy"))
                            .style(style().border(1, ColorToken::Border)),
                    ),
                kivact::ui::tooltip("その他")
                    .placement(kivact::view::Placement::Below)
                    .align(kivact::view::OverlayAlign::Center)
                    .offset(6.0)
                    .attach(
                        kivact::ui::button("その他")
                            .variant(kivact::ui::ButtonVariant::Ghost)
                            .hide_label(true)
                            .icon(kivact::view::icon("dots")),
                    ),
                node()
                    .role(Role::Button)
                    .name("新規作成")
                    .focusable(true)
                    .style(
                        style()
                            .size(44, 44)
                            .rotate(45)
                            .radius(10)
                            .background(rgb(0xffdc3c))
                            .align_items(Align::Center)
                            .justify_content(Justify::Center)
                            .focus(style().ring(3, ColorToken::Focus)),
                    )
                    .child(kivact::view::icon("sparkles").style(style().rotate(-45))),
            )),
        ))
        .style(style().gap(16)),
    )
}

fn button_groups(notify: Callback<(ui::Hue, String, String)>) -> kivact::ui::Card {
    let sync = machine(
        SyncPhase::Idle,
        || SyncMachine,
        move |_, effect| match effect {
            SyncEffect::Finished => notify.call((
                ui::Hue::Mint,
                "同期完了".into(),
                "最新のデータを取得しました。".into(),
            )),
        },
    );
    card(
        "グループ・ローディング",
        column((
            wrap((
                view_mode_buttons(),
                kivact::ui::button(move || {
                    if sync.get() == SyncPhase::Running {
                        "同期中…"
                    } else {
                        "同期する"
                    }
                    .to_string()
                })
                .icon(kivact::view::icon("refresh"))
                .busy(move || sync.get() == SyncPhase::Running)
                .on_press(move || sync.send(SyncEvent::Start)),
            )),
            wrap((
                row((
                    kivact::ui::button("保存して公開")
                        .style(style().radius(0))
                        .on_press(move || {
                            notify.call((
                                ui::Hue::Mint,
                                "保存しました".into(),
                                "変更が反映されました。".into(),
                            ))
                        }),
                    kivact::ui::menu(
                        "保存方法",
                        vec![
                            MenuEntry::item("draft", "下書き保存"),
                            MenuEntry::item("schedule", "予約公開…"),
                        ],
                    )
                    .on_activate(|_: &'static str| {})
                    .attach(
                        kivact::ui::button("保存方法")
                            .variant(kivact::ui::ButtonVariant::Ghost)
                            .hide_label(true)
                            .icon(kivact::view::icon("chevron-down"))
                            .style(
                                style()
                                    .radius(0)
                                    .background(ColorToken::Accent)
                                    .color(Color::WHITE)
                                    .border_left(1, rgba(0xffffff, 0.2)),
                            ),
                    ),
                ))
                .style(style().radius(8).clip(true)),
                kivact::ui::button("もっと見る")
                    .variant(kivact::ui::ButtonVariant::Ghost)
                    .size(kivact::ui::ControlSize::Small)
                    .trailing(kivact::view::icon("arrow-right")),
            )),
        ))
        .style(style().gap(16)),
    )
}

fn date_editor() -> impl kivact::view::Element {
    let date = signal(CalendarDate::new(2026, 10, 3));
    kivact::ui::field(
        "実施日",
        {
            let value: kivact::state::Binding<kivact::i18n::CalendarDate> =
                kivact::state::IntoBinding::into_binding(date);
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
                    .background(ColorToken::SurfaceAlt)
                    .border_right(1, ColorToken::Border),
            ),
            kivact::ui::text_input("報酬額", signal("120,000".to_string())).style(
                style()
                    .height(34)
                    .grow(1)
                    .border(0, Color::TRANSPARENT)
                    .shadow(Shadow::new(0, 0, 0, Color::TRANSPARENT)),
            ),
            row((text("/ 件"),)).style(
                style()
                    .height(36)
                    .padding_x(12)
                    .align_items(Align::Center)
                    .justify_content(Justify::Center)
                    .background(ColorToken::SurfaceAlt)
                    .border_left(1, ColorToken::Border),
            ),
        ))
        .style(
            style()
                .height(36)
                .align_items(Align::Center)
                .border(1, ColorToken::Border)
                .radius(8)
                .clip(true),
        ),
    )
}

fn view_mode_buttons() -> Node {
    row((
        kivact::ui::button("グリッド")
            .variant(kivact::ui::ButtonVariant::Secondary)
            .icon(kivact::view::icon("layout-grid"))
            .style(style().radius_corners(8, 0, 0, 8)),
        kivact::ui::button("リスト")
            .variant(kivact::ui::ButtonVariant::Secondary)
            .icon(kivact::view::icon("list"))
            .style(style().radius(0).margin_left(-1)),
        kivact::ui::button("フィルタ")
            .variant(kivact::ui::ButtonVariant::Ghost)
            .hide_label(true)
            .icon(kivact::view::icon("filter"))
            .style(
                style()
                    .border(1, ColorToken::Border)
                    .radius_corners(0, 8, 8, 0)
                    .margin_left(-1),
            ),
    ))
}
