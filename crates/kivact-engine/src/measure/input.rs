//! Role: Incrementally reconcile one explicitly borrowed editor source.
use crate::measure::editor::EditorNode;
use crate::measure::state::MeasureColumns;
use crate::tree::NodeTree;
use kivact_text::{EditLines, RefreshError, RefreshMode, Source};
use kivact_wire::NodeId;
use std::cell::RefMut;
/// One captured source/style reconciliation, borrowing the retained cache once.
pub(crate) struct RefreshedLines<'a> {
    pub(crate) lines: RefMut<'a, EditLines>,
    pub(crate) source: Source<'a>,
    #[cfg(all(debug_assertions, feature = "testing"))]
    pub(crate) style: Option<kivact_types::protocol::TextStyle>,
}

/// The input's incrementally reconciled lines, unless already borrowed.
pub(crate) fn edit_lines<'a>(
    tree: &'a NodeTree,
    columns: &MeasureColumns,
    editors: &'a [Option<Box<EditorNode>>],
    id: NodeId,
    engine: &mut arvane::Engine,
) -> Option<(RefMut<'a, EditLines>, &'a str)> {
    let refreshed = refresh_lines(tree, columns, editors, id, engine, RefreshMode::Incremental)
        .expect("invariant: incremental refresh performs no integrity verification")?;
    let text = refreshed
        .lines
        .shown(refreshed.source.value, refreshed.source.placeholder);
    Some((refreshed.lines, text))
}

/// Reconcile one source snapshot; inability to borrow consumes no damage.
pub(crate) fn refresh_lines<'a>(
    tree: &'a NodeTree,
    columns: &MeasureColumns,
    editors: &'a [Option<Box<EditorNode>>],
    id: NodeId,
    engine: &mut arvane::Engine,
    mode: RefreshMode,
) -> Result<Option<RefreshedLines<'a>>, RefreshError> {
    let Some(n) = tree.get(id) else {
        return Ok(None);
    };
    let Some(node) = editors[n.index.0].as_deref() else {
        return Ok(None);
    };
    let editor = &node.editor;
    let edit = editor.state();
    let style = crate::measure::input_style(tree, columns, id, engine);
    let Ok(mut slot) = node.lines.try_borrow_mut() else {
        return Ok(None);
    };
    let source = Source {
        value: edit.value(),
        revision: editor.revision(),
        damage: editor.take_damage(),
        placeholder: n.placeholder(),
        password: n.password(),
        multiline: n.multiline(),
    };
    #[cfg(all(debug_assertions, feature = "testing"))]
    let reference_style = (mode == RefreshMode::Verify).then(|| style.clone());
    if slot.is_none() {
        *slot = Some(EditLines::new(source, style.clone(), engine));
    }
    let mut lines = RefMut::map(slot, |slot| {
        slot.as_mut().expect("invariant: the lines were made above")
    });
    lines.refresh(source, style, engine, mode)?;
    Ok(Some(RefreshedLines {
        lines,
        source,
        #[cfg(all(debug_assertions, feature = "testing"))]
        style: reference_style,
    }))
}
