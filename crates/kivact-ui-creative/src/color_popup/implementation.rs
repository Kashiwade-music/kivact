//! Role: Color popup of the Creative kit with live editing and rollback.
use super::machine;
use kivact::state::IntoBinding;
use kivact::state::IntoValue;
use kivact::ui::button;
use kivact::view::NodeBuilder;

use crate::color_wheel;
use kivact::interaction::use_popup;

use kivact::state::{IntoText, binding, machine, memo, signal};
use kivact::style::{ColorToken, style};
use kivact::view::{Role, column, show, text};

use kivact::style::{Color, pct};
use kivact::view::{DismissReason, Placement, overlay};

/// A live color edit transaction: Escape restores the opening color; outside dismissal commits.
#[track_caller]
pub fn color_popup(
    label: impl IntoText,
    color: impl IntoBinding<Color>,
) -> impl kivact::view::Component {
    let label = label.into_text_value();
    let color = color.into_binding();
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
                text(label).style(style().weight(600)),
                color_wheel(color, signal(crate::ColorModel::Rgb))
                    .alpha(true)
                    .wheel_size(180.0)
                    .with_node(move |node| node.style(style().width(240))),
            ))
            .style(style().width(264).padding(12).gap(8).radius(12))
            .style(
                style()
                    .background(ColorToken::Raised)
                    .color(crate::ColorName::Text)
                    .border(1, crate::ColorName::HairStrong),
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
                let swatch = style()
                    .background(value.get())
                    .color(crate::color::transparent_ink());
                style()
                    .width(pct(100))
                    .background(value.get())
                    .color(crate::color::transparent_ink())
                    .hover(swatch.clone())
                    .pressed(swatch.clone())
                    .focus(swatch)
            })
        });
    column((opener, layer))
        .role(Role::Group)
        .style(style().gap(0))
}
