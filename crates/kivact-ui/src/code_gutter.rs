//! Role: Project source line numbers onto the native editor's shaped visual rows.
use kivact_core::custom::NodeRefExt;
use kivact_core::view::NodeRef;
pub(super) fn numbers(editor: NodeRef, source: &str) -> String {
    let Some(layout) = editor.text_layout(0..usize::MAX) else {
        return (1..=source.split('\n').count())
            .map(|n| n.to_string())
            .collect::<Vec<_>>()
            .join("\n");
    };
    let mut number = 1;
    let mut logical_start = 0;
    let mut rows = Vec::with_capacity(layout.lines.len());
    for line in layout.lines {
        rows.push(if line.range.start == logical_start {
            number.to_string()
        } else {
            String::new()
        });
        if source
            .get(line.range.clone())
            .is_some_and(|line| line.ends_with('\n'))
        {
            number += 1;
            logical_start = line.range.end;
        }
    }
    rows.join("\n")
}
