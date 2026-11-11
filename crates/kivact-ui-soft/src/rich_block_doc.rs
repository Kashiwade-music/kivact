//! Role: Preserve Markdown source ranges while editing visual blocks.
use crate::markdown_parser::{
    begins_block, fence, heading, is_rule, list_marker, quote, reference, table_separator, task,
};
use kivact::view::NodeRef;

/// The editor that the formatting commands act on.
#[derive(Clone, Copy, PartialEq)]
pub(super) enum EditTarget {
    /// No editor accepts commands.
    Detached,
    /// This editor accepts formatting.
    Ready(NodeRef),
    /// This editor is composing input; formatting waits.
    Composing(NodeRef),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Kind {
    Paragraph,
    Heading(u32),
    Quote,
    List {
        // xtask-allow(state-fields): independent: parsed from the `N.` marker and incremented on split; a task checkbox may sit on ordered or bullet items, so every number and checked pair comes from valid source.
        number: Option<u32>,
        // xtask-allow(state-fields): independent: parsed from the `[ ]` / `[x]` prefix and toggled only by Document::task, which rewrites the head text in the same step; numbering never constrains it.
        checked: Option<bool>,
    },
    Code(String),
    Preview,
}

impl Kind {
    pub(super) fn editable(&self) -> bool {
        *self != Self::Preview
    }
    pub(super) fn prose(&self) -> bool {
        !matches!(self, Self::Code(_) | Self::Preview)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Block {
    pub(super) id: usize,
    pub(super) kind: Kind,
    head: String,
    pub(super) body: String,
    end: String,
    tail: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Document {
    leading: String,
    pub(super) blocks: Vec<Block>,
    next: usize,
    newline: &'static str,
}

impl Document {
    pub(super) fn parse(source: &str, first: usize) -> Self {
        let newline = if source.contains("\r\n") {
            "\r\n"
        } else {
            "\n"
        };
        let mut doc = Self {
            leading: String::new(),
            blocks: Vec::new(),
            next: first,
            newline,
        };
        let lines = source_lines(source);
        let mut index = 0;
        while index < lines.len() {
            let line = lines[index];
            if line.text.trim().is_empty() {
                let raw = &source[line.start..line.next];
                if let Some(block) = doc.blocks.last_mut() {
                    block.tail.push_str(raw);
                } else {
                    doc.leading.push_str(raw);
                }
                index += 1;
                continue;
            }
            let (kind, body_start, body_end, last) = block_span(&lines, index);
            let terminal = lines[last];
            let id = doc.fresh();
            doc.blocks.push(Block {
                id,
                kind,
                head: source[line.start..body_start].into(),
                body: source[body_start..body_end].into(),
                end: source[body_end..terminal.end.max(body_end)].into(),
                tail: source[terminal.end.max(body_end)..terminal.next].into(),
            });
            index = last + 1;
        }
        if doc.blocks.is_empty() {
            let id = doc.fresh();
            doc.blocks.push(Block {
                id,
                kind: Kind::Paragraph,
                head: String::new(),
                body: String::new(),
                end: String::new(),
                tail: String::new(),
            });
        }
        doc
    }

    fn fresh(&mut self) -> usize {
        let id = self.next;
        self.next += 1;
        id
    }

    pub(super) fn next_id(&self) -> usize {
        self.next
    }

    pub(super) fn markdown(&self) -> String {
        let mut source = self.leading.clone();
        for block in &self.blocks {
            source.push_str(&block.head);
            source.push_str(&block.body);
            source.push_str(&block.end);
            source.push_str(&block.tail);
        }
        source
    }

    pub(super) fn replace(&mut self, id: usize, body: String) {
        let Some(block) = self.blocks.iter_mut().find(|block| block.id == id) else {
            return;
        };
        if matches!(block.kind, Kind::Code(_))
            && block.body.is_empty()
            && !body.is_empty()
            && !block.end.is_empty()
            && !block.end.starts_with(['\r', '\n'])
        {
            block.end.insert_str(0, self.newline);
        }
        block.body = body;
    }

    pub(super) fn task(&mut self, id: usize, checked: bool) {
        let Some(block) = self.blocks.iter_mut().find(|block| block.id == id) else {
            return;
        };
        let Kind::List {
            checked: Some(current),
            ..
        } = &mut block.kind
        else {
            return;
        };
        if *current != checked
            && let Some(at) = block.head.rfind('[')
        {
            block
                .head
                .replace_range(at + 1..at + 2, if checked { "x" } else { " " });
            *current = checked;
        }
    }

    pub(super) fn split(&mut self, id: usize, selection: std::ops::Range<usize>) -> Option<usize> {
        let at = self.blocks.iter().position(|block| block.id == id)?;
        let left = &mut self.blocks[at];
        if !left.kind.prose() {
            return None;
        }
        let right_text = left.body.get(selection.end..)?.to_owned();
        left.body.get(..selection.start)?;
        left.body.truncate(selection.start);
        let (kind, head) = match &left.kind {
            Kind::Quote => (Kind::Quote, left.head.clone()),
            Kind::List { number, checked } => {
                let number = number.map(|n| n.saturating_add(1));
                let checked = checked.map(|_| false);
                let kind = Kind::List { number, checked };
                let mark = number.map_or_else(|| "- ".to_owned(), |number| format!("{number}. "));
                let head = if checked.is_some() {
                    format!("{mark}[ ] ")
                } else {
                    mark
                };
                (kind, head)
            }
            _ => (Kind::Paragraph, String::new()),
        };
        let separator = if matches!(kind, Kind::Quote | Kind::List { .. }) {
            self.newline.into()
        } else {
            self.newline.repeat(2)
        };
        let tail = std::mem::replace(&mut left.tail, separator);
        let id = self.fresh();
        self.blocks.insert(
            at + 1,
            Block {
                id,
                kind,
                head,
                body: right_text,
                end: String::new(),
                tail,
            },
        );
        Some(id)
    }

    pub(super) fn merge(&mut self, id: usize) -> Option<(usize, usize)> {
        let at = self.blocks.iter().position(|block| block.id == id)?;
        if at == 0 || !self.blocks[at].kind.prose() || !self.blocks[at - 1].kind.prose() {
            return None;
        }
        let right = self.blocks.remove(at);
        let left = &mut self.blocks[at - 1];
        let caret = left.body.len();
        left.body.push_str(&right.body);
        left.tail = right.tail;
        Some((left.id, caret))
    }

    pub(super) fn neighbor(&self, id: usize, forward: bool) -> Option<usize> {
        let at = self.blocks.iter().position(|block| block.id == id)?;
        if forward {
            self.blocks[at + 1..]
                .iter()
                .find(|block| block.kind.editable())
        } else {
            self.blocks[..at]
                .iter()
                .rev()
                .find(|block| block.kind.editable())
        }
        .map(|block| block.id)
    }
}

#[derive(Clone, Copy)]
struct Line<'a> {
    text: &'a str,
    start: usize,
    end: usize,
    next: usize,
}
impl Line<'_> {
    fn offset(self, slice: &str) -> usize {
        self.start + (slice.as_ptr() as usize - self.text.as_ptr() as usize)
    }
}

fn is_table(lines: &[Line<'_>], index: usize) -> bool {
    lines.get(index).is_some_and(|line| line.text.contains('|'))
        && lines
            .get(index + 1)
            .is_some_and(|line| table_separator(line.text).is_some())
}

fn source_lines(source: &str) -> Vec<Line<'_>> {
    let mut start = 0;
    source
        .split_inclusive('\n')
        .map(|raw| {
            let text = raw
                .strip_suffix('\n')
                .map(|text| text.strip_suffix('\r').unwrap_or(text))
                .unwrap_or(raw);
            let line = Line {
                text,
                start,
                end: start + text.len(),
                next: start + raw.len(),
            };
            start = line.next;
            line
        })
        .collect()
}

fn block_span(lines: &[Line<'_>], index: usize) -> (Kind, usize, usize, usize) {
    let line = lines[index];
    if let Some((ch, length, info)) = fence(line.text) {
        let closing = (index + 1..lines.len()).find(|&i| {
            fence(lines[i].text)
                .is_some_and(|(end, len, info)| end == ch && len >= length && info.is_empty())
        });
        let (end, last) = match closing {
            Some(last) if last > index + 1 => (lines[last - 1].end, last),
            Some(last) => (line.next, last),
            None => (lines[lines.len() - 1].end.max(line.next), lines.len() - 1),
        };
        (
            Kind::Code(info.split_whitespace().next().unwrap_or("").into()),
            line.next,
            end,
            last,
        )
    } else if let Some((level, body)) = heading(line.text) {
        let start = line.offset(body);
        (Kind::Heading(level), start, start + body.len(), index)
    } else if let Some(body) = quote(line.text) {
        (Kind::Quote, line.offset(body), line.end, index)
    } else if !is_rule(line.text)
        && let Some((_, number, body)) = list_marker(line.text)
    {
        let (checked, body) = task(body);
        (
            Kind::List { number, checked },
            line.offset(body),
            line.end,
            index,
        )
    } else if is_rule(line.text) || reference(line.text).is_some() {
        (Kind::Preview, line.start, line.end, index)
    } else if is_table(lines, index) {
        let mut last = index + 1;
        while lines
            .get(last + 1)
            .is_some_and(|line| line.text.contains('|') && !line.text.trim().is_empty())
        {
            last += 1;
        }
        (Kind::Preview, line.start, lines[last].end, last)
    } else {
        let mut last = index;
        while lines
            .get(last + 1)
            .is_some_and(|line| !line.text.trim().is_empty() && !begins_block(line.text))
            && !is_table(lines, last + 1)
        {
            last += 1;
        }
        (Kind::Paragraph, line.start, lines[last].end, last)
    }
}

#[cfg(test)]
mod tests {
    //! Role: Verify unedited bytes survive block edits and movement.
    use super::*;

    #[test]
    fn replacing_and_toggling_preserve_neighboring_source() {
        let source =
            "  ## Heading ##  \r\n\r\n> quote\r\n- [X] task  \r\n\r\n```rs\r\nx();\r\n```\r\n";
        let mut doc = Document::parse(source, 0);
        assert_eq!(doc.markdown(), source);
        doc.replace(0, "Changed".into());
        doc.task(2, false);
        assert_eq!(
            doc.markdown(),
            source.replace("Heading", "Changed").replace("[X]", "[ ]")
        );
    }

    #[test]
    fn split_and_merge_keep_markdown_boundaries_and_byte_carets() {
        let mut doc = Document::parse("Oneé two\r\n\r\n# unchanged\r\n", 0);
        let id = doc.split(0, 5..5).unwrap();
        assert_eq!(doc.markdown(), "Oneé\r\n\r\n two\r\n\r\n# unchanged\r\n");
        assert_eq!(doc.merge(id), Some((0, 5)));
        assert_eq!(doc.markdown(), "Oneé two\r\n\r\n# unchanged\r\n");
    }
}
