//! Role: Preserve the rendering depth budget for retained canvas placement.
use super::paint::{RED, pixel};
use kivact::control::canvas;
use kivact::draw::Rect;
use kivact::prelude::*;
use kivact::style::{Color, Position};

fn nested(draw: &mut kivact::draw::Draw, depth: usize) {
    if depth == 0 {
        draw.fill_rect(Rect::new(2., 2., 12., 12.), rgb(RED));
    } else {
        draw.layer(
            kivact::draw::Layer::new().transform(kivact::draw::Transform::new()),
            |draw| nested(draw, depth - 1),
        );
    }
}
#[test]
fn a_canvas_at_the_renderer_depth_limit_keeps_its_nonzero_origin() {
    let screen = kivact::test::render_with(
        kivact::test::RenderOptions {
            window: kivact::test::WindowMode::Real,
            ..Default::default()
        },
        || {
            node()
                .child(
                    canvas(|d, _| nested(d, 256))
                        .style(style().position(Position::Absolute).left(40).size(40, 30)),
                )
                .style(style().size(100, 40).background(Color::WHITE))
        },
    );
    let shot = screen.screenshot();
    assert_eq!(
        pixel(&shot, 45, 5),
        RED,
        "the valid depth budget is unchanged by retained placement"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_cached_canvas_respects_new_parent_clips_at_the_global_depth_limit() {
    let (screen, (clip, recordings)) = crate::support::mount_with(
        kivact::test::RenderOptions {
            window: kivact::test::WindowMode::Real,
            ..Default::default()
        },
        || {
            let clip = signal(false);
            let recordings = std::rc::Rc::new(std::cell::Cell::new(0));
            let called = recordings.clone();
            let mut content = canvas(move |d, _| {
                called.set(called.get() + 1);
                nested(d, 1)
            })
            .style(style().position(Position::Absolute).left(40).size(40, 30))
            .into_view();
            for _ in 0..255 {
                content = node()
                    .child(content)
                    .style(move || {
                        style().size(100, 40).overflow(if clip.get() {
                            kivact::style::Overflow::Hidden
                        } else {
                            kivact::style::Overflow::Visible
                        })
                    })
                    .into_view();
            }
            ((clip, recordings), content)
        },
    );
    assert_eq!(pixel(&screen.screenshot(), 45, 5), RED);
    let before_recordings = recordings.get();
    clip.set(true);
    screen.run_until_idle();
    assert_eq!(
        pixel(&screen.screenshot(), 45, 5),
        RED,
        "parent scope depth leaves no extra placement group"
    );
    assert_eq!(
        recordings.get(),
        before_recordings,
        "parent clips reuse the existing recorded Scene"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
