//! Role: Real-window worker completion and font registration scene regressions.
//! Public Screen fixes assets to inline execution and App fonts to startup.
//! These cases use the same AppClient and production test host directly,
//! with existing normal host operations, to exercise later resource changes.
use crate::client::AppClient;
use crate::prelude::{IntoView, NodeBuilder, Theme, rgb, style};
use crate::view::View;
use kivact_engine::testing::{ArvaneTestHost, TestHostOptions, WindowMode};
use kivact_reactive::Runtime;
use kivact_style::asset::Asset;
use kivact_style::draw::{Point, Rect, TextStyle};
use std::panic::Location;
use std::rc::Rc;
use std::time::{Duration, Instant};

fn host(
    root: impl FnOnce() -> View + 'static,
    inline_assets: bool,
) -> (ArvaneTestHost, Rc<Runtime>) {
    let client = AppClient::new(
        "retained scene".into(),
        Theme::light(),
        Some(2),
        Box::new(root),
        Location::caller(),
    );
    let rt = client.runtime().clone();
    let host = ArvaneTestHost::new(
        client.into_client(None),
        TestHostOptions {
            width: 160.,
            height: 60.,
            window: WindowMode::Real,
            inline_assets,
            ..Default::default()
        },
    )
    .expect("real window");
    (host, rt)
}
fn frame(host: &mut ArvaneTestHost) {
    while host.redraw() {}
}
fn pixel(host: &ArvaneTestHost, x: u32, y: u32) -> u32 {
    host.frame()
        .expect("presented frame")
        .pixel(x, y)
        .expect("pixel")
}
fn pump_until(host: &mut ArvaneTestHost, ready: impl Fn(&ArvaneTestHost) -> bool) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while !ready(host) {
        assert!(
            Instant::now() < deadline,
            "worker completed within deadline"
        );
        if host.woken() || host.due() || host.redraw_requested() {
            host.wake();
            frame(host);
        } else {
            std::thread::park_timeout(Duration::from_millis(2));
        }
    }
}

#[test]
fn pending_canvas_assets_complete_after_a_presented_frame_and_remain_live_on_reuse() {
    static SVG:&[u8]=br##"<svg xmlns="http://www.w3.org/2000/svg" width="12" height="12"><rect width="12" height="12" fill="#0000ff"/></svg>"##;
    let (mut host, _) = host(
        || {
            crate::control::canvas(|d, _| {
                d.fill_rect(Rect::new(0., 0., 4., 4.), rgb(0xff0000));
                d.image(&Asset::from(SVG), Rect::new(8., 8., 12., 12.));
            })
            .style(style().size(80, 40).background(rgb(0xffffff)))
            .into_view()
        },
        false,
    );
    // Present only once: its trailing host pump may apply worker completion
    // and request another redraw. Draining redraws here would replace the
    // pending frame before we can assert its pixels.
    assert!(host.redraw(), "present the initial pending frame");
    assert_eq!(
        pixel(&host, 2, 2),
        0xff0000,
        "non-resource commands draw in pending frame"
    );
    assert_eq!(
        pixel(&host, 12, 12),
        0xffffff,
        "worker pixels are pending in initial frame"
    );
    pump_until(&mut host, |h| pixel(h, 12, 12) == 0x0000ff);
    for width in [161., 162., 163.] {
        host.resize(width, 60.);
        frame(&mut host);
        assert_eq!(
            pixel(&host, 12, 12),
            0x0000ff,
            "cached resource remains used"
        );
    }
    host.memory_warning();
    host.resize(164., 60.);
    frame(&mut host);
    pump_until(&mut host, |h| pixel(h, 12, 12) == 0x0000ff);
}

#[test]
fn registering_a_font_after_a_cached_canvas_frame_replaces_fallback_shaping() {
    let (mut host, rt) = host(
        || {
            crate::control::canvas(|d, _| {
                d.text(
                    "WWWWii",
                    Point::new(4., 4.),
                    TextStyle {
                        size: 24.,
                        ..Default::default()
                    },
                );
            })
            .style(
                style()
                    .size(150, 50)
                    .font_family("Inter Variable")
                    .background(rgb(0xffffff)),
            )
            .into_view()
        },
        true,
    );
    frame(&mut host);
    let before = host.frame().expect("fallback frame").pixels.to_vec();
    host.resize(161., 60.);
    frame(&mut host);
    crate::view::requests::register_app_fonts(
        &rt,
        vec![(
            Asset::from(&include_bytes!("../../../../arvane/tests/fonts/InterVariable.ttf")[..]),
            Location::caller(),
        )],
    );
    host.wake();
    frame(&mut host);
    assert!(host.has_font_family("Inter Variable"));
    let after = host.frame().expect("registered font frame");
    let changed =
        (0..50).any(|y| (0..150).any(|x| before[y * 160 + x] != after.pixels[y * 161 + x]));
    assert!(changed, "registered family replaces cached fallback glyphs");
    host.memory_warning();
    host.resize(162., 60.);
    frame(&mut host);
    assert!(
        host.has_font_family("Inter Variable"),
        "memory pressure retains registration"
    );
}

fn retained_host(
    scene: std::sync::Arc<kivact_wire::Scene>,
    inline_assets: bool,
    scale: f32,
) -> ArvaneTestHost {
    retained_reporting_host(scene, inline_assets, scale, Rc::default())
}
fn retained_reporting_host(
    scene: std::sync::Arc<kivact_wire::Scene>,
    inline_assets: bool,
    scale: f32,
    reports: Rc<std::cell::RefCell<Vec<u32>>>,
) -> ArvaneTestHost {
    use kivact_engine::bridge::Client;
    use kivact_wire::{Dimension, LayoutProps, NodeId, NodeKind, SceneLayer, TurnOutput};
    ArvaneTestHost::new(
        Client::new(
            move |_, _, writer| {
                writer.create(NodeId(1), NodeKind::Root);
                writer.create(NodeId(2), NodeKind::Box(Default::default()));
                writer.set_layout(
                    NodeId(2),
                    LayoutProps {
                        width: Dimension::Px(160.),
                        height: Dimension::Px(60.),
                        ..Default::default()
                    },
                );
                writer.insert(NodeId(1), NodeId(2), None);
                writer.set_scene(NodeId(2), SceneLayer::Draw, scene.clone());
                writer.commit();
                None
            },
            move |events, _, _| {
                for event in events {
                    if let kivact_wire::HostEvent::DrawDropped { count, .. } = event {
                        reports.borrow_mut().push(*count);
                    }
                }
                TurnOutput::default()
            },
        ),
        TestHostOptions {
            width: 160.,
            height: 60.,
            scale,
            window: WindowMode::Real,
            inline_assets,
            ..Default::default()
        },
    )
    .expect("real retained-scene host")
}

#[test]
fn a_successful_old_svg_raster_is_invalidated_when_the_worker_completes_its_replacement() {
    use kivact_wire::{Fill, Rgba, Scene, SceneCmd};
    use std::sync::Arc;
    static SVG:&[u8]=br##"<svg xmlns="http://www.w3.org/2000/svg" width="12" height="12"><circle cx="6" cy="6" r="5" fill="#0000ff"/></svg>"##;
    let scene = Arc::new(Scene {
        cmds: vec![
            SceneCmd::Rect {
                rect: Rect::new(0., 0., 160., 60.),
                fill: Fill::Solid(Rgba {
                    r: 1.,
                    g: 1.,
                    b: 1.,
                    a: 1.,
                }),
            },
            SceneCmd::Image {
                source: Asset::from(SVG),
                rect: Rect::new(8., 8., 12., 12.),
            },
        ],
        ..Default::default()
    });
    // This allocation is sent exactly once; scale changes never record a new scene.
    let mut host = retained_host(scene.clone(), false, 1.);
    frame(&mut host);
    pump_until(&mut host, |h| pixel(h, 14, 14) == 0x0000ff);
    // Present again with the successful small raster available.
    host.resize(161., 60.);
    frame(&mut host);
    host.set_scale(2.);
    assert!(
        host.redraw(),
        "present exactly the successful fallback frame"
    );
    let fallback = {
        let frame = host.frame().expect("fallback raster frame");
        (0..120)
            .flat_map(|y| frame.pixels[y * 322..y * 322 + 320].iter().copied())
            .collect::<Vec<_>>()
    };
    let mut reference = retained_host(scene.clone(), true, 2.);
    frame(&mut reference);
    let sharp = reference
        .frame()
        .expect("fresh high resolution raster")
        .pixels
        .to_vec();
    assert!(
        fallback != sharp,
        "old small raster is a successful temporary fallback"
    );
    pump_until(&mut host, |h| {
        let current = h.frame().expect("replacement frame");
        (0..120).all(|y| (0..320).all(|x| current.pixels[y * 322 + x] == sharp[y * 320 + x]))
    });
    let ready = host.frame().expect("sharp replacement");
    for y in 0..120 {
        for x in 0..320 {
            assert_eq!(
                ready.pixels[y * 322 + x],
                sharp[y * 320 + x],
                "ready raster {x},{y}"
            );
        }
    }
}

#[test]
fn a_depth_fallback_reports_a_dropped_local_command_only_once() {
    use kivact_wire::{Fill, Rgba, Scene, SceneCmd};
    use std::sync::Arc;
    let mut cmds = vec![SceneCmd::PushTransform([1., 0., 0., 1., 0., 0.]); 256];
    let red = Fill::Solid(Rgba {
        r: 1.,
        g: 0.,
        b: 0.,
        a: 1.,
    });
    cmds.push(SceneCmd::Rect {
        rect: Rect::new(f32::NAN, 0., 2., 2.),
        fill: red.clone(),
    });
    cmds.push(SceneCmd::Rect {
        rect: Rect::new(2., 2., 12., 12.),
        fill: red,
    });
    cmds.extend(std::iter::repeat_n(SceneCmd::PopTransform, 256));
    let reports = Rc::new(std::cell::RefCell::new(Vec::new()));
    let mut host = retained_reporting_host(
        Arc::new(Scene {
            cmds,
            ..Default::default()
        }),
        true,
        1.,
        reports.clone(),
    );
    frame(&mut host);
    host.wake();
    frame(&mut host);
    assert_eq!(pixel(&host, 5, 5), 0xff0000);
    assert_eq!(
        &*reports.borrow(),
        &[1],
        "zero-origin trial diagnostics are discarded before fallback"
    );
}
