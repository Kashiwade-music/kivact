//! Role: Verify viewport coordinate sharing, fitting, and CPU snapshot output.
use super::support::mount;
use kivact::draw::Pixels;
use kivact::geometry::Point;
use kivact::interaction::{ViewTransform, use_zoom_pan};
use kivact::prelude::*;
use kivact::view::{ImageFit, viewport};

fn pixels() -> Pixels {
    Pixels::new(2, 1, vec![230, 30, 20, 255, 230, 30, 20, 255].into())
}

#[test]
fn viewport_maps_points_between_view_and_content() {
    let (screen, (position, api)) = mount(|| {
        let api = use_zoom_pan(signal(ViewTransform {
            offset: Point::new(10., 20.),
            scale: 2.,
        }));
        let position = signal(Point::new(0., 0.));
        (
            (position, api),
            viewport(pixels())
                .transform(api)
                .with_node(move |node| {
                    node.on_pointer(move |event| {
                        position.set(api.to_content(event.pos()));
                        kivact::view::Handled::No
                    })
                })
                .style(style().size(300, 200))
                .test_id("viewport"),
        )
    });
    screen.get_by_test_id("viewport").hover();
    screen.mouse_move(110., 120.);
    assert_eq!(position.get_untracked(), Point::new(50., 50.));
    assert_eq!(api.to_view(Point::new(50., 50.)), Point::new(110., 120.));
    screen.scroll(0., -100.);
    let content = api.to_content(Point::new(110., 120.));
    assert!((content.x - 50.).abs() < 0.001 && (content.y - 50.).abs() < 0.001);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn viewport_shows_pixels_with_fit() {
    let (screen, fit) = mount(|| {
        let fit = signal(ImageFit::Contain);
        (
            fit,
            viewport(pixels())
                .fit(fit)
                .style(style().size(100, 100).background(Color::WHITE)),
        )
    });
    let red_count = || {
        screen
            .screenshot()
            .pixels
            .iter()
            .filter(|&&pixel| {
                let red = (pixel >> 16) & 0xff;
                let green = (pixel >> 8) & 0xff;
                let blue = pixel & 0xff;
                red > 180 && green < 80 && blue < 80
            })
            .count()
    };
    let contained = red_count();
    fit.set(ImageFit::Fill);
    screen.run_until_idle();
    let filled = red_count();
    assert!(
        contained > 2000 && filled > contained + 2000,
        "fit changes the painted area: contain={contained}, fill={filled}"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn viewport_snapshot_draws_its_pixels() {
    let (screen, capture) = mount(|| {
        let target = node_ref();
        let capture = callback(move |reply: std::sync::mpsc::Sender<_>| {
            kivact::task::spawn(async move {
                reply
                    .send(target.snapshot().await)
                    .expect("snapshot receiver lives");
            });
        });
        (
            capture,
            viewport(pixels())
                .fit(ImageFit::Fill)
                .node_ref(target)
                .style(style().size(80, 40)),
        )
    });
    let (send, receive) = std::sync::mpsc::channel();
    capture.call(send);
    screen.run_until_idle();
    let image = receive
        .try_recv()
        .expect("snapshot completes")
        .expect("pixels render on CPU");
    assert_eq!((image.width, image.height), (80, 40));
    assert!(
        image
            .rgba
            .as_chunks::<4>()
            .0
            .iter()
            .filter(|p| p[0] > 180 && p[1] < 80 && p[2] < 80)
            .count()
            > 2000
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
