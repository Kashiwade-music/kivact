//! Role: Compare retained paint order, ink and child indices with a fresh walk.

use crate::model::Model;
use crate::paint::state::{ChildIndex, Drawn, Ink, PaintList, StaleFacts};

impl Model {
    pub(crate) fn paint_self_check(&mut self) -> Vec<String> {
        // Run both walks on disposable lists. Even refreshing the retained
        // list's lazy bookkeeping must not repair the live cache for later.
        let original = std::mem::take(&mut self.paint.list);
        let kept = std::mem::replace(&mut self.paint.stale_facts, StaleFacts::All);
        self.paint.list = original.clone();
        let inks: Vec<_> = self
            .tree
            .nodes
            .iter()
            .map(|(id, node)| (*id, self.paint.ink[node.index.0].get()))
            .collect();
        self.paint_list();
        let retained = std::mem::take(&mut self.paint.list);
        let retained_inks: Vec<_> = self
            .tree
            .nodes
            .iter()
            .map(|(id, node)| (*id, self.paint.ink[node.index.0].take()))
            .collect();
        self.paint.list = PaintList {
            pass: retained.pass,
            ink_gen: retained.ink_gen,
            ..PaintList::default()
        };
        self.paint.stale_facts = StaleFacts::All;
        self.paint_list();
        let rebuilt = std::mem::take(&mut self.paint.list);
        let mut errors = Vec::new();
        compare_entries(&retained, &rebuilt, &mut errors);
        for (id, before) in &retained_inks {
            let Some(node) = self.tree.get(*id) else {
                continue;
            };
            let after = self.paint.ink[node.index.0].get();
            // A stale generation or differently sized cache is intentionally
            // ignored by the production walk, so compare only reusable inks.
            if let (Some(before), Some(after)) = (*before, after)
                && before.generation == retained.ink_gen
                && before.size == after.size
                && before.content == after.content
                && ink_value(before) != ink_value(after)
            {
                errors.push(format!(
                    "paint node {} ink: retained {before:?}, rebuilt {after:?}",
                    id.0
                ));
            }
        }
        // Normalize retained indices against their own walk's ink state,
        // independently of the freshly rebuilt reference caches.
        for (id, ink) in &retained_inks {
            if let Some(node) = self.tree.get(*id) {
                self.paint.ink[node.index.0].set(*ink);
            }
        }
        for (id, after) in &rebuilt.indices {
            let before = retained.indices.get(id).map(|index| {
                let mut index = index.clone();
                index.refresh(self, retained.pass, retained.ink_gen);
                index
            });
            match before.as_ref() {
                Some(before) if same_index(before, after) => {}
                before => errors.push(format!(
                    "paint node {} child index: retained {before:?}, rebuilt {after:?}",
                    id.0
                )),
            }
        }
        // Index refresh can populate inks too. Restore every original cell
        // after all comparisons, including cells that originally held None.
        for (id, ink) in inks {
            if let Some(node) = self.tree.get(id) {
                self.paint.ink[node.index.0].set(ink);
            }
        }
        self.paint.list = original;
        self.paint.stale_facts = kept;
        errors
    }
}

fn ink_value(ink: Ink) -> (Option<Drawn>, kivact_types::Size, kivact_types::Size) {
    (ink.rect, ink.size, ink.content)
}

// Pass counters and pending touched children describe cache maintenance,
// not the spatial index that the walk reads after refreshing it.
fn same_index(before: &ChildIndex, after: &ChildIndex) -> bool {
    before.axis == after.axis
        && before.starts == after.starts
        && before.furthest == after.furthest
        && before.sticky == after.sticky
        && before.entries.len() == after.entries.len()
        && before
            .entries
            .iter()
            .zip(&after.entries)
            .all(|(before, after)| {
                (
                    before.id,
                    before.order,
                    before.sticky,
                    before.location,
                    before.extent,
                ) == (
                    after.id,
                    after.order,
                    after.sticky,
                    after.location,
                    after.extent,
                )
            })
}

fn compare_entries(retained: &PaintList, rebuilt: &PaintList, errors: &mut Vec<String>) {
    for index in 0..retained.entries.len().max(rebuilt.entries.len()) {
        let before = retained.entries.get(index);
        let after = rebuilt.entries.get(index);
        if before != after {
            errors.push(format!(
                "paint entry {index}: retained {before:?}, rebuilt {after:?}"
            ));
        }
    }
    if retained.transforms != rebuilt.transforms {
        errors.push(format!(
            "paint transforms: retained {:?}, rebuilt {:?}",
            retained.transforms, rebuilt.transforms
        ));
    }
    if retained.layers != rebuilt.layers {
        errors.push(format!(
            "paint layers: retained {:?}, rebuilt {:?}",
            retained.layers, rebuilt.layers
        ));
    }
}
