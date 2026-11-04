//! Role: Validate declared accessible text runs and selection geometry.

/// Why `text` cannot be published as a node's text runs, if it cannot: at
/// least one run, finite non-negative rectangles, carets inside their runs
/// and a selection and caret inside the text.
pub fn check_text(text: &crate::SemanticsText) -> Result<(), &'static str> {
    let finite = |r: crate::Rect| {
        [r.x, r.y, r.width, r.height, r.x + r.width, r.y + r.height]
            .into_iter()
            .all(f32::is_finite)
            && r.width >= 0.0
            && r.height >= 0.0
    };
    if text.runs.is_empty() {
        return Err("it has no run (empty text is one empty run)");
    }
    if !finite(text.caret) {
        return Err("its caret rectangle is not finite or has a negative size");
    }
    let mut length = 0u32;
    for run in text.runs.iter() {
        let count = u32::try_from(run.value.chars().count()).map_err(|_| "a run is too long")?;
        if !finite(run.bounds) || run.carets.iter().any(|c| !finite(c.rect)) {
            return Err("a run's rectangle is not finite or has a negative size");
        }
        if run.carets.iter().any(|c| c.offset > count) {
            return Err("a caret lies past the end of its run");
        }
        length = length.checked_add(count).ok_or("the text is too long")?;
    }
    let end = text
        .start
        .checked_add(length)
        .ok_or("the text is too long")?;
    let inside = |at: u32| (text.start..=end).contains(&at);
    if !inside(text.selection.0) || !inside(text.selection.1) {
        return Err("the selection lies outside the runs");
    }
    let mut ids = std::collections::HashSet::new();
    if !text.runs.iter().all(|run| ids.insert(run.id)) {
        return Err("two runs have the same id");
    }
    Ok(())
}
