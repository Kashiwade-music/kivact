//! Role: Parse Markdown blocks without adding dependencies to the kit.

#[derive(Debug)]
pub(crate) enum Block<'a> {
    Heading(u32, &'a str),
    Paragraph(String),
    Quote(Vec<Block<'a>>),
    Code(&'a str, String),
    List(Vec<ListEntry<'a>>),
    Table(Vec<Vec<&'a str>>, Vec<Alignment>),
    Rule,
}

#[derive(Debug)]
pub(crate) struct ListEntry<'a> {
    // xtask-allow(state-fields): option: ordinal parsed from the Markdown author's `N.` marker and incremented per sibling; None means a bullet item.
    pub(crate) number: Option<u32>,
    // xtask-allow(state-fields): option: task mark parsed from the author's `[ ]` / `[x]` prefix on ordered or bullet items alike; None means a plain item.
    pub(crate) checked: Option<bool>,
    pub(crate) blocks: Vec<Block<'a>>,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum Alignment {
    Start,
    Center,
    End,
}

pub(crate) fn parse<'a>(lines: &[&'a str], soft_breaks: bool, depth: usize) -> Vec<Block<'a>> {
    if depth >= 100 {
        return vec![Block::Paragraph(lines.join(" "))];
    }
    let mut result = Vec::new();
    let mut index = 0;
    while index < lines.len() {
        let line = lines[index];
        if line.trim().is_empty() || reference(line).is_some() {
            index += 1;
        } else if let Some((character, length, info)) = fence(line) {
            result.push(fenced_block(lines, &mut index, character, length, info));
        } else if let Some((level, title)) = heading(line) {
            result.push(Block::Heading(level, title));
            index += 1;
        } else if is_rule(line) {
            result.push(Block::Rule);
            index += 1;
        } else if quote(line).is_some() {
            result.push(quoted_block(lines, &mut index, soft_breaks, depth));
        } else if let Some((indent, number, _)) = list_marker(line) {
            result.push(list_block(
                lines,
                &mut index,
                soft_breaks,
                depth,
                indent,
                number,
            ));
        } else if line.contains('|')
            && let Some(align) = lines.get(index + 1).and_then(|line| table_separator(line))
        {
            let header = table_cells(line);
            if header.len() != align.len() {
                result.push(Block::Paragraph(line.to_owned()));
                index += 1;
                continue;
            }
            let mut rows = vec![header];
            index += 2;
            while index < lines.len()
                && lines[index].contains('|')
                && !lines[index].trim().is_empty()
            {
                let mut row = table_cells(lines[index]);
                row.resize(align.len(), "");
                rows.push(row);
                index += 1;
            }
            result.push(Block::Table(rows, align));
        } else {
            result.push(paragraph_block(lines, &mut index, soft_breaks));
        }
    }
    result
}

pub(crate) fn begins_block(line: &str) -> bool {
    heading(line).is_some()
        || fence(line).is_some()
        || is_rule(line)
        || quote(line).is_some()
        || list_marker(line).is_some()
        || reference(line).is_some()
}

fn limited_indent(line: &str) -> Option<&str> {
    let trimmed = line.trim_start_matches(' ');
    (line.len() - trimmed.len() <= 3).then_some(trimmed)
}

pub(crate) fn heading(line: &str) -> Option<(u32, &str)> {
    let line = limited_indent(line)?;
    let count = line.bytes().take_while(|byte| *byte == b'#').count();
    if !(1..=6).contains(&count) {
        return None;
    }
    let text = line.get(count..)?;
    if !text.is_empty() && !text.starts_with(char::is_whitespace) {
        return None;
    }
    let text = text.trim();
    let trailing = text.trim_end_matches('#');
    let text = if trailing.is_empty() || trailing.ends_with(char::is_whitespace) {
        trailing.trim_end()
    } else {
        text
    };
    Some((count as u32, text))
}

pub(crate) fn fence(line: &str) -> Option<(char, usize, &str)> {
    let line = limited_indent(line)?;
    let first = line.chars().next()?;
    if !matches!(first, '`' | '~') {
        return None;
    }
    let count = line.bytes().take_while(|byte| *byte == first as u8).count();
    (count >= 3).then(|| (first, count, line[count..].trim()))
}

pub(crate) fn quote(line: &str) -> Option<&str> {
    let content = limited_indent(line)?.strip_prefix('>')?;
    Some(content.strip_prefix(' ').unwrap_or(content))
}

pub(crate) fn is_rule(line: &str) -> bool {
    let Some(line) = limited_indent(line) else {
        return false;
    };
    let mut marks = line.chars().filter(|character| !character.is_whitespace());
    let Some(first @ ('-' | '*' | '_')) = marks.next() else {
        return false;
    };
    marks.clone().count() >= 2 && marks.all(|character| character == first)
}

pub(crate) fn list_marker(line: &str) -> Option<(usize, Option<u32>, &str)> {
    let trimmed = limited_indent(line)?;
    let indent = line.len() - trimmed.len();
    let first = trimmed.chars().next()?;
    if matches!(first, '-' | '*' | '+') && trimmed[1..].starts_with(char::is_whitespace) {
        return Some((indent, None, trimmed[1..].trim_start()));
    }
    let digits = trimmed.bytes().take_while(u8::is_ascii_digit).count();
    if !(1..=9).contains(&digits) {
        return None;
    }
    let rest = trimmed[digits..].strip_prefix(['.', ')'])?;
    if !rest.starts_with(char::is_whitespace) {
        return None;
    }
    Some((
        indent,
        Some(trimmed[..digits].parse().ok()?),
        rest.trim_start(),
    ))
}

pub(crate) fn task(content: &str) -> (Option<bool>, &str) {
    for (marker, checked) in [("[ ] ", false), ("[x] ", true), ("[X] ", true)] {
        if let Some(content) = content.strip_prefix(marker) {
            return (Some(checked), content);
        }
    }
    (None, content)
}

fn table_cells(line: &str) -> Vec<&str> {
    let line = line.trim();
    let line = line.strip_prefix('|').unwrap_or(line);
    let line = line.strip_suffix('|').unwrap_or(line);
    line.split('|').map(str::trim).collect()
}

pub(crate) fn table_separator(line: &str) -> Option<Vec<Alignment>> {
    table_cells(line)
        .into_iter()
        .map(|cell| {
            let dashes = cell.trim_matches(':');
            if dashes.is_empty() || !dashes.bytes().all(|byte| byte == b'-') {
                return None;
            }
            Some(match (cell.starts_with(':'), cell.ends_with(':')) {
                (true, true) => Alignment::Center,
                (false, true) => Alignment::End,
                _ => Alignment::Start,
            })
        })
        .collect()
}

pub(crate) fn reference(line: &str) -> Option<(&str, &str)> {
    let line = limited_indent(line)?.strip_prefix('[')?;
    let (label, url) = line.split_once("]:")?;
    let url = url.split_whitespace().next()?.trim_matches(['<', '>']);
    (!label.is_empty() && !url.is_empty()).then_some((label, url))
}

fn fenced_block<'a>(
    lines: &[&'a str],
    index: &mut usize,
    character: char,
    length: usize,
    info: &'a str,
) -> Block<'a> {
    let start = *index + 1;
    *index = start;
    while *index < lines.len()
        && !fence(lines[*index])
            .is_some_and(|(ch, len, rest)| ch == character && len >= length && rest.is_empty())
    {
        *index += 1;
    }
    let block = Block::Code(
        info.split_whitespace().next().unwrap_or(""),
        lines[start..*index].join("\n"),
    );
    *index = (*index + 1).min(lines.len());
    block
}

fn quoted_block<'a>(
    lines: &[&'a str],
    index: &mut usize,
    soft_breaks: bool,
    depth: usize,
) -> Block<'a> {
    let mut quoted = Vec::new();
    while *index < lines.len() {
        let line = lines[*index];
        if let Some(line) = quote(line) {
            quoted.push(line);
        } else if line.trim().is_empty() || begins_block(line) {
            break;
        } else {
            quoted.push(line);
        }
        *index += 1;
    }
    Block::Quote(parse(&quoted, soft_breaks, depth + 1))
}

fn list_block<'a>(
    lines: &[&'a str],
    index: &mut usize,
    soft_breaks: bool,
    depth: usize,
    indent: usize,
    number: Option<u32>,
) -> Block<'a> {
    let mut items = Vec::new();
    let mut ordinal = number;
    while let Some((_, _, content)) =
        lines
            .get(*index)
            .and_then(|line| list_marker(line))
            .filter(|(next_indent, number, _)| {
                *next_indent == indent && number.is_some() == ordinal.is_some()
            })
    {
        let (checked, content) = task(content);
        let mut body = vec![content];
        *index += 1;
        while let Some(line) = lines.get(*index) {
            let leading = line.len() - line.trim_start_matches(' ').len();
            if leading > indent {
                body.push(&line[(indent + 2).min(leading)..]);
            } else if line.trim().is_empty()
                && lines
                    .get(*index + 1)
                    .is_some_and(|next| next.len() - next.trim_start_matches(' ').len() > indent)
            {
                body.push("");
            } else {
                break;
            }
            *index += 1;
        }
        items.push(ListEntry {
            number: ordinal,
            checked,
            blocks: parse(&body, soft_breaks, depth + 1),
        });
        ordinal = ordinal.map(|number| number.saturating_add(1));
    }
    Block::List(items)
}

fn paragraph_block<'a>(lines: &[&'a str], index: &mut usize, soft_breaks: bool) -> Block<'a> {
    let mut paragraph = String::new();
    loop {
        let line = lines[*index];
        let hard = line.ends_with("  ") || line.ends_with('\\');
        paragraph.push_str(if let Some(line) = line.strip_suffix('\\') {
            line.trim_start()
        } else {
            line.trim()
        });
        *index += 1;
        if *index == lines.len()
            || lines[*index].trim().is_empty()
            || begins_block(lines[*index])
            || (lines[*index].contains('|')
                && lines
                    .get(*index + 1)
                    .is_some_and(|line| table_separator(line).is_some()))
        {
            break;
        }
        paragraph.push(if hard || soft_breaks { '\n' } else { ' ' });
    }
    Block::Paragraph(paragraph)
}
