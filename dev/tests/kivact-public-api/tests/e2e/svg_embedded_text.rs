//! Role: Tests that text of an SVG embedded in another SVG is drawn.
//!
//! The outer document has no text of its own: it only holds an
//! `<image href="data:image/svg+xml;base64,..">` whose inner document draws a
//! large text. The font database of the outer SVG must be the one the inner
//! parse uses, or the text silently disappears (4.1 節, R06).

use kivact::prelude::*;
use kivact::test::render;
use kivact::view::Asset;
use kivact::view::ImageFit;

/// The standard base64 alphabet, padded as a data URL needs.
fn base64(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut encoded = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let second = chunk.get(1).copied().unwrap_or(0);
        let third = chunk.get(2).copied().unwrap_or(0);
        let bits = (u32::from(chunk[0]) << 16) | (u32::from(second) << 8) | u32::from(third);
        encoded.push(char::from(ALPHABET[(bits >> 18) as usize & 63]));
        encoded.push(char::from(ALPHABET[(bits >> 12) as usize & 63]));
        encoded.push(if chunk.len() > 1 {
            char::from(ALPHABET[(bits >> 6) as usize & 63])
        } else {
            '='
        });
        encoded.push(if chunk.len() > 2 {
            char::from(ALPHABET[bits as usize & 63])
        } else {
            '='
        });
    }
    encoded
}

fn svg_with_embedded_text(text: &str) -> Vec<u8> {
    let inner = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"240\" height=\"120\">\
         <rect width=\"240\" height=\"120\" fill=\"#ffffff\"/>\
         <text x=\"12\" y=\"80\" font-family=\"sans-serif\" font-size=\"64\" fill=\"#000000\">{text}</text>\
         </svg>"
    );
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"240\" height=\"120\">\
         <image width=\"240\" height=\"120\" href=\"data:image/svg+xml;base64,{}\"/>\
         </svg>",
        base64(inner.as_bytes())
    )
    .into_bytes()
}

#[test]
fn text_inside_an_svg_embedded_as_a_data_url_is_drawn() {
    let render_svg = |text: &str| {
        let bytes = svg_with_embedded_text(text);
        render(move || {
            image(Asset::from(bytes))
                .fit(ImageFit::Fill)
                .style(style().size(240, 120))
        })
    };
    let with_text = render_svg("EMBEDDED");
    let without_text = render_svg("");
    assert!(with_text.errors().is_empty(), "{:?}", with_text.errors());
    assert!(
        without_text.errors().is_empty(),
        "{:?}",
        without_text.errors()
    );
    let difference = with_text
        .screenshot()
        .raster_difference(&without_text.screenshot());
    assert!(
        difference.mean > 0.0,
        "the embedded text drew no pixels: {difference:?}"
    );
    assert!(
        difference.far > 0.0,
        "the embedded text left no strongly different pixels: {difference:?}"
    );
}
