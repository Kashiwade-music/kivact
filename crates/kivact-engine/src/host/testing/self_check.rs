//! Role: Run debug reference comparisons for a settled test host.

use super::ArvaneTestHost;

impl ArvaneTestHost {
    /// Compare retained engine results with complete reference computations.
    ///
    /// Called after settling by `Screen`; errors become its normal failure
    /// evidence, including the node and retained/reference values. Debug only.
    pub fn self_check(&mut self) -> Vec<String> {
        if std::env::var_os("KIVACT_SELF_CHECK").is_some_and(|value| value == "0") {
            return Vec::new();
        }
        let mut model = self.host.model.borrow_mut();
        let mut engine = self.host.engine.borrow_mut();
        let mut errors = model.layout_self_check(&mut engine);
        errors.extend(model.paint_self_check());
        errors.extend(model.text_self_check());
        for (id, node) in model.tree.nodes.iter() {
            let caret = model.editors[node.index.0]
                .as_deref()
                .map(|editor| editor.editor.state())
                .map_or(0, |edit| edit.caret());
            let checked = model.refresh_lines(*id, &mut engine, kivact_text::RefreshMode::Verify);
            let discrepancies = match checked {
                Err(error) => error.discrepancies,
                Ok(Some(refreshed)) => {
                    super::text_reference::compare(refreshed, caret, &mut engine)
                }
                Ok(None) => Vec::new(),
            };
            errors.extend(
                discrepancies
                    .into_iter()
                    .map(|error| format!("EditLines node {}: {error}", id.0)),
            );
        }
        // A rejected publication deliberately leaves the last accepted
        // platform tree behind the model until a subsequent successful retry.
        if model.a11y.publication_pending() {
            return errors;
        }
        let published = self.window.accessibility().tree();
        // The independent build works on copies; the originals are put back after it.
        let independent = model.a11y.clone();
        let state = std::mem::replace(&mut model.a11y, independent);
        let lines: Vec<_> = model
            .editors
            .iter_mut()
            .enumerate()
            .filter_map(|(slot, node)| {
                let node = node.as_mut()?;
                let independent = node.lines.borrow().clone();
                Some((slot, std::mem::replace(node.lines.get_mut(), independent)))
            })
            .collect();
        let rebuilt = Some(model.build_semantics(&mut engine));
        model.a11y = state;
        for (slot, original) in lines {
            *model.editors[slot]
                .as_mut()
                .expect("invariant: reference does not remove editors")
                .lines
                .get_mut() = original;
        }
        if published != rebuilt {
            match (&published, &rebuilt) {
                (Some(published), Some(rebuilt)) => {
                    if published.root != rebuilt.root || published.focus != rebuilt.focus {
                        errors.push(format!(
                            "accessibility root/focus: published {:?}/{:?}, rebuilt {:?}/{:?}",
                            published.root, published.focus, rebuilt.root, rebuilt.focus
                        ));
                    }
                    // Compare reading order via child links: the builder emits a
                    // region leaf before primary children, while the shell canonicalizes.
                    let index: std::collections::BTreeMap<_, _> =
                        rebuilt.nodes.iter().map(|n| (n.id, n)).collect();
                    let mut order = Vec::new();
                    let mut stack = vec![rebuilt.root];
                    while let Some(id) = stack.pop() {
                        order.push(id);
                        if let Some(node) = index.get(&id) {
                            stack.extend(node.children.iter().rev().copied());
                        }
                    }
                    if order != published.nodes.iter().map(|n| n.id).collect::<Vec<_>>() {
                        errors.push("accessibility canonical reading order differs".to_string());
                    }
                    let before: std::collections::BTreeMap<_, _> = published
                        .nodes
                        .iter()
                        .map(|node| (node.id.0, node))
                        .collect();
                    let after: std::collections::BTreeMap<_, _> =
                        rebuilt.nodes.iter().map(|node| (node.id.0, node)).collect();
                    let ids: std::collections::BTreeSet<_> =
                        before.keys().chain(after.keys()).copied().collect();
                    for id in ids {
                        let before = before.get(&id);
                        let after = after.get(&id);
                        if before != after {
                            errors.push(format!(
                                "accessibility node {id}: published {before:?}, rebuilt {after:?}"
                            ));
                        }
                    }
                }
                _ => errors.push(format!(
                    "accessibility tree presence: published {}, rebuilt {}",
                    published.is_some(),
                    rebuilt.is_some()
                )),
            }
        }
        errors
    }
}
