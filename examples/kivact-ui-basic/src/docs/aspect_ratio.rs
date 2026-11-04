//! Role: Catalog page of Aspect Ratio: a photo at 16:9, square and portrait boxes.

use crate::model::{Example, Page, PreviewAlign};
use kivact::prelude::*;
use kivact::view::Asset;
use kivact::view::ImageFit;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page {
        slug: "aspect-ratio",
        name: "Aspect Ratio",
        description: "利用できる幅いっぱいの箱を、指定した縦横比（幅 / 高さ）で作ります。子は箱の大きさに合わせて伸びます。",
        usage: "use kivact::prelude::*;
use kivact_ui_basic as basic;",
        source: include_str!("aspect_ratio.rs"),
        align: PreviewAlign::Center,
        examples: vec![
            Example::new(
                "wide",
                "16 / 9",
                "画像を 16:9 の箱に収めます。`ImageFit::Cover` で、はみ出す部分を箱が切り取ります。",
                wide,
            ),
            Example::new(
                "square",
                "Square",
                "比率に 1.0 を渡すと正方形になります。同じ画像が中央で切り取られます。",
                square,
            ),
            Example::new(
                "portrait",
                "Portrait",
                "1 未満の比率は縦長の箱になります。中身は画像でなくてもかまいません。",
                portrait,
            ),
        ],
    }
}

// example: wide
fn wide() -> impl IntoView {
    let photo = Asset::from(include_bytes!("../../assets/images/landscape.png").as_slice());
    column(basic::aspect_ratio(
        16.0 / 9.0,
        image(photo)
            .fit(ImageFit::Cover)
            .name("Mountains under a pale sun")
            .style(style().size(pct(100), pct(100))),
    ))
    .style(style().width(450))
}
// end example

// example: square
fn square() -> impl IntoView {
    let photo = Asset::from(include_bytes!("../../assets/images/landscape.png").as_slice());
    column(basic::aspect_ratio(
        1.0,
        image(photo)
            .fit(ImageFit::Cover)
            .name("Mountains under a pale sun")
            .style(style().size(pct(100), pct(100))),
    ))
    .style(style().width(200))
}
// end example

// example: portrait
fn portrait() -> impl IntoView {
    column(basic::aspect_ratio(
        3.0 / 4.0,
        column(text("3 / 4")).style(
            style()
                .height(pct(100))
                .align_items(Align::Center)
                .justify_content(Justify::Center)
                .background(ColorToken::SurfaceAlt),
        ),
    ))
    .style(style().width(150))
}
// end example
