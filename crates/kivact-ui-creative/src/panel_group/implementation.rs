//! Role: Inspector panel groups with peer collapse and reordering.
use super::data::InspectorPanel;
use super::machine;

use crate::Color;
use kivact::geometry::Rect;
use kivact::i18n::{message, message_with};
use kivact::interaction::{DragCoordinates, DragPhase, DragUpdate, use_disclosure, use_drag};
use kivact::state::IntoBinding;
use kivact::state::IntoValue;
use kivact::state::{Value, machine, memo};
use kivact::style::{ColorToken, style};
use kivact::view::NodeBuilder;
use kivact::view::icon;
use kivact::view::{IntoView, Role, column, node, node_ref, row, text};

use kivact::state::binding;
use kivact::style::{Align, use_theme};
use kivact::view::A11yAction;
use kivact::view::for_each;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Inspector panels with peer collapse and grip reordering. Content remains mounted when closed.
/// Opening or closing a panel (a click, a key or an assistive action) while Ctrl or Meta is held
/// leaves only that panel open. The grip also offers accessible up/down movement.
#[track_caller]
pub fn panel_group<V: IntoView>(
    panels: impl IntoBinding<Vec<InspectorPanel>>,
    content: impl Fn(Value<InspectorPanel>) -> V + 'static,
) -> impl kivact::view::Component {
    let panels = panels.into_binding();
    let data = panels.into_value();
    let rectangles = Rc::new(RefCell::new(HashMap::<u64, Value<Option<Rect>>>::new()));
    let drag = machine(
        machine::State::Idle,
        move || machine::Machine {
            panels: data.get_untracked(),
        },
        move |_, next| panels.set(next),
    );
    let rows = for_each(data, |panel| panel.id, {
        let rectangles = rectangles.clone();
        move |panel| {
            let id = panel.get_untracked().id;
            kivact::state::on_cleanup(move || {
                drag.send(machine::Event::Removed(id));
            });
            let reference = node_ref();
            rectangles.borrow_mut().insert(id, reference.rect());
            let drag_rects = rectangles.clone();
            let expanded = binding(
                memo(move || panel.with(|panel| panel.expanded)),
                move |open: bool| {
                    let held = kivact::view::modifiers().get();
                    let mut next = data.get();
                    if held.ctrl || held.meta {
                        // Ctrl or Meta: only this panel stays open.
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
                    drag.send(machine::Event::Begin(id));
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
                    drag.send(machine::Event::Target(found));
                })
                .on_end(move |update: DragUpdate<u64>| {
                    drag.send(machine::Event::End(update.phase == DragPhase::End));
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
                .with_node(move |node| node.style(style().size(18, 18).padding(0)))
                // The grip is its own press target, so a click or drag on it
                // does not reach the header's Activate and toggle the panel.
                .on_press(|| {})
                .with_node(move |node| {
                    node.a11y_action(A11yAction::Increment, move |_| reposition(1))
                })
                .with_node(move |node| {
                    node.a11y_action(A11yAction::Decrement, move |_| reposition(-1))
                })
                .with_node(move |node| node.part(reorder.source(id)));
            let header = row((
                icon("chevron-down")
                    .style(move || style().rotate(if panel.get().expanded { 0 } else { -90 })),
                text(move || panel.get().title).style(style().grow(1)),
                grip,
            ))
            .part(use_disclosure(expanded).trigger())
            .name(move || panel.get().title)
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
                let theme = &use_theme().get();
                let p = theme.ext::<crate::tokens::Tokens>();
                let mut style = style()
                    .radius(7)
                    .background(p.island)
                    .border(1, p.hair)
                    .opacity(if drag.get().panel() == Some(id) {
                        0.65
                    } else {
                        1.0
                    });
                if let Some((target, after)) = drag.get().target()
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
