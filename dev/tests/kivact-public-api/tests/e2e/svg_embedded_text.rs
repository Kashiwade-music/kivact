//! Role: Tests that text of an SVG embedded in another SVG is drawn.
//!
//! The outer document has no text of its own: it only holds an
//! `<image href="data:image/svg+xml,..">` whose inner document draws a large
//! text. The font database of the outer SVG must be the one the inner parse
//! uses, or the text silently disappears (4.1 節, R06).

use kivact::prelude::*;
use kivact::test::render;
use kivact::view::Asset;
use kivact::view::ImageFit;

fn percent_encode(text: &str) -> String {
    let mut encoded = String::new();
    for byte in text.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                encoded.push(char::from(byte));
            }
            _ => encoded.push_str(&format!("%{byte:02X}")),
        }
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
         <image width=\"240\" height=\"120\" href=\"data:image/svg+xml;utf8,{}\"/>\
         </svg>",
        percent_encode(&inner)
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
