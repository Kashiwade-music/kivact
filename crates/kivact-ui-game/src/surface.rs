//! Role: Reusable hard-edged surfaces and decorative drawing vocabulary.
use crate::color::{CYAN, YELLOW};
use crate::fonts::MONO_FAMILY_VAR;
use kivact::component;
use kivact::draw::{Draw, Layer, Path, Stroke};
use kivact::geometry::{Point, Rect};
use kivact::motion::{FrameRate, frame_loop};
use kivact::platform;
use kivact::state::{Value, effect_on, memo, signal};
use kivact::style::color::{ACCENT, BORDER, SURFACE};
use kivact::style::{Transform, style, use_theme};
use kivact::view::{View, node, node_ref, text};

/// Monospaced caption used for IDs, shortcuts and machine-readable values.
#[component]
pub fn mono(label: Value<String>) -> impl Element {
    text(label).style(move || {
        let t = use_theme().get();
        style()
            .font_family(t.get(&MONO_FAMILY_VAR).as_str().unwrap_or_default())
            .font_size(10.5)
            .font_weight(600)
    })
}

/// Surface with opposing 10 px HUD brackets.
#[component]
pub fn hud(content: View) -> impl Element {
    node()
        .style(style().border(1, BORDER).background(SURFACE))
        .draw_over(|d, s| {
            let c = ACCENT;
            d.line(Point::new(0., 10.), Point::new(0., 0.), 2., c);
            d.line(Point::new(0., 0.), Point::new(10., 0.), 2., c);
            d.line(
                Point::new(s.width - 10., s.height),
                Point::new(s.width, s.height),
                2.,
                c,
            );
            d.line(
                Point::new(s.width, s.height),
                Point::new(s.width, s.height - 10.),
                2.,
                c,
            );
        })
        .child(content)
}

/// Concentric command halo with built-in decorative rotation, respecting reduced motion.
#[component]
pub fn halo() -> impl Element {
    let time = signal(0_f64);
    let node_ref = node_ref();
    let presence = node_ref.presence();
    let clock = frame_loop(move |frame| time.set(frame.now.as_secs_f64()))
        .while_present(move || presence.get() == kivact::view::Presence::Present);
    let running = memo(move || {
        let visible = node_ref.rect().get().is_some_and(|rect| {
            rect.y < platform::window_size().get().height && rect.y + rect.height > 0.
        });
        visible && !platform::is_motion_reduced().get() && platform::is_surface_active().get()
    });
    let run = move |running: bool| {
        if running {
            clock.start(FrameRate::Hz(30.));
        } else {
            clock.stop();
        }
    };
    run(running.get_untracked());
    effect_on(running, move |running| run(*running));
    node()
        .node_ref(node_ref)
        .style(style().size(180, 180))
        .draw(move |d, s| {
            // Reduced motion paints statically immediately; active frames use the absolute clock.
            let time = if platform::is_motion_reduced().get() {
                0.
            } else {
                time.get()
            };
            d.layer(
                Layer::new().transform(Transform::new().scale_xy(s.width / 180., s.height / 180.)),
                |d| {
                    let c = Point::new(90., 90.);
                    for (radius, width, count) in [(82., 1.4, 64), (68., 5., 12), (50., 2., 1)] {
                        for i in 0..count {
                            let rotation = if count == 64 {
                                time * 360. / if s.width < 80. { 24. } else { 40. }
                            } else if count == 12 {
                                -time * 360. / 26.
                            } else {
                                0.
                            };
                            let start = i as f64 * 360. / count as f64 + rotation;
                            let end = if count == 1 {
                                359.8
                            } else {
                                start + 360. / count as f64 * 0.66
                            };
                            let path = Path::new().arc(c, radius, start, end, false);
                            d.stroke_path(
                                &path,
                                &Stroke::new(width),
                                if count == 1 { CYAN } else { ACCENT },
                            );
                        }
                    }
                    draw_halo_brackets(d, c);
                    d.fill_path(
                        &Path::new()
                            .move_to(Point::new(c.x, c.y - 25.))
                            .line_to(Point::new(c.x + 14., c.y))
                            .line_to(Point::new(c.x, c.y + 25.))
                            .line_to(Point::new(c.x - 14., c.y))
                            .close(),
                        ACCENT,
                    );
                    d.fill_rect(Rect::new(128., 36., 14., 7.), YELLOW);
                },
            );
        })
}

fn draw_halo_brackets(d: &mut Draw, c: Point) {
    for (start, end) in [
        (Point::new(c.x, 4.), Point::new(c.x, 29.)),
        (Point::new(c.x, 151.), Point::new(c.x, 176.)),
        (Point::new(4., c.y), Point::new(29., c.y)),
        (Point::new(151., c.y), Point::new(176., c.y)),
    ] {
        d.line(start, end, 1., ACCENT);
    }
}
