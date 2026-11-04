//! Role: Catalog page of Resizable: horizontal, vertical and a grip handle.

use crate::model::{Example, Page, PreviewAlign};
use kivact::prelude::*;
use kivact::ui::split;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page {
        slug: "resizable",
        name: "Resizable",
        description: "境界をドラッグかキーボードで動かして、領域の大きさを変えます。kivact::ui::split を basic の theme で描きます。大きさは合計 1 の割合の並びで、アプリが持ちます。矢印キーで 1 % ずつ、Home と End で下限と上限、Enter と二重クリックで隣と等分にします。どの領域も、既定ではグループの 10 % より小さくなりません（min_size で変えられます）。",
        usage: "use kivact::prelude::*;\nuse kivact::ui::split;\nuse kivact_ui_basic as basic;",
        source: include_str!("resizable.rs"),
        align: PreviewAlign::Center,
        examples: vec![
            Example::new(
                "horizontal",
                "Horizontal",
                "split にパネルの view と割合の束縛を渡します。向きの既定は横並びです。Tab で境界にフォーカスして左右の矢印を押せます。",
                horizontal,
            ),
            Example::new(
                "vertical",
                "Vertical",
                "orientation に Orientation::Vertical を渡すと上下に並べます。境界は上下の矢印で動きます。",
                vertical,
            ),
            Example::new(
                "handle",
                "With handle",
                "grip(true) で境界につまみを描きます。min_size でこれ以上縮められない割合を変えられます。",
                handle,
            ),
        ],
    }
}

// example: horizontal
fn horizontal() -> impl IntoView {
    /// A centered label for a panel.
    fn panel(label: &str) -> impl IntoView {
        node()
            .style(
                style()
                    .height(pct(100))
                    .align_items(Align::Center)
                    .justify_content(Justify::Center),
            )
            .child(basic::large(label.to_string()))
    }
    let sizes = signal(vec![0.25f64, 0.75]);
    split(sizes, vec![panel("One"), panel("Two")]).style(
        style()
            .width(420)
            .height(160)
            .border(1, basic::Color::Border)
            .padding(1),
    )
}
// end example

// example: vertical
fn vertical() -> impl IntoView {
    /// A centered label for a panel.
    fn panel(label: &str) -> impl IntoView {
        node()
            .style(
                style()
                    .height(pct(100))
                    .align_items(Align::Center)
                    .justify_content(Justify::Center),
            )
            .child(basic::large(label.to_string()))
    }
    let sizes = signal(vec![0.3f64, 0.7]);
    split(sizes, vec![panel("Header"), panel("Content")])
        .orientation(Orientation::Vertical)
        .style(
            style()
                .width(420)
                .height(220)
                .border(1, basic::Color::Border)
                .padding(1),
        )
}
// end example

// example: handle
fn handle() -> impl IntoView {
    /// A centered label for a panel.
    fn panel(label: &str) -> impl IntoView {
        node()
            .style(
                style()
                    .height(pct(100))
                    .align_items(Align::Center)
                    .justify_content(Justify::Center),
            )
            .child(basic::large(label.to_string()))
    }
    let sizes = signal(vec![0.25f64, 0.5, 0.25]);
    column((
        split(
            sizes,
            vec![panel("Sidebar"), panel("Content"), panel("Inspector")],
        )
        .grip(true)
        .min_size(0.15)
        .style(
            style()
                .width(480)
                .height(160)
                .border(1, basic::Color::Border)
                .padding(1),
        ),
        basic::muted(move || {
            let shares: Vec<String> = sizes
                .get()
                .iter()
                .map(|size| format!("{:.0}%", size * 100.0))
                .collect();
            format!("Panels: {}", shares.join(" / "))
        }),
    ))
    .style(style().gap(12))
}
// end example
