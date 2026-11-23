//! Role: The fonts SVG `<text>` is turned into paths with.

use arvane_text::{FamilyQuery, FontFace, FontLookup};
use kivact_types::shaped::FontSource;
use resvg::usvg;
use resvg::usvg::fontdb;
use std::sync::{Arc, Mutex};

/// The App's font files for SVG text, in registration order. The system
/// fonts come from arvane-text's process-wide scan when a document has text.
pub(crate) struct SvgFonts {
    app: Vec<FontSource>,
}

impl SvgFonts {
    /// No App fonts yet.
    pub(crate) fn new() -> Self {
        Self { app: Vec::new() }
    }

    /// The same App fonts plus `font`.
    pub(crate) fn with_font(&self, font: FontSource) -> Self {
        let mut app = self.app.clone();
        app.push(font);
        Self { app }
    }

    /// The font resolver usvg uses for `svg`: no font when the document
    /// has no text, else the system and App fonts, loaded into usvg's
    /// database face by face as the text asks for them.
    pub(crate) fn resolver(&self, svg: &[u8]) -> usvg::FontResolver<'static> {
        if !has_text(svg) {
            return usvg::FontResolver {
                select_font: Box::new(|_, _| None),
                select_fallback: Box::new(|_, _, _| None),
            };
        }
        let lookup = Arc::new(Mutex::new(FontLookup::new(&self.app)));
        let fallback = Arc::clone(&lookup);
        usvg::FontResolver {
            select_font: Box::new(move |font, db| {
                let families: Vec<FamilyQuery<'_>> = font.families().iter().map(family).collect();
                let italic = font.style() != usvg::FontStyle::Normal;
                let face = lock(&lookup).face(
                    &families,
                    f32::from(font.weight()),
                    italic,
                    width(font.stretch()),
                )?;
                load(db, &face)
            }),
            select_fallback: Box::new(move |c, used, db| {
                let default = usvg::FontResolver::default_fallback_selector();
                if let Some(id) = default(c, used, db) {
                    return Some(id);
                }
                for face in lock(&fallback).fallback_faces() {
                    load(db, &face);
                }
                default(c, used, db)
            }),
        }
    }
}

fn lock(lookup: &Mutex<FontLookup>) -> std::sync::MutexGuard<'_, FontLookup> {
    lookup
        .lock()
        .expect("invariant: the SVG font lookup lives on one parse and never panics while held")
}

fn family(family: &usvg::FontFamily) -> FamilyQuery<'_> {
    match family {
        usvg::FontFamily::Serif => FamilyQuery::Serif,
        usvg::FontFamily::SansSerif => FamilyQuery::SansSerif,
        usvg::FontFamily::Cursive => FamilyQuery::Cursive,
        usvg::FontFamily::Fantasy => FamilyQuery::Fantasy,
        usvg::FontFamily::Monospace => FamilyQuery::Monospace,
        usvg::FontFamily::Named(name) => FamilyQuery::Named(name),
    }
}

/// CSS `font-stretch` as a width ratio.
fn width(stretch: usvg::FontStretch) -> f32 {
    match stretch {
        usvg::FontStretch::UltraCondensed => 0.5,
        usvg::FontStretch::ExtraCondensed => 0.625,
        usvg::FontStretch::Condensed => 0.75,
        usvg::FontStretch::SemiCondensed => 0.875,
        usvg::FontStretch::Normal => 1.0,
        usvg::FontStretch::SemiExpanded => 1.125,
        usvg::FontStretch::Expanded => 1.25,
        usvg::FontStretch::ExtraExpanded => 1.5,
        usvg::FontStretch::UltraExpanded => 2.0,
    }
}

/// Load `face` into usvg's database and give its id there (a font
/// collection's faces are loaded in index order).
fn load(db: &mut Arc<fontdb::Database>, face: &FontFace) -> Option<fontdb::ID> {
    let ids = Arc::make_mut(db).load_font_source(fontdb::Source::Binary(Arc::clone(&face.data)));
    ids.get(face.index as usize).copied()
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

#[cfg(test)]
mod tests {
    //! Role: Tests of the SVG font resolver and its registered faces.
    use super::*;

    #[test]
    fn svg_text_uses_a_registered_font_without_a_system_font_database() {
        let tuffy: std::sync::Arc<[u8]> =
            std::sync::Arc::from(&include_bytes!("../../../arvane/assets/fonts/Tuffy.ttf")[..]);
        let fonts = SvgFonts::new().with_font(kivact_types::shaped::FontSource::from_shared(tuffy));
        let svg = br#"<svg xmlns="http://www.w3.org/2000/svg" width="40" height="20"><text x="0" y="15" font-family="Tuffy" font-size="12">Ab</text></svg>"#;
        let options = usvg::Options {
            font_resolver: fonts.resolver(svg),
            ..usvg::Options::default()
        };
        let tree = usvg::Tree::from_data(svg, &options).expect("valid SVG");
        assert!(tree.root().has_children(), "the text became paths");
        assert!(
            tree.fontdb()
                .faces()
                .all(|face| matches!(face.source, usvg::fontdb::Source::Binary(_))),
            "no font was read from the system font directories"
        );
    }

    #[test]
    fn an_svg_without_text_loads_no_font() {
        let svg = br#"<svg xmlns="http://www.w3.org/2000/svg" width="4" height="4"><rect width="4" height="4"/></svg>"#;
        let options = usvg::Options {
            font_resolver: SvgFonts::new().resolver(svg),
            ..usvg::Options::default()
        };
        let tree = usvg::Tree::from_data(svg, &options).expect("valid SVG");
        assert_eq!(tree.fontdb().len(), 0);
    }
}
