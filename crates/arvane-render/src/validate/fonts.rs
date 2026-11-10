//! Role: Bounded successful font metadata with allocation lifetime protection.
use crate::error::RenderError;
use kivact_types::render::FontSource;
use skrifa::raw::TableProvider;
use std::sync::{Arc, Weak};
#[derive(Default)]
pub(crate) struct Fonts {
    entries: Vec<Entry>,
}
struct Entry {
    bytes: Weak<dyn AsRef<[u8]> + Send + Sync>,
    index: u32,
    glyphs: u32,
    len: usize,
}
impl Fonts {
    pub(crate) fn clear(&mut self) {
        self.entries.clear();
    }
    pub(super) fn glyphs(&mut self, bytes: &FontSource, index: u32) -> Result<u32, RenderError> {
        self.entries.retain(|entry| entry.bytes.strong_count() != 0);
        if let Some(at) = self.entries.iter().position(|entry| {
            entry.index == index
                && entry.bytes.as_ptr().cast::<u8>() as usize == bytes.id()
                && entry.len == bytes.len()
        }) {
            let entry = self.entries.remove(at);
            let glyphs = entry.glyphs;
            self.entries.push(entry);
            return Ok(glyphs);
        }
        let font =
            skrifa::FontRef::from_index(bytes, index).map_err(|_| RenderError::InvalidFont)?;
        let glyphs = u32::from(
            font.maxp()
                .map_err(|_| RenderError::InvalidFont)?
                .num_glyphs(),
        );
        if self.entries.len() == 256 {
            self.entries.remove(0);
        }
        self.entries.push(Entry {
            bytes: Arc::downgrade(bytes.source()),
            len: bytes.len(),
            index,
            glyphs,
        });
        Ok(glyphs)
    }
}
