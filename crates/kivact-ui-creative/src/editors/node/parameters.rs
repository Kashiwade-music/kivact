//! Role: Node parameter popups forwarding edits to the editor session.
use crate::ColorName;
use crate::editors::node::gesture::{NodeMode, State};
use crate::editors::node::model::{NodeControlValue, NodeRequest};
use kivact::state::IntoValue;
use kivact::state::binding;
use kivact::view::Element;
use kivact::view::{column, node};

use kivact::style::{ColorToken, style};
use kivact::view::{IntoView, Placement, Role, overlay_at};

pub(super) fn popup(state: State) -> impl IntoView {
    let visible = binding(
        (move || {
            state
                .mode
                .with(|mode| matches!(mode, NodeMode::Popup { .. }))
                && !state.blocked.get()
        })
        .into_value(),
        move |visible: bool| {
            if !visible {
                state.close_popup();
            }
        },
    );
    let popup = kivact::interaction::use_popup(visible);
    kivact::view::show(popup.is_open(), move || {
        let (id, index, point) = state
            .mode
            .with_untracked(|mode| match mode {
                NodeMode::Popup {
                    node, index, at, ..
                } => Some((*node, *index, *at)),
                _ => None,
            })
            .expect("invariant: shown node popup has a control");
        let content = control_content(state, id, index);
        overlay_at(
            point,
            Placement::Below,
            node()
                .child(content)
                .style(style().padding(8).radius(8).min_width(180))
                .style(
                    style()
                        .background(ColorToken::Raised)
                        .border(1, ColorName::HairStrong),
                ),
        )
        .with_node(move |node| node.part(popup.content()))
    })
}

fn control_content(state: State, id: u64, index: usize) -> kivact::view::View {
    let data = state.data();
    let control = state.control(id, index);
    match control.map(|c| (c.label, c.value)) {
        Some((label, NodeControlValue::Number { min, max, step, .. })) => crate::number(
            label,
            binding(
                (move || {
                    data.with(|nodes| {
                        nodes
                            .iter()
                            .find(|n| n.id == id)
                            .and_then(|n| n.controls.get(index))
                            .map_or(0.0, |c| {
                                if let NodeControlValue::Number { value, .. } = c.value {
                                    value
                                } else {
                                    0.0
                                }
                            })
                    })
                })
                .into_value(),
                move |value| {
                    if !state.blocked.get() && state.popup_is(id, index) && state.edit.is_active() {
                        state.change_raw(id, index, Some(value), 0.0);
                    }
                },
            ),
        )
        .range(
            kivact::interaction::ScalarRange::new(min, max)
                .expect("invariant: finite ordered control range"),
        )
        .step(step)
        .precision(3)
        .on_edit(move |phase| state.control_phase(id, index, phase))
        .with_node(move |node| node.disabled(state.blocked))
        .with_node(move |node| node.style(style().width(190)))
        .into_view(),
        Some((_, NodeControlValue::Choice { options, selected })) => {
            choice_content(state, id, index, options, selected)
        }
        _ => node().into_view(),
    }
}

fn choice_content(
    state: State,
    id: u64,
    index: usize,
    options: Vec<String>,
    selected: usize,
) -> kivact::view::View {
    column(
        options
            .into_iter()
            .enumerate()
            .map(|(choice, label)| {
                kivact::ui::button(label)
                    .variant(kivact::ui::ButtonVariant::Secondary)
                    .with_node(move |node| node.role(Role::Option))
                    .with_node(move |node| node.selected(choice == selected))
                    .with_node(move |node| node.disabled(state.blocked))
                    .on_press(move || {
                        if state.busy() {
                            return;
                        }
                        if let Some(control) = state.control(id, index) {
                            state.edit.once((), || {
                                if let NodeControlValue::Choice { options, .. } =
                                    control.value.clone()
                                {
                                    state.request(NodeRequest::Control {
                                        node: id,
                                        index,
                                        value: NodeControlValue::Choice {
                                            options,
                                            selected: choice,
                                        },
                                    });
                                }
                            });
                        }
                        state.close_popup();
                    })
                    .into_view()
            })
            .collect::<Vec<_>>(),
    )
    .into_view()
}
