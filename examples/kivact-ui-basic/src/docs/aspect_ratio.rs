//! Catalog page of Aspect Ratio: a photo at 16:9, square and portrait boxes.

use catalog_shell::{Example, Page};
use kivact::prelude::*;
use kivact::view::Asset;
use kivact::view::ImageFit;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page::new("aspect-ratio", "Aspect Ratio", "Components", include_str!("aspect_ratio.rs"))
        .description("A box of the full available width with the given aspect ratio (width / height). Its child grows to fill the box.")
        .usage("use kivact::prelude::*;
use kivact_ui_basic as basic;")
        .example(Example::new(
            "wide",
            "16 / 9",
            "Fit an image in a 16:9 box. With ImageFit::Cover the box cuts off the part that overflows.",
            wide,
        ))
        .example(Example::new(
            "square",
            "Square",
            "A ratio of 1.0 gives a square. The same image is cut to the center.",
            square,
        ))
        .example(Example::new(
            "portrait",
            "Portrait",
            "A ratio below 1 gives a tall box. The content need not be an image.",
            portrait,
        ))
}

// example: wide
fn wide() -> impl IntoView {
    let photo = Asset::from(include_bytes!("../../assets/images/landscape.png").as_slice());
    column(
        node()
            .style(
                style()
                    .flex_direction(kivact::style::FlexDirection::Column)
                    .width(pct(100))
                    .aspect_ratio(16.0 / 9.0)
                    .overflow(kivact::style::Overflow::Hidden),
            )
            .child(
                image(photo)
                    .fit(ImageFit::Cover)
                    .name("Mountains under a pale sun")
                    .style(style().size(pct(100), pct(100))),
            ),
    )
    .style(style().width(450))
}
// end example

// example: square
fn square() -> impl IntoView {
    let photo = Asset::from(include_bytes!("../../assets/images/landscape.png").as_slice());
    column(
        node()
            .style(
                style()
                    .flex_direction(kivact::style::FlexDirection::Column)
                    .width(pct(100))
                    .aspect_ratio(1.0)
                    .overflow(kivact::style::Overflow::Hidden),
            )
            .child(
                image(photo)
                    .fit(ImageFit::Cover)
                    .name("Mountains under a pale sun")
                    .style(style().size(pct(100), pct(100))),
            ),
    )
    .style(style().width(200))
}
// end example

// example: portrait
fn portrait() -> impl IntoView {
    column(
        node()
            .style(
                style()
                    .flex_direction(kivact::style::FlexDirection::Column)
                    .width(pct(100))
                    .aspect_ratio(3.0 / 4.0)
                    .overflow(kivact::style::Overflow::Hidden),
            )
            .child(
                column(text("3 / 4")).style(
                    style()
                        .height(pct(100))
                        .align_items(Align::Center)
                        .justify_content(Justify::Center)
                        .background(basic::MUTED),
                ),
            ),
    )
    .style(style().width(150))
}
// end example
