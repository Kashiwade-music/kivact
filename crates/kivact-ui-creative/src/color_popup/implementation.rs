//! Role: Color popup of the Creative kit with live editing and rollback.
use super::machine;
use crate::color::{HAIR_STRONG, TEXT, TRANSPARENT_INK};
use crate::color_wheel::{ColorModel, color_wheel};
use kivact::component;
use kivact::interaction::use_popup;
use kivact::state::{IntoValue, Signal, Value, binding, machine, memo, signal};
use kivact::style::{Color, pct, style};
use kivact::ui::button;
use kivact::view::{DismissReason, Element, Placement, Role, column, overlay, show, text};

/// A live color edit transaction: Escape restores the opening color; outside dismissal commits.
#[component]
pub fn color_popup(label: Value<String>, color: Signal<Color>) -> impl Element {
    let value = color.into_value();
    let phase = machine(
        machine::ColorPopupPhase::Closed,
        move || machine::Machine {
            color: value.get_untracked(),
        },
        move |_, effect| match effect {
            machine::Effect::Restore(initial) => color.set(initial),
        },
    );
    let open = binding(
        memo(move || matches!(phase.get(), machine::ColorPopupPhase::Editing { .. })),
        move |open| {
            phase.send(if open {
                machine::Event::Open
            } else {
                machine::Event::Close
            })
        },
    );
    let popup = use_popup(open);
    let trigger = popup.trigger();
    let anchor = kivact::view::node_ref();
    let trigger = kivact::custom::Part::new(trigger.state(), move |node| {
        node.part(trigger).node_ref(anchor)
    });
    let layer = show(popup.is_open(), move || {
        overlay(
            anchor,
            Placement::Below,
            column((
                text(label).style(style().font_weight(600)),
                color_wheel(color, signal(ColorModel::Rgb))
                    .alpha(true)
                    .wheel_size(180.0)
                    .with_node(move |node| node.style(style().width(240))),
            ))
            .style(style().width(264).padding(12).gap(8).border_radius(12))
            .style(
                style()
                    .background(kivact::style::color::RAISED)
                    .color(TEXT)
                    .border(1, HAIR_STRONG),
            ),
        )
        .with_node(move |node| node.role(Role::Dialog))
        .with_node(move |node| node.name(label))
        .offset(4)
        .on_dismiss(move |reason| {
            if reason == DismissReason::Escape {
                phase.send(machine::Event::Escape);
            }
        })
        .with_node(move |node| node.part(popup.content()))
    });
    let opener = button(label)
        .variant(kivact::ui::ButtonVariant::Secondary)
        .with_node(move |node| node.part(trigger))
        .with_node(move |node| {
            node.style(move || {
                let swatch = style().background(value.get()).color(TRANSPARENT_INK);
                style()
                    .width(pct(100))
                    .background(value.get())
                    .color(TRANSPARENT_INK)
                    .hover(swatch.clone())
                    .active(swatch.clone())
                    .focus(swatch)
            })
        });
    column((opener, layer))
        .role(Role::Group)
        .style(style().gap(0))
}
