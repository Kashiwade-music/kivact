//! Role: Inspector panel groups with peer collapse and reordering.

use crate::Color;
use kivact::control::icon;
use kivact::draw::Rect;
use kivact::i18n::{message, message_with};
use kivact::interaction::{DragCoordinates, DragPhase, DragUpdate, use_disclosure, use_drag};
use kivact::prelude::{
    ColorToken, IntoValue, IntoView, Node, NodeBuilder, Role, Value, column, memo, node, node_ref,
    row, signal, style, text, var,
};
use kivact::primitive::NodeExt;
use kivact::style::{Align, current_theme};
use kivact::view::{A11yAction, Handled};
use kivact::view::{ButtonPress, MouseButton, PointerPhase, for_each};
use kivact::{Binding, binding};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// A retained inspector panel. Identities must be unique within its group.
#[derive(Clone, Debug, PartialEq)]
pub struct InspectorPanel {
    pub id: u64,
    pub title: String,
    pub expanded: bool,
}
impl InspectorPanel {
    pub fn new(id: u64, title: impl Into<String>) -> Self {
        Self {
            id,
            title: title.into(),
            expanded: true,
        }
    }
    pub fn collapsed(mut self) -> Self {
        self.expanded = false;
        self
    }
}
/// Inspector panels with peer collapse and grip reordering. Content remains mounted when closed.
/// Ctrl/Meta-click opens only the chosen panel. The grip also offers accessible up/down movement.
#[track_caller]
pub fn panel_group<V: IntoView>(
    panels: impl Into<Binding<Vec<InspectorPanel>>>,
    content: impl Fn(Value<InspectorPanel>) -> V + 'static,
) -> Node {
    let panels = panels.into();
    let data = panels.into_value();
    let rectangles = Rc::new(RefCell::new(HashMap::<u64, Value<Option<Rect>>>::new()));
    let dragged_panel = signal(None::<u64>);
    let target = signal(None::<(u64, bool)>);
    let rows = for_each(data, |panel| panel.id, {
        let rectangles = rectangles.clone();
        move |id, panel| {
            kivact::on_cleanup(move || {
                if dragged_panel.get_untracked() == Some(id) {
                    dragged_panel.set(None);
                    target.set(None);
                }
            });
            let reference = node_ref();
            rectangles.borrow_mut().insert(id, reference.rect());
            let drag_rects = rectangles.clone();
            // Ctrl/Meta on the press makes the next write open only this panel.
            let exclusive = var(false);
            let expanded = binding(
                memo(move || panel.with(|panel| panel.expanded)),
                move |open: bool| {
                    let only_this = exclusive.update(std::mem::take);
                    let mut next = data.get();
                    if only_this {
                        for item in &mut next {
                            item.expanded = item.id == id;
                        }
                    } else if let Some(item) = next.iter_mut().find(|item| item.id == id) {
                        item.expanded = open;
                    }
                    panels.set(next);
                },
            );
            let reposition = move |delta: isize| {
                let mut next = data.get_untracked();
                if let Some(index) = next.iter().position(|item| item.id == id) {
                    let destination = (index as isize + delta)
                        .clamp(0, next.len().saturating_sub(1) as isize)
                        as usize;
                    let item = next.remove(index);
                    next.insert(destination, item);
                    panels.set(next);
                }
            };
            let reorder = use_drag(DragCoordinates::Window)
                .on_start(move |_| {
                    dragged_panel.set(Some(id));
                    target.set(None);
                })
                .on_update(move |update: DragUpdate<u64>| {
                    let order = data.get();
                    let mut found = None;
                    for item in order.iter().filter(|item| item.id != id) {
                        if let Some(rect) = drag_rects.borrow().get(&item.id).and_then(|r| r.get())
                        {
                            let after = update.pos.y >= rect.center().y;
                            found = Some((item.id, after));
                            if !after {
                                break;
                            }
                        }
                    }
                    target.set(found);
                })
                .on_end(move |update: DragUpdate<u64>| {
                    if update.phase == DragPhase::End
                        && let Some((destination, after)) = target.get()
                    {
                        let mut next = data.get();
                        if let Some(index) = next.iter().position(|item| item.id == id) {
                            let item = next.remove(index);
                            if let Some(index) = next.iter().position(|item| item.id == destination)
                            {
                                next.insert(index + usize::from(after), item);
                                panels.set(next);
                            }
                        }
                    }
                    dragged_panel.set(None);
                    target.set(None);
                });
            let mut grip = kivact::ui::button(message_with(
                "creative.panels.reorder",
                "Reorder {title}",
                move || [("title", panel.get().title)],
            ))
            .variant(kivact::ui::ButtonVariant::Secondary)
            .hide_label(true)
            .icon(icon("more"));
            grip = grip
                .style(style().size(18, 18).padding(0))
                // The grip is its own press target, so a click or drag on it
                // does not reach the header's Activate and toggle the panel.
                .on_press(|| {})
                .on_a11y(A11yAction::Increment, move || reposition(1))
                .on_a11y(A11yAction::Decrement, move || reposition(-1))
                .part(reorder.source(id));
            let header = row((
                icon("chevron-down")
                    .style(move || style().rotate(if panel.get().expanded { 0 } else { -90 })),
                text(move || panel.get().title).style(style().grow(1)),
                grip,
            ))
            .part(use_disclosure(expanded).trigger())
            .name(move || panel.get().title)
            .on_pointer_default(move |event| {
                match event.phase {
                    PointerPhase::Down(ButtonPress {
                        button: MouseButton::Primary,
                        ..
                    }) => {
                        exclusive.set(event.modifiers.ctrl || event.modifiers.meta);
                    }
                    PointerPhase::Leave | PointerPhase::Cancel => exclusive.set(false),
                    _ => {}
                }
                Handled::No
            })
            .on_key_default(move |_| {
                exclusive.set(false);
                Handled::No
            })
            .style(
                style()
                    .height(30)
                    .padding_x(8)
                    .gap(6)
                    .align_items(Align::Center)
                    .background(Color::Header)
                    .hover(style().background(ColorToken::Hover)),
            );
            column((
                header,
                node()
                    .hidden(move || !panel.get().expanded)
                    .style(style().padding(10))
                    .child(content(panel)),
            ))
            .node_ref(reference)
            .role(Role::Group)
            .name(message_with(
                "creative.panels.panel",
                "{title} panel",
                move || [("title", panel.get().title)],
            ))
            .style(move || {
                let theme = &current_theme();
                let p = theme.get::<crate::tokens::Tokens>();
                let mut style = style()
                    .radius(7)
                    .background(p.island)
                    .border(1, p.hair)
                    .opacity(if dragged_panel.get() == Some(id) {
                        0.65
                    } else {
                        1.0
                    });
                if let Some((target, after)) = target.get()
                    && target == id
                {
                    style = if after {
                        style.border_bottom(2, p.accent)
                    } else {
                        style.border_top(2, p.accent)
                    };
                }
                style
            })
        }
    });
    column((rows,))
        .role(Role::Group)
        .name(message("creative.panels.label", "Property panels"))
        .style(style().gap(6))
}
