//! Role: Compare retained editing chunks with a full split of the displayed text.

use crate::text_input::EditLines;
use crate::text_input::records::Before;

impl EditLines {
    pub(crate) fn discrepancies(&self, text: &str, engine: &mut arvane::Engine) -> Vec<String> {
        let mut errors = Vec::new();
        let expected = self.split_rows(text, engine);
        let mut prefix = Before::default();
        for (chunk_index, chunk) in self.chunks.iter().enumerate() {
            let actual = self.before.get(chunk_index);
            if actual.is_none_or(|actual| {
                (actual.line, actual.byte, actual.chars, actual.units)
                    != (prefix.line, prefix.byte, prefix.chars, prefix.units)
            }) {
                errors.push(format!(
                    "chunk {chunk_index} prefix: retained {actual:?}, rebuilt {prefix:?}"
                ));
            }
            let base = prefix.byte;
            for line in &chunk.lines {
                let retained = (base + line.start, base + line.content, base + line.end);
                let rebuilt = expected.get(prefix.line);
                if rebuilt != Some(&retained) {
                    errors.push(format!(
                        "line {} in chunk {chunk_index}: retained {retained:?}, rebuilt {rebuilt:?}",
                        prefix.line
                    ));
                }
                if let Some(&(start, _, end)) = rebuilt {
                    let source = &text[start..end];
                    let counts = (source.chars().count(), source.encode_utf16().count());
                    if (line.chars as usize, line.units as usize) != counts {
                        errors.push(format!(
                            "line {} scalar/UTF16 counts: retained ({}, {}), rebuilt {counts:?}",
                            prefix.line, line.chars, line.units
                        ));
                    }
                }
                prefix.line += 1;
            }
            let bytes = chunk.lines.last().map_or(0, |line| line.end);
            let chars: usize = chunk.lines.iter().map(|line| line.chars as usize).sum();
            let units: usize = chunk.lines.iter().map(|line| line.units as usize).sum();
            if (chunk.bytes, chunk.chars, chunk.units) != (bytes, chars, units) {
                errors.push(format!(
                    "chunk {chunk_index} totals: retained ({}, {}, {}), rebuilt ({bytes}, {chars}, {units})",
                    chunk.bytes, chunk.chars, chunk.units
                ));
            }
            prefix.byte += bytes;
            prefix.chars += chars;
            prefix.units += units;
        }
        if prefix.line != expected.len() {
            errors.push(format!(
                "line count: retained {}, rebuilt {}",
                prefix.line,
                expected.len()
            ));
        }
        let total = self.before.last();
        if total.is_none_or(|total| {
            (total.line, total.byte, total.chars, total.units)
                != (
                    prefix.line,
                    text.len(),
                    text.chars().count(),
                    text.encode_utf16().count(),
                )
        }) || self.before.len() != self.chunks.len() + 1
        {
            errors.push(format!(
                "chunk prefix totals: retained {total:?}, rebuilt {prefix:?}; text has {} bytes",
                text.len()
            ));
        }
        errors
    }
}
