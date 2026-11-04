//! Role: Shared values and view builders for Creative catalog specimens.

use kivact::Binding;
use kivact::binding;
use kivact::prelude::NodeBuilder;
use kivact::prelude::{IntoText, IntoValue, IntoView, Node, Role, column, grid, node, row, text};
use kivact::style::Track;
use kivact::style::{Align, style};
use kivact_ui_creative as ui;

pub(super) fn keyed_options(
    entries: impl IntoIterator<Item = (&'static str, &'static str)>,
) -> Vec<kivact::ui::ListItem<String>> {
    entries
        .into_iter()
        .map(|(key, label)| kivact::ui::ListItem::new(key.to_owned(), label))
        .collect()
}

pub(super) const SECTIONS: [(&str, &str, usize, &str); 22] = [
    (
        "Buttons",
        "BTN",
        1,
        "プッシュ / アイコン / トグル / セグメント / ツールパレット",
    ),
    (
        "Numeric Fields",
        "NUM",
        1,
        "ドラッグ数値 / スライダー / ベクトル / タイムコード / キーフレーム状態",
    ),
    (
        "Selection & Input",
        "SEL",
        1,
        "チェック / スイッチ / 列挙 / テキスト / データブロック",
    ),
    (
        "Indicators & Labels",
        "IND",
        1,
        "LED / 状態 / キーキャップ / ラベル / キャッシュ",
    ),
    ("Transport", "TRP", 2, "再生 / 録音 / ロケーター / LCD"),
    ("Knobs & Faders", "KNB", 2, "ノブ / フェーダー / 範囲"),
    ("Meters", "MTR", 2, "ピーク / 横型 / ラウドネス"),
    (
        "Mixer Console",
        "MIX",
        2,
        "ルーティング / インサート / パン / M S / 音量",
    ),
    (
        "Property Editor",
        "PROP",
        3,
        "折りたたみ / プロパティ / キー状態 / 修飾子",
    ),
    (
        "Areas & Docking",
        "DOCK",
        3,
        "エディタ種類 / 分割 / 最大化 / ドキュメントタブ",
    ),
    ("Outliner", "TREE", 3, "階層 / 選択 / 表示 / ロック"),
    (
        "Media Pool & Data Table",
        "TBL",
        3,
        "並び替え / フィルター / 行選択 / ラベル",
    ),
    (
        "Layer Stack",
        "LYR",
        4,
        "A/V スイッチ / レイヤー / 描画モード / 親子",
    ),
    (
        "Arrange Timeline",
        "ARR",
        4,
        "ルーラー / ループ / トラック / クリップ / オートメーション",
    ),
    (
        "Dope Sheet & Graph",
        "ANIM",
        4,
        "キー選択 / 移動 / ボックス選択 / F カーブ",
    ),
    (
        "Node Editor",
        "NODE",
        5,
        "ノード / 型付きソケット / リンク接続 / パン / ズーム",
    ),
    (
        "Color Grading & Scopes",
        "GRADE",
        5,
        "リフト / ガンマ / ゲイン / パレード / 波形 / ベクトル",
    ),
    (
        "Color Picker",
        "CLR",
        5,
        "インライン / スウォッチ / RGB / HSV / HEX",
    ),
    (
        "3D Viewport",
        "VP",
        5,
        "ビュー / グリッド / 選択 / パン / ズーム",
    ),
    ("Menus", "MENU", 6, "メニュー / 検索 / コンテキスト"),
    (
        "Dialogs & Popovers",
        "DLG",
        6,
        "ウィンドウ / ポップオーバー / ツールチップ / バナー",
    ),
    ("Jobs & Log", "JOB", 6, "進捗 / 通知 / ログ"),
];

pub(super) fn separator() -> Node {
    node()
        .role(Role::Separator)
        .style(style().size(1, 20).margin_x(2))
        .style(style().background(ui::Color::HairStrong))
}

pub(super) fn specs(content: impl IntoView) -> Node {
    grid(content).style(
        style()
            .grid_template_columns(Track::repeat_auto_fill(Track::minmax(290, Track::fr(1))))
            .min_width(0),
    )
}

pub(super) fn spec(title: &str, subtitle: &str, content: impl IntoView) -> Node {
    column((
        row((
            text(title.to_string()).style(style().weight(600).font_size(11.5)),
            text(subtitle.to_string())
                .style(style().font_size(11))
                .style(style().color(ui::Color::Dim)),
        ))
        .style(style().gap(8).min_height(16)),
        content,
    ))
    .style(
        style()
            .padding_top(16)
            .padding_x(18)
            .padding_bottom(20)
            .gap(10)
            .min_width(0),
    )
    .style(
        style()
            .border_right(1, ui::Color::Hair)
            .border_bottom(1, ui::Color::Hair),
    )
}

pub(super) fn line(content: impl IntoView) -> Node {
    row(content).style(style().gap(6).align_items(Align::Center).wrap(true))
}

pub(super) fn col(content: impl IntoView) -> Node {
    column(content).style(style().gap(8).min_width(0))
}

pub(super) fn note(message: impl IntoText) -> kivact::control::Node {
    text(message)
        .style(style().font_size(11.5).line_height(1.7))
        .style(style().color(ui::Color::Dim))
}

pub(super) fn bind<T: Clone + PartialEq + 'static>(
    read: impl IntoValue<T>,
    write: impl Fn(T) + 'static,
) -> Binding<T> {
    binding(read.into_value(), write)
}

/// A row of the catalog's text tables: an id, one text per column and the
/// color of its left edge.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct TextRow {
    pub(super) id: u64,
    pub(super) cells: Vec<String>,
    pub(super) color: Option<kivact::style::Color>,
}

impl TextRow {
    pub(super) fn new(id: u64, cells: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            id,
            cells: cells.into_iter().map(Into::into).collect(),
            color: None,
        }
    }
    pub(super) fn color(mut self, color: kivact::style::Color) -> Self {
        self.color = Some(color);
        self
    }
}

/// A table of `TextRow`s with a column for each `(title, width)` (a width
/// of 0 shares the room), choosing `selected` and sorted by `sort`. Every
/// column sorts by its text and matches the table's filter, ignoring case.
pub(super) fn text_table(
    columns: &[(&str, f64)],
    rows: impl IntoValue<Vec<TextRow>>,
    selected: impl Into<Binding<Vec<u64>>>,
    sort: impl Into<Binding<Option<(usize, kivact::interaction::SortOrder)>>>,
) -> kivact::ui::DataTable<TextRow, u64> {
    let columns = columns
        .iter()
        .enumerate()
        .map(|(i, (title, width))| {
            kivact::ui::DataColumn::new(*title, move |row: &TextRow| {
                row.cells.get(i).cloned().unwrap_or_default()
            })
            .sortable()
            .filter(move |row, query| {
                row.cells
                    .get(i)
                    .is_some_and(|cell| cell.to_lowercase().contains(&query.to_lowercase()))
            })
            .width(*width)
        })
        .collect();
    kivact::ui::data_table(rows, |row: &TextRow| row.id, columns)
        .selection(selected)
        .sort(sort)
        .focus(kivact::interaction::TableFocus::Rows(
            kivact::interaction::ListPolicy {
                typeahead: true,
                range: kivact::interaction::ListRangePolicy {
                    missing_anchor: kivact::interaction::ListMissingAnchor::FirstSelected,
                    ..Default::default()
                },
                ..Default::default()
            },
        ))
        .row_color(|row: &TextRow| row.color)
        // The editors' tables are dense: no selection summary, and the
        // filter text is set by the editor's own field.
        .toolbar(|_| ())
        .footer(|_| ())
}
