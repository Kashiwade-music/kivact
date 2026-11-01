#![allow(dead_code, unused_variables)]
// Usage example: canvas, animation and a custom primitive
use kivact::motion::{Spring, Transition, animated};
use kivact::prelude::*;
use kivact::control::{canvas};
use kivact::draw::{Draw, Rect, Size};
use kivact::style::{Color};
use kivact::view::{A11yAction, Handled, KeyEvent, PointerEvent, PointerId};
use kivact::primitive::NodeExt;
use kivact::Callback;
include!("../support/prelude_stubs.rs");

fn meter(level: Value<f32>) -> impl IntoView {
    let shown = animated(level, Transition::spring(Spring::GENTLE)); // Value<f32> を補間する
    canvas(move |d: &mut Draw, size: Size| {
        d.fill_rect(size.to_rect(), Color::token(ColorToken::Surface));
        d.fill_rect(Rect::new(0.0, 0.0, size.width * shown.get(), size.height), Color::token(ColorToken::Accent));
    })
    .style(style().height(24).grow(1))
    .role(Role::ProgressBar)
    .name("レベル")
    .value_text(move || format!("{:.0}%", level.get() * 100.0))
}

// button と同じ水準の基本部品を、公開 API だけで作る。
// 引数が多い部品は、利用者が struct（KnobProps）にまとめてよい
pub fn knob(props: KnobProps) -> impl IntoView {
    let drag = var(None::<(PointerId, f32)>);
    node()
        .role(Role::Slider)
        .name(props.label)
        .value_text(move || format!("{:.2}", props.value.get()))
        .focusable(true)
        .style(style().size(48, 48).focus(style().ring(2, ColorToken::Focus)))
        .draw(move |d, size| draw_knob(d, size, props.value.get()))
        .on_pointer(move |e: PointerEvent| knob_drag(e, drag, props))
        .on_key(move |k: KeyEvent| match k.key {
            Key::ArrowUp => {
                props.on_change.call(step(props.value.get_untracked(), 0.05));
                Handled::Yes
            }
            _ => Handled::No,
        })
        .on_a11y(A11yAction::Increment, move || props.on_change.call(step(props.value.get_untracked(), 0.05)))
}

fn main() {
    let _ = meter;
    let _ = knob;
}
