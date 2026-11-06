//! Role: Incrementally reconcile one explicitly borrowed editor source.
use crate::measure::editor::EditorNode;
use crate::measure::state::MeasureColumns;
use crate::text_input::{EditLines, RefreshError, RefreshMode, Source};
use crate::tree::NodeTree;
use kivact_types::NodeId;
use std::cell::RefMut;

/// The width available to the editor during measurement or painting.
#[derive(Clone, Copy)]
pub(crate) enum LineViewport {
    Measure(Option<f64>),
    Paint(f64),
}
/// One captured source/style reconciliation, borrowing the retained cache once.
pub(crate) struct RefreshedLines<'a> {
    pub(crate) lines: RefMut<'a, EditLines>,
    pub(crate) source: Source<'a>,
    #[cfg(all(debug_assertions, feature = "testing"))]
    pub(crate) style: Option<kivact_types::ComputedTextStyle>,
}

/// The input's incrementally reconciled lines, borrowed once.
pub(crate) fn edit_lines<'a>(
    tree: &'a NodeTree,
    columns: &MeasureColumns,
    editors: &'a [Option<Box<EditorNode>>],
    id: NodeId,
    viewport: LineViewport,
    engine: &mut arvane::Engine,
) -> Option<(RefMut<'a, EditLines>, &'a str)> {
    let refreshed = refresh_lines(
        tree,
        columns,
        editors,
        id,
        viewport,
        engine,
        RefreshMode::Incremental,
    )
    .expect("invariant: incremental refresh performs no integrity verification")?;
    let text = refreshed
        .lines
        .shown(refreshed.source.value, refreshed.source.placeholder);
    Some((refreshed.lines, text))
}

/// Reconcile one source snapshot with exclusive access to its line cache.
pub(crate) fn refresh_lines<'a>(
    tree: &'a NodeTree,
    columns: &MeasureColumns,
    editors: &'a [Option<Box<EditorNode>>],
    id: NodeId,
    viewport: LineViewport,
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
    let wrap = n.edit_line_layout().wrap;
    let measurement = wrap && matches!(viewport, LineViewport::Measure(_));
    let width = match viewport {
        LineViewport::Measure(width) => width,
        LineViewport::Paint(width) => Some(width),
    };
    let cache = if measurement {
        &node.measure_lines
    } else {
        &node.lines
    };
    let mut slot = cache
        .try_borrow_mut()
        .expect("invariant: editor lines are borrowed once per refresh");
    let source = Source {
        value: edit.value(),
        revision: editor.revision(),
        damage: if measurement {
            None
        } else {
            slot.as_mut().and_then(|lines| lines.damage.take())
        },
        placeholder: n.placeholder(),
        password: n.password(),
        multiline: n.multiline(),
        wrap_width: width.filter(|width| wrap && width.is_finite() && *width > 0.0),
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
