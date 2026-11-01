//! Role: Color popup of the Creative kit with live editing and rollback.
use kivact::ui::button;

use crate::color_wheel;
use kivact::Binding;
use kivact::interaction::{EditSession, use_popup};
use kivact::prelude::*;
use kivact::primitive::NodeExt;
use kivact::style::{Color, pct, rgba};
use kivact::view::A11yAction;
use kivact::view::{DismissReason, Placement, overlay};

/// A live color edit transaction: Escape restores the opening color; outside dismissal commits.
#[track_caller]
pub fn color_popup(label: impl IntoText, color: impl Into<Binding<Color>>) -> Node {
    let label = label.into_text_value();
    let color = color.into();
    let value = color.into_value();
    let session = EditSession::new(move |initial| color.set(initial));
    let open = signal(false);
    effect(move || {
        if !open.get() {
            session.commit();
        }
    });
    let popup = use_popup(open);
    let trigger = popup.trigger();
    let anchor = trigger.node_ref();
    let layer = show(popup.shown(), move || {
        overlay(
            anchor,
            Placement::Below,
            column((
                text(label).style(style().weight(600)),
                color_wheel(color, signal(crate::ColorModel::Rgb))
                    .alpha(true)
                    .wheel_size(180.0)
                    .style(style().width(240)),
            ))
            .style(style().width(264).padding(12).gap(8).radius(12))
            .style(
                style()
                    .background(ColorToken::Raised)
                    .color(crate::Color::Text)
                    .border(1, crate::Color::HairStrong),
            ),
        )
        .role(Role::Dialog)
        .name(label)
        .offset(4)
        .on_dismiss(move |reason| {
            if reason == DismissReason::Escape {
                session.cancel();
            }
        })
        .part(popup.content())
    });
    let snapshot = move || {
        if !open.get_untracked() {
            session.begin(value.get_untracked());
        }
    };
    let opener = button(label)
        .variant(kivact::ui::ButtonVariant::Secondary)
        .part(trigger)
        .style(move || {
            let swatch = style().background(value.get()).color(rgba(0, 0.0));
            style()
                .width(pct(100))
                .background(value.get())
                .color(rgba(0, 0.0))
                .hover(swatch.clone())
                .pressed(swatch.clone())
                .focus(swatch)
        })
        .on_press(snapshot)
        .on_a11y(A11yAction::Expand, snapshot);
    column((opener, layer))
        .role(Role::Group)
        .style(style().gap(0))
}
