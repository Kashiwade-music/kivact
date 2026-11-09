//! Role: Dynamic, caller-owned choice identities in the existing Pop catalogue.

use super::common::{card, cards_grid, half, section, wrap};
use kivact::interaction::ListItem;
use kivact::prelude::{ColorToken, Element, IntoText, IntoValue, Node, column, row, signal, text};
use kivact::style::{Align, style};
use kivact::ui::RadioGroup;
use kivact::view::Orientation;
use kivact_ui_pop as ui;

fn option(key: &str, label: &str) -> ListItem<String> {
    ListItem::new(key.to_owned(), label)
}

/// A segmented control: a standard radio group in the kit's pill look, its
/// segments showing the icon, the label and the count badge.
fn segmented<K: kivact::state::ItemKey>(
    label: impl IntoText,
    options: impl IntoValue<Vec<ListItem<K>>>,
    value: impl kivact::state::IntoBinding<Option<K>>,
) -> RadioGroup<K> {
    kivact::ui::radio_group(label, options, value)
        .orientation(Orientation::Horizontal)
        .variant(kivact::ui::RadioGroupVariant::Segmented)
}

pub(super) fn selection() -> Node {
    let tab = signal("overview".to_owned());
    let tabs = signal(vec![
        option("overview", "概要"),
        option("skills", "スキル").with_description("4"),
        option("equipment", "装備"),
        option("profile", "プロフィール"),
    ]);
    let periods = signal(vec![
        option("day", "日"),
        option("week", "週"),
        option("month", "月"),
        option("year", "年"),
    ]);
    let period = signal(Some("day".to_owned()));
    let layouts = signal(vec![
        option("cards", "カード").with_icon("layout-grid"),
        option("list", "リスト").with_icon("list"),
        option("chart", "チャート").with_icon("chart-bar"),
    ]);
    let layout = signal(Some("cards".to_owned()));
    let segments = column((
        segmented("Period", periods, period),
        segmented("Layout", layouts, layout),
        kivact::ui::button("表示方法を並べ替え")
            .on_press(move || layouts.update(|options| options.reverse())),
        wrap((
            kivact::ui::toggle_button("ストライカー", signal(true)),
            kivact::ui::toggle_button("スペシャル", kivact::state::signal(false)),
            kivact::ui::toggle_button("タンク", signal(true)),
            kivact::ui::toggle_button("ヒーラー", kivact::state::signal(false)),
            kivact::ui::toggle_button("サポート", kivact::state::signal(false)),
        )),
    ))
    .style(style().gap(16).align_items(Align::Start));
    let tab_choices = column((
        kivact::ui::tabs("スキルの表示", tabs, tab)
            .item(ui::tab_label)
            .content(|key| {
                text(panel(Some(key.as_str()))).style(style().color(ColorToken::TextMuted))
            }),
        row((
            kivact::ui::button("タブ順を入れ替え")
                .on_press(move || tabs.update(|options| options.reverse())),
            kivact::ui::button("スキルの表示を切り替え").on_press(move || {
                tabs.update(|options| {
                    if options.iter().any(|option| option.key() == "skills") {
                        options.retain(|option| option.key() != "skills");
                    } else {
                        options.insert(
                            1.min(options.len()),
                            option("skills", "スキル").with_description("4"),
                        );
                    }
                })
            }),
        ))
        .style(style().gap(8).margin_top(12)),
        segmented(
            "Filter",
            vec![
                option("all", "すべて"),
                option("unread", "未読"),
                option("favorites", "お気に入り"),
            ],
            signal(Some("all".to_owned())),
        )
        .style(style().margin_top(16)),
    ))
    .style(style().align_items(Align::Start));
    section(
        "04",
        "Selection & Tabs",
        "セグメント・タブ・トグル",
        cards_grid((
            half(card("セグメントコントロール", segments)),
            half(card("タブ", tab_choices)),
        )),
    )
}

fn panel(key: Option<&str>) -> String {
    match key {
        Some("overview") => {
            "冷静沈着な調査部の副部長。後方からの精密射撃で味方を支援します。ストライカー / 後衛"
        }
        Some("skills") => "EX スキル「蒼穹の弾幕」— 範囲内の敵に攻撃力の 412% のダメージ。",
        Some("equipment") => "帽子 T7 / グローブ T6 / 腕時計 T7",
        Some("profile") => "誕生日 10月3日 · 趣味 天体観測 · 身長 158cm",
        _ => "タブを選択してください。",
    }
    .to_owned()
}
