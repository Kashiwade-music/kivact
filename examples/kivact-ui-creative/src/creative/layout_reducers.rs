//! Role: Application reducers for catalog hierarchy and row order.

use kivact::{Signal, signal};
use kivact_ui_creative as ui;

pub(super) fn outliner(items: Signal<Vec<ui::TreeItem>>, state: ui::OutlinerState) -> ui::Outliner {
    let initial = signal(Vec::new());
    ui::outliner(items, state)
        .on_item_drop(move |drop: ui::TreeDrop| {
            items.update(|items| {
                let Some(from) = items.iter().position(|item| item.id == drop.id) else {
                    return;
                };
                let mut item = items.remove(from);
                item.parent = drop.target;
                let at = match drop.after {
                    Some(after) => items
                        .iter()
                        .position(|item| item.id == after)
                        .map_or(items.len(), |at| at + 1),
                    None => items
                        .iter()
                        .position(|item| item.parent == drop.target)
                        .unwrap_or(from.min(items.len())),
                };
                items.insert(at, item);
            });
        })
        .on_rename(move |(id, name): (u64, String)| {
            if name.trim().is_empty() {
                return;
            }
            items.update(|items| {
                if let Some(item) = items.iter_mut().find(|item| item.id == id) {
                    item.label = name;
                }
            });
        })
        .on_restriction(move |event: ui::EditorEvent<ui::Restriction>| match event {
            ui::EditorEvent::Begin => initial.set(items.get()),
            ui::EditorEvent::Request(request) => items.update(|items| {
                for item in items
                    .iter_mut()
                    .filter(|item| request.ids.contains(&item.id))
                {
                    match request.flag {
                        ui::RestrictionFlag::Visible => item.visible = request.value,
                        ui::RestrictionFlag::Selectable => item.selectable = request.value,
                        ui::RestrictionFlag::Rendered => item.rendered = request.value,
                        // Variants added later fall back to being ignored.
                        _ => {}
                    }
                }
            }),
            ui::EditorEvent::Cancel => items.set(initial.get()),
            ui::EditorEvent::Commit => {}
            // Variants added later fall back to being ignored.
            _ => {}
        })
}

pub(super) fn reorder(rows: Signal<Vec<super::common::TextRow>>, id: u64, destination: usize) {
    rows.update(|rows| {
        if let Some(source) = rows.iter().position(|row| row.id == id) {
            let row = rows.remove(source);
            rows.insert(destination.min(rows.len()), row);
        }
    });
}
