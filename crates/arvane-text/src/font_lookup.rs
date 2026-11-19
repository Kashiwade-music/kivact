//! Role: Font faces for text drawn off the UI thread (SVG text).

use parley::fontique::{
    Blob, Collection, FontStyle, FontWeight, FontWidth, GenericFamily, SourceCache,
};
use std::sync::Arc;

/// A family as SVG names it.
#[derive(Clone, Copy, Debug)]
pub enum FamilyQuery<'a> {
    /// A family by name.
    Named(&'a str),
    /// `serif`.
    Serif,
    /// `sans-serif`.
    SansSerif,
    /// `monospace`.
    Monospace,
    /// `cursive`.
    Cursive,
    /// `fantasy`.
    Fantasy,
}

/// One face of a font file: the file's bytes and the face's index in it.
#[derive(Clone)]
pub struct FontFace {
    /// The whole font file.
    pub data: Arc<dyn AsRef<[u8]> + Send + Sync>,
    /// The face's index in the file (0 unless a collection).
    pub index: u32,
}

/// The system fonts and the App's fonts, for one thread that is not
/// the UI thread. Making it waits for the system font scan.
pub struct FontLookup {
    collection: Collection,
    sources: SourceCache,
}

impl FontLookup {
    /// The system fonts plus `app` (the App's font files, in registration order).
    pub fn new(app: &[kivact_types::shaped::FontSource]) -> FontLookup {
        let mut collection = crate::system_fonts::collection();
        for font in app {
            collection.register_fonts(Blob::new(Arc::clone(font.source())), None);
        }
        FontLookup {
            collection,
            sources: SourceCache::default(),
        }
    }

    /// The face of the first of `families` that is installed, matched to
    /// `weight` (100–900), `italic` and `width` (1.0 is normal).
    pub fn face(
        &mut self,
        families: &[FamilyQuery<'_>],
        weight: f32,
        italic: bool,
        width: f32,
    ) -> Option<FontFace> {
        let style = if italic {
            FontStyle::Italic
        } else {
            FontStyle::Normal
        };
        for family in families {
            let id = match family {
                FamilyQuery::Named(name) => self.collection.family_id(name),
                FamilyQuery::Serif => self.generic(GenericFamily::Serif),
                FamilyQuery::SansSerif => self.generic(GenericFamily::SansSerif),
                FamilyQuery::Monospace => self.generic(GenericFamily::Monospace),
                FamilyQuery::Cursive => self.generic(GenericFamily::Cursive),
                FamilyQuery::Fantasy => self.generic(GenericFamily::Fantasy),
            };
            let Some(info) = id.and_then(|id| self.collection.family(id)) else {
                continue;
            };
            let Some(font) = info.match_font(
                FontWidth::from_ratio(width),
                style,
                FontWeight::new(weight),
                true,
            ) else {
                continue;
            };
            let index = font.index();
            if let Some(blob) = font.load(Some(&mut self.sources)) {
                return Some(FontFace {
                    data: Arc::new(blob),
                    index,
                });
            }
        }
        None
    }

    /// Faces of the families SVG text falls back to when a character is
    /// missing: the first family of the generic sans-serif, system-ui,
    /// serif, monospace and emoji families, in this order.
    pub fn fallback_faces(&mut self) -> Vec<FontFace> {
        let generics = [
            GenericFamily::SansSerif,
            GenericFamily::SystemUi,
            GenericFamily::Serif,
            GenericFamily::Monospace,
            GenericFamily::Emoji,
        ];
        let mut faces = Vec::new();
        for generic in generics {
            let Some(id) = self.generic(generic) else {
                continue;
            };
            let Some(info) = self.collection.family(id) else {
                continue;
            };
            let Some(font) = info.default_font().cloned() else {
                continue;
            };
            let index = font.index();
            if let Some(blob) = font.load(Some(&mut self.sources)) {
                faces.push(FontFace {
                    data: Arc::new(blob),
                    index,
                });
            }
        }
        faces
    }

    /// The first installed family of a generic family.
    fn generic(&mut self, generic: GenericFamily) -> Option<parley::fontique::FamilyId> {
        self.collection.generic_families(generic).next()
    }
}
