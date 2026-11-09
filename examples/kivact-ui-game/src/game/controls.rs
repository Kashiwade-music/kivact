//! Role: controls specimens for the operation-console catalog.

use super::common::{
    cards, line, list, meta, segmented, specimen, strings, suppress_search_shortcut, wide,
};
use super::state::Demo;
use kivact::interaction::{ListItem, MenuEntry, Toast, Tone, toaster};
use kivact::platform;
use kivact::prelude::{
    ColorToken, Element, IntoView, Node, NodeRef, Value, column, dynamic, for_index, node, row,
    signal, spacer, text,
};
use kivact::state::IntoValue;
use kivact::state::binding;
use kivact::style::use_theme;
use kivact::style::{Align, auto, pct, style};
use kivact::task::spawn;
use kivact_ui_game as g;
use std::time::Duration;

pub(super) fn choice_options(items: &[(&str, &str)]) -> Vec<ListItem<String>> {
    items
        .iter()
        .map(|(key, label)| ListItem::new((*key).to_owned(), *label))
        .collect()
}

pub(super) fn buttons(scene: Demo) -> Node {
    let busy = signal(false);
    let verify = signal(false);
    let period = signal(Some("7d".to_owned()));
    let grid_on = signal(true);
    let filter_on = signal(false);
    cards(
        (
            button_variants(scene),
            button_sizes(scene),
            button_states(busy, verify),
            button_group(scene, period),
            button_toolbar(scene, grid_on, filter_on),
            chevron_action().note({
                text("画面に1つだけ置く最重要アクション。通常フォームでは .btn--primary を使う。")
            }),
        ),
        330.,
    )
}
pub(super) fn separator() -> Node {
    node().style(
        style()
            .size(1, 18)
            .margin_x(4)
            .background(ColorToken::Border),
    )
}
pub(super) fn inputs(search: NodeRef) -> Node {
    let tags = signal(strings(&["infra", "定期点検"]));
    let code = signal("OPS_2041!".to_owned());
    let invalid = move || {
        let value = code.get();
        value.is_empty()
            || !value
                .chars()
                .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '-')
    };
    let memo = signal(String::new());
    let files = signal(vec![("inventory_2026Q3.csv".to_owned(), 64_f64)]);
    cards(
        (
            text_fields(code, invalid),
            validated_fields(search),
            text_and_numbers(memo),
            file_upload(files),
            tag_entry(tags),
        ),
        330.,
    )
}
pub(super) fn selection() -> Node {
    let permissions = signal((true, true, false));
    let all = binding(
        (move || {
            let v = permissions.get();
            v.0 && v.1 && v.2
        })
        .into_value(),
        move |v| permissions.set((v, v, v)),
    );
    let mixed = move || {
        let v = permissions.get();
        (v.0 || v.1 || v.2) && !(v.0 && v.1 && v.2)
    };
    let cpu = signal(72_f64);
    cards(
        (
            checkbox_choices(permissions, all, mixed).on_key(suppress_search_shortcut),
            radio_choices().on_key(suppress_search_shortcut),
            switch_choices().on_key(suppress_search_shortcut),
            select_choices(),
            slider_choices(cpu),
            selectable_cards().on_key(suppress_search_shortcut),
        ),
        330.,
    )
}
pub(super) fn setting(title: &str, description: &str, control: impl IntoView) -> Node {
    row((
        column((
            text(title.to_owned()).style(style().weight(700)),
            text(description.to_owned())
                .style(style().font_size(11.5).color(ColorToken::TextMuted)),
        ))
        .style(style().grow(1).gap(3)),
        control,
    ))
    .style(
        style()
            .padding_x(14)
            .padding_y(12)
            .gap(16)
            .align_items(Align::Center)
            .border_bottom(1, ColorToken::Border),
    )
}
/// The navigation specimens.
pub(super) fn navigation() -> Node {
    let underline = signal("summary".to_owned());
    let boxed_tabs = signal("metrics".to_owned());
    let step = signal(1_usize);
    let page = signal(2_usize);
    cards(
        (
            underline_tabs(underline),
            boxed_tab_examples(boxed_tabs),
            wide(wizard_steps(step)),
            pagination_controls(page),
            vertical_navigation(),
        ),
        460.,
    )
}

fn button_variants(scene: Demo) -> crate::game::specimens::Specimen<impl kivact::view::Element> {
    specimen(
        "BTN-01",
        "バリアント",
        line((
            kivact::ui::button("承認する")
                .variant(kivact_ui_game::Slant)
                .test_id("game-primary")
                .on_press(move || {
                    toaster().push(
                        Toast::new("承認")
                            .body("案件を承認しました。")
                            .tone(Tone::Success)
                            .duration(Some(Duration::from_secs(3))),
                    );
                }),
            kivact::ui::button("下書き保存")
                .variant(kivact_ui_game::Slant)
                .variant(kivact::ui::ButtonVariant::Secondary),
            kivact::ui::button("キャンセル")
                .variant(kivact_ui_game::Slant)
                .variant(kivact::ui::ButtonVariant::Ghost),
            kivact::ui::button("削除")
                .variant(kivact_ui_game::Slant)
                .icon(kivact::view::icon("trash"))
                .variant(kivact::ui::ButtonVariant::Danger)
                .on_press(move || scene.delete.set(true)),
            kivact::ui::button("コンソール")
                .variant(kivact_ui_game::Slant)
                .icon(kivact::view::icon("terminal-2"))
                .variant(kivact_ui_game::ButtonTone::Dark)
                .on_press(move || scene.palette.set(true)),
        )),
    )
}

fn button_sizes(scene: Demo) -> crate::game::specimens::Specimen<impl kivact::view::Element> {
    specimen(
        "BTN-02",
        "サイズ / アイコン",
        line((
            kivact::ui::button("Small")
                .variant(kivact_ui_game::Slant)
                .size(kivact::ui::ControlSize::Small),
            kivact::ui::button("Medium").variant(kivact_ui_game::Slant),
            kivact::ui::button("新規作成")
                .variant(kivact_ui_game::Slant)
                .icon(kivact::view::icon("plus"))
                .size(kivact::ui::ControlSize::Large)
                .on_press(move || scene.create.set(true)),
            kivact::ui::tooltip("ダウンロード")
                .placement(kivact::view::Placement::Below)
                .align(kivact::view::OverlayAlign::Center)
                .offset(6.)
                .attach(
                    kivact::ui::button("ダウンロード")
                        .variant(kivact_ui_game::Slant)
                        .icon(kivact::view::icon("download"))
                        .hide_label(true)
                        .variant(kivact::ui::ButtonVariant::Secondary)
                        .style(style().width(34).padding_x(0)),
                ),
            kivact::ui::tooltip("追加")
                .placement(kivact::view::Placement::Below)
                .align(kivact::view::OverlayAlign::Center)
                .offset(6.)
                .attach(
                    kivact::ui::button("追加")
                        .variant(kivact_ui_game::Slant)
                        .icon(kivact::view::icon("plus"))
                        .hide_label(true)
                        .style(style().width(34).padding_x(0)),
                ),
        )),
    )
}

fn button_states(
    busy: kivact::state::Signal<bool>,
    verify: kivact::state::Signal<bool>,
) -> crate::game::specimens::Specimen<impl kivact::view::Element> {
    specimen(
        "BTN-03",
        "状態",
        line((
            kivact::ui::button("同期")
                .variant(kivact_ui_game::Slant)
                .test_id("game-busy")
                .icon(kivact::view::icon("refresh"))
                .busy(busy)
                .on_press(move || {
                    busy.set(true);
                    spawn(async move {
                        kivact::task::sleep(Duration::from_millis(1400)).await;
                        busy.set(false);
                    });
                }),
            kivact::ui::button("検証を実行")
                .variant(kivact_ui_game::Slant)
                .variant(kivact::ui::ButtonVariant::Secondary)
                .busy(verify)
                .on_press(move || {
                    verify.set(true);
                    spawn(async move {
                        kivact::task::sleep(Duration::from_millis(1400)).await;
                        verify.set(false);
                    });
                }),
            kivact::ui::button("無効")
                .variant(kivact_ui_game::Slant)
                .test_id("game-disabled")
                .disabled(true),
            kivact::ui::button("無効")
                .variant(kivact_ui_game::Slant)
                .variant(kivact::ui::ButtonVariant::Secondary)
                .disabled(true),
        )),
    )
}

fn button_group(
    scene: Demo,
    period: kivact::state::Signal<Option<String>>,
) -> crate::game::specimens::Specimen<impl kivact::view::Element> {
    specimen(
        "BTN-04",
        "ボタングループ",
        list((
            segmented(
                "Period",
                choice_options(&[("24h", "24H"), ("7d", "7D"), ("30d", "30D"), ("90d", "90D")]),
                period,
            ),
            line((
                kivact::ui::button("保存して公開")
                    .variant(kivact_ui_game::Slant)
                    .icon(kivact::view::icon("device-floppy")),
                kivact::ui::menu(
                    "▾",
                    vec![
                        MenuEntry::item("schedule", "予約公開…"),
                        MenuEntry::item("limited", "限定公開で保存"),
                        MenuEntry::item("draft", "下書きとして保存"),
                    ],
                )
                .on_activate(move |_: &'static str| scene.create.set(true))
                .offset(0.)
                .attach(
                    kivact::ui::button("▾")
                        .variant(kivact_ui_game::Slant)
                        .variant(kivact::ui::ButtonVariant::Secondary),
                ),
            )),
        )),
    )
}

fn button_toolbar(
    scene: Demo,
    grid_on: kivact::state::Signal<bool>,
    filter_on: kivact::state::Signal<bool>,
) -> crate::game::specimens::Specimen<impl kivact::view::Element> {
    specimen(
        "BTN-05",
        "ツールバー",
        line((
            kivact::ui::toolbar("Console tools").start((
                kivact::ui::tooltip("元に戻す Ctrl+Z")
                    .placement(kivact::view::Placement::Below)
                    .align(kivact::view::OverlayAlign::Center)
                    .offset(6.)
                    .attach(
                        g::tool_button("戻す")
                            .icon(kivact::view::icon("history"))
                            .hide_label(true),
                    ),
                g::tool_button("コピー")
                    .icon(kivact::view::icon("copy"))
                    .hide_label(true)
                    .on_press(|| {
                        spawn(platform::copy_text("OPS-2041"));
                    }),
                separator(),
                g::tool_button("グリッド表示")
                    .icon(kivact::view::icon("layout-grid"))
                    .hide_label(true)
                    .with_node(|node| node.selected(grid_on))
                    .on_press(move || grid_on.update(|v| *v = !*v)),
                g::tool_button("フィルタ")
                    .icon(kivact::view::icon("filter"))
                    .hide_label(true)
                    .test_id("game-toolbar-filter")
                    .with_node(|node| node.selected(filter_on))
                    .on_press(move || filter_on.update(|v| *v = !*v)),
                separator(),
                g::tool_button("編集")
                    .icon(kivact::view::icon("pencil"))
                    .hide_label(true),
                g::tool_button("削除")
                    .icon(kivact::view::icon("trash"))
                    .hide_label(true)
                    .on_press(move || scene.delete.set(true)),
            )),
            kivact::ui::button("詳細を表示 ↗")
                .variant(kivact_ui_game::Slant)
                .variant(kivact::ui::ButtonVariant::Ghost)
                .on_press(move || scene.drawer.set(true)),
        )),
    )
}

fn chevron_action() -> crate::game::specimens::Specimen<impl kivact::view::Element> {
    specimen(
        "BTN-06",
        "シェブロン CTA",
        g::chevron("▷ DEPLOY", "READY").on_press(move || {
            toaster().push(
                Toast::new("デプロイ完了")
                    .body("すべてのワーカーが起動しました。")
                    .tone(Tone::Success)
                    .duration(Some(Duration::from_secs(4))),
            );
        }),
    )
}

fn text_fields(
    code: kivact::state::Signal<String>,
    invalid: impl Fn() -> bool + Copy + 'static,
) -> crate::game::specimens::Specimen<impl kivact::view::Element> {
    specimen(
        "INP-01",
        "テキストフィールド",
        list((
            kivact::ui::field(
                "案件名 *",
                kivact::ui::text_input("案件名", signal("第 3 区画 定期点検".to_owned()))
                    .test_id("game-title-input"),
            )
            .hint("REQUIRED · 一覧とメール通知の件名に使用されます。"),
            kivact::ui::field(
                "管理コード",
                kivact::ui::text_input("管理コード", code)
                    .invalid(invalid)
                    .style(move || {
                        let t = &use_theme().get();
                        style()
                            .border_left(
                                3,
                                if invalid() {
                                    t.colors().danger
                                } else {
                                    t.colors().success
                                },
                            )
                            .border_color(if invalid() {
                                t.colors().danger
                            } else {
                                t.colors().success
                            })
                    }),
            )
            .hint("使用可能なコードです")
            .error(move || {
                invalid().then(|| "英大文字・数字・ハイフンのみ使用できます".to_owned())
            }),
            kivact::ui::field(
                "エンドポイント",
                kivact::ui::text_input(
                    "エンドポイント",
                    signal("api.ops.example.internal".to_owned()),
                )
                .style(move || {
                    let t = &use_theme().get();
                    style()
                        .border_left(3, t.colors().success)
                        .border_color(t.colors().success)
                }),
            )
            .hint("疎通確認済み (38ms)"),
        )),
    )
}

fn validated_fields(
    search: NodeRef,
) -> crate::game::specimens::Specimen<impl kivact::view::Element> {
    specimen(
        "INP-02",
        "装飾付き入力",
        list((
            kivact::ui::field(
                "検索",
                kivact::ui::input_group(
                    "",
                    (
                        kivact::ui::text_input("検索", kivact::state::signal(String::new()))
                            .with_node(|node| node.node_ref(search))
                            .test_id("game-search-input")
                            .placeholder("ID・件名・担当者")
                            .style(style().width(pct(100)).padding_left(34).padding_right(52)),
                    ),
                )
                .variant(g::GameInputGroupVariant::Adorned)
                .start(kivact::view::icon("search"))
                .end(g::mono("/")),
            ),
            kivact::ui::field(
                "Webhook URL",
                kivact::ui::input_group(
                    "",
                    (
                        kivact::ui::text_input("Webhook URL", kivact::state::signal(String::new()))
                            .placeholder("hooks.example.com/ops"),
                    ),
                )
                .variant(g::GameInputGroupVariant::Inline)
                .start(g::mono("https://"))
                .end(g::mono("POST")),
            ),
            kivact::ui::field(
                "月次予算",
                kivact::ui::input_group(
                    "",
                    (
                        kivact::ui::text_input("月次予算", signal("1,200,000".to_owned()))
                            .style(style().width(pct(100)).padding_left(34).padding_right(52)),
                    ),
                )
                .variant(g::GameInputGroupVariant::Adorned)
                .start(kivact::view::icon("database"))
                .end(g::mono("JPY")),
            ),
            kivact::ui::field(
                "API キー ID",
                kivact::ui::text_input("API キー ID", signal("key_7f3a…c91e".to_owned()))
                    .read_only(true)
                    .on_key(suppress_search_shortcut)
                    .style(style().background(ColorToken::SurfaceAlt)),
            ),
        )),
    )
}

fn text_and_numbers(
    memo: kivact::state::Signal<String>,
) -> crate::game::specimens::Specimen<impl kivact::view::Element> {
    specimen(
        "INP-03",
        "テキストエリア / 数値",
        list((
            kivact::ui::field(
                "作業メモ",
                kivact::ui::text_area("作業メモ", memo)
                    .max_length(200)
                    .placeholder("引き継ぎ事項を入力…"),
            )
            .hint(move || format!("{} / 200", memo.get().chars().count())),
            line((
                kivact::ui::field(
                    "同時実行数",
                    kivact::ui::number_input(
                        "同時実行数",
                        signal(4.0_f64),
                        kivact::interaction::ScalarRange::new(1.0, 32.0)
                            .and_then(|range| range.with_step(1.0))
                            .expect("invariant: the bounds and the step are finite"),
                    ),
                )
                .style(style().width(140)),
                kivact::ui::field(
                    "リージョン",
                    kivact::ui::text_input("リージョン", signal("ap-northeast-1".to_owned()))
                        .disabled(true),
                )
                .style(style().width(150)),
            )),
        )),
    )
}

fn file_upload(
    files: kivact::state::Signal<Vec<(String, f64)>>,
) -> crate::game::specimens::Specimen<impl kivact::view::Element> {
    specimen(
        "INP-04",
        "ファイルアップロード",
        list((
            kivact::ui::dropzone(kivact::i18n::message("game.dropzone.label", "Upload files"))
                .multiple()
                .pick_on_press()
                .icon(kivact::view::icon("cloud-upload"))
                .on_files(move |paths| {
                    for file in paths {
                        let path = file.path;
                        let name = path
                            .file_name()
                            .map(|v| v.to_string_lossy().into_owned())
                            .unwrap_or_else(|| path.display().to_string());
                        let index = files.with_untracked(Vec::len);
                        files.update(|v| v.push((name, 0.)));
                        spawn(async move {
                            for _ in 0..10 {
                                kivact::task::sleep(Duration::from_millis(220)).await;
                                files.update(|v| {
                                    if let Some(file) = v.get_mut(index) {
                                        file.1 = (file.1 + 10.).min(100.);
                                    }
                                });
                            }
                        });
                    }
                })
                .hint(kivact::i18n::message("game.dropzone.hint", "CSV / JSON"))
                .filters(vec![(
                    "CSV / JSON".to_owned(),
                    vec!["csv".to_owned(), "json".to_owned()],
                )]),
            for_index(files, |index, item: Value<(String, f64)>| {
                row((
                    dynamic(
                        move || item.get().1 >= 100.,
                        |done| {
                            kivact::view::icon(if done { "circle-check" } else { "file-text" })
                                .style(style().color(if done {
                                    ColorToken::Success
                                } else {
                                    ColorToken::Accent
                                }))
                        },
                    ),
                    column((
                        text(move || item.get().0).style(style().weight(700).font_size(12)),
                        kivact::ui::progress(move || Some(item.get().1 / 100.0))
                            .name("ファイル処理")
                            .variant(kivact::interaction::Tone::Info)
                            .variant(g::Striped)
                            .moving(true)
                            .size(kivact::ui::ControlSize::Small),
                    ))
                    .style(style().grow(1).gap(4)),
                    g::mono(move || format!("{:.0}%", item.get().1))
                        .test_id(format!("game-file-progress-{index}")),
                ))
                .style(
                    style()
                        .padding_x(10)
                        .padding_y(8)
                        .align_items(Align::Center)
                        .gap(10)
                        .border(1, ColorToken::Border)
                        .background(ColorToken::SurfaceAlt),
                )
            }),
        )),
    )
}

fn tag_entry(
    tags: kivact::state::Signal<Vec<String>>,
) -> crate::game::specimens::Specimen<impl kivact::view::Element> {
    specimen(
        "INP-05",
        "タグ入力",
        kivact::ui::field(
            "ラベル",
            kivact::ui::tag_input("タグを追加", tags)
                .on_add(move |tag| tags.update(|items| items.push(tag)))
                .on_remove(move |index| {
                    tags.update(|items| {
                        if index < items.len() {
                            items.remove(index);
                        }
                    })
                }),
        ),
    )
}

fn checkbox_choices(
    permissions: kivact::state::Signal<(bool, bool, bool)>,
    all: kivact::state::Binding<bool>,
    mixed: impl Fn() -> bool + Copy + 'static,
) -> crate::game::specimens::Specimen<impl kivact::view::Element> {
    specimen(
        "SEL-01",
        "チェックボックス",
        list((
            kivact::ui::checkbox("全権限", all).indeterminate(mixed),
            column((
                kivact::ui::checkbox("閲覧", permissions.project(|v| &v.0, |v| &mut v.0)),
                kivact::ui::checkbox("編集", permissions.project(|v| &v.1, |v| &mut v.1)),
                kivact::ui::checkbox("承認", permissions.project(|v| &v.2, |v| &mut v.2)),
                kivact::ui::checkbox("監査ログ削除（無効）", kivact::state::signal(false))
                    .disabled(true),
            ))
            .style(style().padding_left(24).gap(10)),
        )),
    )
}

fn radio_choices() -> crate::game::specimens::Specimen<impl kivact::view::Element> {
    specimen(
        "SEL-02",
        "ラジオ（ダイヤ）",
        list((
            kivact::ui::radio_group(
                "同期方式",
                choice_options(&[
                    ("live", "リアルタイム同期"),
                    ("scheduled", "定期同期"),
                    ("manual", "手動"),
                ]),
                signal(Some("live".to_owned())),
            ),
            kivact::ui::radio_group(
                "外部トリガー",
                vec![
                    ListItem::new("external".to_owned(), "外部トリガー")
                        .with_description("Enterprise プランのみ")
                        .disabled(true),
                ],
                signal(None),
            ),
        )),
    )
}

fn switch_choices() -> crate::game::specimens::Specimen<impl kivact::view::Element> {
    specimen(
        "SEL-03",
        "スイッチ",
        list((
            setting(
                "二要素認証を必須化",
                "全メンバーのログインに適用",
                kivact::ui::switch("二要素認証を必須化", signal(true)).hide_label(true),
            ),
            setting(
                "夜間メンテナンス通知",
                "02:00–04:00 JST",
                kivact::ui::switch("夜間メンテナンス通知", signal(false)).hide_label(true),
            ),
            setting(
                "SSO 強制",
                "管理者がロック中",
                kivact::ui::switch("SSO 強制", signal(true))
                    .hide_label(true)
                    .disabled(true),
            ),
        )),
    )
}

fn select_choices() -> crate::game::specimens::Specimen<impl kivact::view::Element> {
    specimen(
        "SEL-04",
        "セレクト",
        list((
            kivact::ui::field(
                "担当部署",
                kivact::ui::select(
                    "Select",
                    choice_options(&[
                        ("maintenance", "整備部"),
                        ("information", "情報管理部"),
                        ("finance", "会計室"),
                        ("security", "警備局"),
                    ]),
                    signal(Some("maintenance".to_owned())),
                )
                .placeholder("")
                .name("担当部署"),
            )
            .on_key(suppress_search_shortcut),
            kivact::ui::field(
                "タイムゾーン",
                kivact::ui::select(
                    "Select",
                    choice_options(&[
                        ("jst", "JST (UTC+09:00)"),
                        ("utc", "UTC"),
                        ("pst", "PST (UTC-08:00)"),
                    ]),
                    signal(Some("jst".to_owned())),
                )
                .placeholder("")
                .name("タイムゾーン"),
            )
            .hint(|| "作業時刻を表示するタイムゾーンを選択します。"),
        )),
    )
}

fn slider_choices(
    cpu: kivact::state::Signal<f64>,
) -> crate::game::specimens::Specimen<impl kivact::view::Element> {
    specimen(
        "SEL-05",
        "スライダー / セグメント",
        list((
            kivact::ui::field(
                "CPU 上限",
                line((
                    kivact::ui::slider("CPU 上限", cpu)
                        .test_id("game-cpu-slider")
                        .style(style().grow(1).width(auto()))
                        .test_id("game-cpu-slider")
                        .style(style().grow(1).width(auto())),
                    meta(move || format!("{:.0}%", cpu.get())),
                )),
            )
            .hint(
                || "0                 25                 50                 75                 100",
            )
            .on_key(suppress_search_shortcut),
            kivact::ui::field(
                "ログレベル",
                segmented(
                    "Log level",
                    choice_options(&[
                        ("debug", "DEBUG"),
                        ("info", "INFO"),
                        ("warn", "WARN"),
                        ("error", "ERROR"),
                    ]),
                    signal(Some("info".to_owned())),
                ),
            ),
        )),
    )
}

fn selectable_cards() -> crate::game::specimens::Specimen<impl kivact::view::Element> {
    specimen(
        "SEL-06",
        "選択カード",
        kivact::ui::radio_group(
            "プラン",
            vec![
                ListItem::new("standard".to_owned(), "STANDARD").with_description("10 seats"),
                ListItem::new("tactical".to_owned(), "TACTICAL").with_description("50 seats · SSO"),
                ListItem::new("command".to_owned(), "COMMAND").with_description("無制限 · 監査"),
            ],
            signal(Some("tactical".to_owned())),
        )
        .variant(g::ChoiceCard)
        .item(|item, state| {
            let price = match item.with_untracked(|item| item.key().clone()).as_str() {
                "standard" => "¥9,800/月",
                "tactical" => "¥39,800/月",
                _ => "要相談",
            };
            (
                g::card_label(item, state),
                g::mono(price).style(style().color(ColorToken::Accent)),
            )
        }),
    )
}

fn underline_tabs(
    underline: kivact::state::Signal<String>,
) -> crate::game::specimens::Specimen<impl kivact::view::Element> {
    specimen(
        "NAV-01",
        "タブ（下線）",
        kivact::ui::tabs(
            "案件の表示",
            choice_options(&[
                ("summary", "概要"),
                ("tasks", "タスク"),
                ("files", "ファイル"),
                ("audit", "監査ログ"),
            ]),
            underline,
        )
        .content(|key| {
            text(match key.as_str() {
                "summary" => "案件の基本情報・担当者・期限を表示します。",
                "tasks" => "12 件のタスクのうち 8 件が完了しています。",
                "files" => "添付ファイル 4 件（合計 18.2MB）。",
                "audit" => "直近 30 日の操作履歴を表示します。",
                _ => "",
            })
            .style(style().font_size(12.5).color(ColorToken::TextMuted))
        }),
    )
}

fn boxed_tab_examples(
    boxed_tabs: kivact::state::Signal<String>,
) -> crate::game::specimens::Specimen<impl kivact::view::Element> {
    specimen(
        "NAV-02",
        "タブ（ボックス）",
        kivact::ui::tabs(
            "記録の表示",
            vec![
                ListItem::new("metrics".to_owned(), "メトリクス").with_icon("activity"),
                ListItem::new("logs".to_owned(), "ログ").with_icon("terminal-2"),
                ListItem::new("settings".to_owned(), "設定").with_icon("settings"),
            ],
            boxed_tabs,
        )
        .variant(g::Boxed)
        .content(|key| {
            text(match key.as_str() {
                "metrics" => "p95 レイテンシ 142ms / エラー率 0.04%",
                "logs" => "[09:40:12] INFO worker-3 job=OPS-2041 completed",
                "settings" => "この環境の設定は読み取り専用です。",
                _ => "",
            })
            .style(style().font_size(12.5).color(ColorToken::TextMuted))
        }),
    )
}

fn wizard_steps(
    step: kivact::state::Signal<usize>,
) -> crate::game::specimens::Specimen<impl kivact::view::Element> {
    specimen(
        "NAV-03",
        "ステッパー（ウィザード）",
        list((
            kivact::ui::stepper(
                strings(&["基本情報", "権限設定", "連携", "確認", "完了"])
                    .into_iter()
                    .enumerate()
                    .map(|(index, label)| {
                        kivact::ui::StepperItem::new(label).indicator(
                            kivact::ui::StepIndicator::custom(move || {
                                text(move || {
                                    if index < step.get() {
                                        "✓".to_owned()
                                    } else {
                                        format!("{:02}", index + 1)
                                    }
                                })
                            }),
                        )
                    })
                    .collect::<Vec<_>>(),
                step,
            ),
            row((
                g::mono(move || format!("STEP {} / 5", step.get() + 1)).style(style().grow(1)),
                kivact::ui::button("戻る")
                    .variant(kivact_ui_game::Slant)
                    .size(kivact::ui::ControlSize::Small)
                    .variant(kivact::ui::ButtonVariant::Ghost)
                    .disabled(move || step.get() == 0)
                    .on_press(move || step.update(|v| *v = v.saturating_sub(1))),
                kivact::ui::button(move || {
                    (if step.get() == 4 {
                        "完了 ✓"
                    } else {
                        "次へ ›"
                    })
                    .to_owned()
                })
                .variant(kivact_ui_game::Slant)
                .size(kivact::ui::ControlSize::Small)
                .on_press(move || {
                    if step.get() == 4 {
                        step.set(0);
                        toaster().push(
                            Toast::new("セットアップ完了")
                                .body("ワークスペースの初期設定が完了しました。")
                                .tone(Tone::Success)
                                .duration(Some(Duration::from_secs(4))),
                        );
                    } else {
                        step.update(|v| *v += 1);
                    }
                }),
            ))
            .style(style().gap(12).align_items(Align::Center)),
        )),
    )
}

fn pagination_controls(
    page: kivact::state::Signal<usize>,
) -> crate::game::specimens::Specimen<impl kivact::view::Element> {
    specimen(
        "NAV-04",
        "パンくず / ページネーション",
        list((
            kivact::ui::breadcrumb(
                strings(&["組織", "プロジェクト", "環境: production", "OPS-2041"])
                    .into_iter()
                    .enumerate()
                    .map(|(index, label)| ListItem::new(index, label))
                    .collect::<Vec<_>>(),
            ),
            row((
                kivact::ui::pagination(12, page).item(|item| {
                    text(match item {
                        kivact::ui::PageItem::First => "«".to_owned(),
                        kivact::ui::PageItem::Previous => "‹".to_owned(),
                        kivact::ui::PageItem::Next => "›".to_owned(),
                        kivact::ui::PageItem::Last => "»".to_owned(),
                        kivact::ui::PageItem::Ellipsis => "…".to_owned(),
                        kivact::ui::PageItem::Page(page) => (page + 1).to_string(),
                    })
                }),
                spacer(),
                g::mono(move || {
                    format!(
                        "{}–{} / 289",
                        (page.get() - 1) * 25 + 1,
                        (page.get() * 25).min(289)
                    )
                })
                .style(style().color(ColorToken::TextMuted)),
            ))
            .style(style().align_items(Align::Center)),
        )),
    )
}

fn vertical_navigation() -> crate::game::specimens::Specimen<impl kivact::view::Element> {
    specimen(
        "NAV-05",
        "縦ナビゲーション",
        kivact::ui::tabs(
            "メインメニュー",
            vec![
                ListItem::new("dashboard".to_owned(), "ダッシュボード").with_icon("layout-grid"),
                ListItem::new("operations".to_owned(), "作戦").with_icon("target"),
                ListItem::new("materials".to_owned(), "資材").with_icon("box"),
                ListItem::new("security".to_owned(), "セキュリティ").with_icon("shield-check"),
            ],
            signal("dashboard".to_owned()),
        )
        .orientation(kivact::view::Orientation::Vertical)
        .variant(g::Navigation)
        .item(|item, state| {
            let count = match item.with_untracked(|item| item.key().clone()).as_str() {
                "operations" => Some(kivact::ui::badge("3").tone(kivact::interaction::Tone::Info)),
                "security" => Some(kivact::ui::badge("!").tone(kivact::interaction::Tone::Danger)),
                _ => None,
            };
            (g::navigation_label(item, state), count)
        }),
    )
}
