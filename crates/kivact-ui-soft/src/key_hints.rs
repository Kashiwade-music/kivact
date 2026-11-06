//! Role: Show one timed navigation hint on actual keyboard targets.
use kivact::motion::{Easing, Transition};
use kivact::state::{
    IntoValue, Machine, MachineHandle, Step, effect, machine, on_cleanup, provide, try_use_context,
};
use kivact::style::{Align, TransitionProp, style};
use kivact::view::{
    Handled, IntoView, Key, NodeBuilder, NodeRef, Orientation, OverlayAlign, Placement, View,
    node_ref, overlay, row, show_some, text,
};
use std::time::Duration;

#[derive(Clone, Copy, PartialEq)]
enum KeyHint {
    Hidden,
    Showing { anchor: NodeRef },
    Seen,
}
enum Event {
    Focus(NodeRef),
    Blur(NodeRef),
    Dismiss,
}
enum Effect {}
struct Controller;
impl Machine for Controller {
    type State = KeyHint;
    type Event = Event;
    type Effect = Effect;
    fn transition(&self, state: &KeyHint, event: Event) -> Step<KeyHint, Event, Effect> {
        match (state, event) {
            (KeyHint::Hidden, Event::Focus(anchor)) => Step::to(KeyHint::Showing { anchor })
                .after(Duration::from_millis(3000), Event::Dismiss),
            (KeyHint::Showing { anchor }, Event::Blur(target)) if *anchor == target => {
                Step::to(KeyHint::Seen)
            }
            (KeyHint::Showing { .. }, Event::Dismiss) => Step::to(KeyHint::Seen),
            _ => Step::stay(),
        }
    }
}
#[derive(Clone, Copy)]
struct Hints(MachineHandle<Controller>);
/// Give a group one first-visit hint. Supply its navigation axis and attach
/// [`keyboard_hint`] to each participating target inside the content factory.
pub fn with_key_hints<V: IntoView>(
    orientation: impl IntoValue<Orientation>,
    content: impl FnOnce() -> V + 'static,
) -> View {
    let orientation = orientation.into_value();
    let state = machine(KeyHint::Hidden, || Controller, |_, effect| match effect {});
    let content = provide(Hints(state), move || content().into_view());
    let hint = show_some(
        move || match state.get() {
            KeyHint::Showing { anchor } => Some(anchor),
            _ => None,
        },
        move |anchor| {
            let keys = move || {
                if orientation.get() == Orientation::Vertical {
                    ("up", "down")
                } else {
                    ("left", "right")
                }
            };
            let label = kivact::i18n::message("soft.keyboard_hint.to_navigate", "to navigate");
            let surface = row((
                kivact::ui::kbd(move || keys().0.to_owned()),
                kivact::ui::kbd(move || keys().1.to_owned()),
                text(label).style(style().margin_left(4)),
            ))
            .hit_testable(false)
            .style(|| {
                let t = kivact::style::use_tokens::<crate::Tokens>();
                let base = style()
                    .gap(4)
                    .padding_x(8)
                    .padding_y(4)
                    .align_items(Align::Center)
                    .background(t.colors.bg_popover)
                    .radius(t.radius.element)
                    .color(t.colors.text_secondary)
                    .font_size(12)
                    .line_height(20.0 / 12.0)
                    .text_wrap(false)
                    .enter(style().opacity(0.0))
                    .exit(style().opacity(0.0))
                    .transition(
                        TransitionProp::Paint,
                        Transition::tween(150, Easing::CubicBezier(0.25, 0.1, 0.25, 1.0)),
                    );
                t.shadows
                    .low
                    .iter()
                    .cloned()
                    .fold(base, |base, shadow| base.shadow(shadow))
            });
            overlay(anchor.get(), Placement::Below, surface)
                .offset(8.0)
                .align(OverlayAlign::Start)
        },
    );
    (content, hint).into_view()
}
/// Observe this actual target's keyboard focus without changing its role or
/// navigation behavior. Outside [`with_key_hints`] this attachment is inert.
pub fn keyboard_hint<T: NodeBuilder>(target: T) -> T {
    let reference = node_ref();
    target.with_node(move |node| {
        node.node_ref(reference)
            .on_attach(move |mount| {
                let Some(Hints(state)) = try_use_context::<Hints>().map(|value| value.get()) else {
                    return;
                };
                on_cleanup(move || state.send(Event::Blur(reference)));
                let focus_visible = reference.focus_visible();
                effect(move || {
                    if focus_visible.get() && mount.eligible.get() {
                        state.send(Event::Focus(reference));
                    } else {
                        state.send(Event::Blur(reference));
                    }
                });
            })
            .on_key(move |event| {
                if matches!(
                    event.key(),
                    Key::ArrowUp | Key::ArrowDown | Key::ArrowLeft | Key::ArrowRight
                ) && let Some(Hints(state)) = try_use_context::<Hints>().map(|value| value.get())
                {
                    state.send(Event::Dismiss);
                }
                Handled::No
            })
    })
}
