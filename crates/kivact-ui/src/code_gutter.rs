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

pub(super) fn gutter(
    editor: NodeRef,
    code: kivact_core::state::Value<String>,
    variants: &crate::variants::Variants,
    state: kivact_core::state::Value<crate::chat_values::CodeBlockState>,
) -> kivact_core::view::Node {
    use kivact_core::view::text;
    let geometry = editor.rect();
    crate::layer::with_recipe_style(
        text(move || {
            let _geometry = geometry.get();
            code.with(|source| numbers(editor, source))
        })
        .hidden(move || !state.get().numbered),
        |r| r.code_block_recipes().gutter_recipe(),
        variants,
        move || state.get(),
    )
    .style(move || {
        let _geometry = geometry.get();
        let _source = code.get();
        match editor.text_layout(0..2).and_then(|layout| {
            layout
                .lines
                .get(1)
                .map(|next| next.baseline - layout.lines[0].baseline)
        }) {
            Some(height) if height > 0.0 => {
                kivact_core::style::style().line_height(kivact_core::style::LineHeight::px(height))
            }
            _ => kivact_core::style::style(),
        }
    })
}
