//! Role: The font database of SVG `<text>`, built on the first SVG that has text.

use std::sync::{Arc, Mutex};

use resvg::usvg;

/// System fonts plus the App's fonts, loaded the first time an SVG with
/// text is parsed (on the asset worker thread), never at startup.
pub(crate) struct SvgFonts {
    /// The App's font files, in registration order.
    app: Vec<Arc<Vec<u8>>>,
    /// The database once built (a pure cache of `app` and the system fonts).
    db: Mutex<Option<Arc<usvg::fontdb::Database>>>,
}

impl SvgFonts {
    /// No App fonts yet; nothing is loaded.
    pub(crate) fn new() -> Self {
        Self {
            app: Vec::new(),
            db: Mutex::new(None),
        }
    }

    /// The same App fonts plus `bytes`; the database is built again on its next use.
    pub(crate) fn with_font(&self, bytes: Vec<u8>) -> Self {
        let mut app = self.app.clone();
        app.push(Arc::new(bytes));
        Self {
            app,
            db: Mutex::new(None),
        }
    }

    /// The database for `svg`: an empty one when the document has no text
    /// element, else the system and App fonts (built on the first call).
    pub(crate) fn for_document(&self, svg: &[u8]) -> Arc<usvg::fontdb::Database> {
        if !has_text(svg) {
            return Arc::new(usvg::fontdb::Database::new());
        }
        let mut db = self
            .db
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        Arc::clone(db.get_or_insert_with(|| {
            let mut fonts = usvg::fontdb::Database::new();
            fonts.load_system_fonts();
            for font in &self.app {
                fonts.load_font_data(font.as_ref().clone());
            }
            Arc::new(fonts)
        }))
    }
}

/// Whether the SVG may draw text with these fonts: a text element (`<text`,
/// or a prefixed `:text`), or an embedded SVG image (a `data:` URL of type
/// `image/svg+xml`, base64 or percent-encoded, whose inner document uses
/// the same font database). A false positive only loads the fonts.
fn has_text(svg: &[u8]) -> bool {
    let has = |needle: &[u8]| {
        svg.windows(needle.len())
            .any(|w| w.eq_ignore_ascii_case(needle))
    };
    has(b"<text") || has(b":text") || has(b"svg+xml") || has(b"svg%2bxml")
}
