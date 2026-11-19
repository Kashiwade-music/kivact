//! Role: Glyph counts of the fonts scenes draw with, read once per font face.

use crate::error::RenderError;
use kivact_types::shaped::FontSource;
use skrifa::raw::TableProvider;
use std::collections::HashMap;
use std::sync::{Arc, Weak};

/// The most font faces remembered; past it the table starts again.
const MAX_FACES: usize = 256;

/// The glyph count of each font face drawn lately, by the address and
/// length of its bytes and its index in the file.
#[derive(Default)]
pub(crate) struct Fonts {
    entries: HashMap<(usize, usize, u32), Entry>,
}

struct Entry {
    /// A face whose bytes were freed is pruned before its address can be
    /// reused by other bytes.
    bytes: Weak<dyn AsRef<[u8]> + Send + Sync>,
    glyphs: u32,
}

impl Fonts {
    /// Forget every face.
    pub(crate) fn clear(&mut self) {
        self.entries.clear();
    }

    /// Forget the faces whose bytes were freed (once per frame scope).
    pub(crate) fn prune(&mut self) {
        self.entries
            .retain(|_, entry| entry.bytes.strong_count() != 0);
    }

    /// The glyph count of face `index` of `bytes`.
    pub(super) fn glyphs(&mut self, bytes: &FontSource, index: u32) -> Result<u32, RenderError> {
        let key = (bytes.id(), bytes.len(), index);
        if let Some(entry) = self.entries.get(&key)
            && entry.bytes.strong_count() != 0
        {
            return Ok(entry.glyphs);
        }
        let glyphs = read_glyph_count(bytes, index)?;
        if self.entries.len() >= MAX_FACES {
            self.prune();
            if self.entries.len() >= MAX_FACES {
                self.entries.clear();
            }
        }
        self.entries.insert(
            key,
            Entry {
                bytes: Arc::downgrade(bytes.source()),
                glyphs,
            },
        );
        Ok(glyphs)
    }
}

/// The glyph count of face `index` of `bytes`, read from the font file.
fn read_glyph_count(bytes: &FontSource, index: u32) -> Result<u32, RenderError> {
    let font = skrifa::FontRef::from_index(bytes, index).map_err(|_| RenderError::InvalidFont)?;
    let glyphs = u32::from(
        font.maxp()
            .map_err(|_| RenderError::InvalidFont)?
            .num_glyphs(),
    );
    Ok(glyphs)
}
