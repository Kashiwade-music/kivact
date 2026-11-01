//! Role: Tests that a write builds the paint list only when stacking facts change.
//!
//! The paint list is built again when something may move, show, hide,
//! reorder or reclip nodes. A color alone, or a text color alone, only
//! redraws: the list is not built again (`paint_list_builds`). A change of
//! the facts the list's steps are made of (opacity, a z-index, a clip, how
//! far a text paints) does build it again, and the engine's own checks
//! (the retained list against a fresh build, the kept text styles against
//! a fresh resolution) hold after each change.

use std::cell::RefCell;
use std::rc::Rc;

use kivact_engine::bridge::{Client, Writer};
use kivact_engine::testing::{ArvaneTestHost, TestHostOptions, WindowMode};
use kivact_wire::{
    Dimension, Fill, LayoutProps, NodeId, NodeKind, PaintProps, Rgba, TextData, TextProps,
    TextShadow, TurnOutput,
};

const ROOT: NodeId = NodeId(1);
const COLUMN: NodeId = NodeId(2);
/// The first box (or text) of the column.
const FIRST: u64 = 3;

fn rgb(r: f32, g: f32, b: f32) -> Rgba {
    Rgba { r, g, b, a: 1.0 }
}

fn paint(color: Rgba) -> PaintProps {
    PaintProps {
        background: Some(Fill::Solid(color)),
        ..PaintProps::default()
    }
}

fn start() -> (ArvaneTestHost, Writer) {
    let writer = Rc::new(RefCell::new(None));
    let host = ArvaneTestHost::new(
        Client::new(
            {
                let writer = writer.clone();
                move |_, _, w| {
                    *writer.borrow_mut() = Some(w);
                    None
                }
            },
            |_, _, _| TurnOutput::default(),
        ),
        // A real window, like every other test that draws (design §10).
        TestHostOptions {
            window: WindowMode::Real,
            ..TestHostOptions::default()
        },
    )
    .expect("the test host starts");
    let writer = writer.borrow_mut().take().expect("the client was started");
    (host, writer)
}

/// Draw until the host asks for nothing more.
fn settle(host: &mut ArvaneTestHost) {
    host.wake();
    while host.redraw() {}
}

/// A root and a column holding `count` children of `kind`, 40 x 6 px each
/// (more than the window shows), drawn.
fn column_of(count: u64, kind: impl Fn(u64) -> NodeKind) -> (ArvaneTestHost, Writer) {
    let (mut host, writer) = start();
    writer.create(ROOT, NodeKind::Root);
    writer.create(COLUMN, NodeKind::Box(Default::default()));
    writer.insert(ROOT, COLUMN, None);
    for n in FIRST..FIRST + count {
        let id = NodeId(n);
        writer.create(id, kind(n));
        writer.set_layout(
            id,
            LayoutProps {
                width: Dimension::Px(40.0),
                height: Dimension::Px(6.0),
                ..LayoutProps::default()
            },
        );
        writer.set_paint(id, paint(rgb(1.0, 0.0, 0.0)));
        writer.insert(COLUMN, id, None);
    }
    writer.commit();
    settle(&mut host);
    (host, writer)
}

fn boxes(count: u64) -> (ArvaneTestHost, Writer) {
    column_of(count, |_| NodeKind::Box(Default::default()))
}

fn texts(count: u64) -> (ArvaneTestHost, Writer) {
    column_of(count, |_| {
        NodeKind::Text(TextData {
            content: Some("text".to_owned()),
            spans: None,
        })
    })
}

/// The engine's references (retained against fresh) agree.
fn assert_checked(host: &mut ArvaneTestHost) {
    #[cfg(debug_assertions)]
    {
        let errors = host.self_check();
        assert!(errors.is_empty(), "{errors:?}");
    }
    let _ = host;
}

/// Run `change`, draw, and say whether the paint list was built again.
fn restacks(host: &mut ArvaneTestHost, writer: &Writer, change: impl FnOnce(&Writer)) -> bool {
    let before = host.paint_list_builds();
    change(writer);
    writer.commit();
    settle(host);
    assert_checked(host);
    host.paint_list_builds() > before
}

#[test]
fn a_color_alone_redraws_without_building_the_list_again() {
    let (mut host, writer) = boxes(200);
    assert!(host.paint_list_builds() > 0, "the first frame builds it");
    let middle = NodeId(FIRST + 100);
    let near = NodeId(FIRST);
    for (id, color) in [(near, rgb(0.0, 0.0, 1.0)), (middle, rgb(0.0, 1.0, 0.0))] {
        let built = restacks(&mut host, &writer, |w| {
            w.set_paint(id, paint(color));
        });
        assert!(!built, "a background color changed only {id:?}");
    }
}

#[test]
fn a_change_of_what_the_list_is_made_of_builds_it_again() {
    let (mut host, writer) = boxes(200);
    let id = NodeId(FIRST + 3);
    let changes: Vec<(&str, PaintProps)> = vec![
        (
            "opacity",
            PaintProps {
                opacity: 0.5,
                ..paint(rgb(1.0, 0.0, 0.0))
            },
        ),
        (
            "z-index",
            PaintProps {
                z_index: 3,
                ..paint(rgb(1.0, 0.0, 0.0))
            },
        ),
        (
            "clip",
            PaintProps {
                clip: true,
                ..paint(rgb(1.0, 0.0, 0.0))
            },
        ),
        ("a background removed", PaintProps::default()),
    ];
    for (what, props) in changes {
        let built = restacks(&mut host, &writer, |w| {
            w.set_paint(id, props);
        });
        assert!(built, "{what} changed");
    }
}

#[test]
fn a_text_color_alone_redraws_and_how_far_a_text_paints_builds_the_list_again() {
    let (mut host, writer) = texts(100);
    let style = |color: Rgba, shadow: bool| TextProps {
        color: Some(color),
        shadows: shadow.then(|| {
            vec![TextShadow {
                x: 2.0,
                y: 2.0,
                blur: 8.0,
                color: rgb(0.0, 0.0, 0.0),
            }]
        }),
        ..TextProps::default()
    };
    // The first style is written once; the texts have been read since.
    restacks(&mut host, &writer, |w| {
        w.set_text_style(COLUMN, Some(style(rgb(0.0, 0.0, 1.0), false)));
    });
    let built = restacks(&mut host, &writer, |w| {
        w.set_text_style(COLUMN, Some(style(rgb(0.0, 1.0, 0.0), false)));
    });
    assert!(!built, "a text color changed only");
    let built = restacks(&mut host, &writer, |w| {
        w.set_text_style(COLUMN, Some(style(rgb(0.0, 1.0, 0.0), true)));
    });
    assert!(built, "a text shadow reaches farther");
    let built = restacks(&mut host, &writer, |w| {
        w.set_text_style(COLUMN, Some(style(rgb(1.0, 0.0, 0.0), true)));
    });
    assert!(!built, "the shadow is the same");
}
