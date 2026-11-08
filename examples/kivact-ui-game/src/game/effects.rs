//! Role: The Effects chapter: 3D, filters, shaders, timelines and scroll motion.

use super::common::{cards, specimen};
use kivact::draw::Paint;
use kivact::draw::filter::Channel;
use kivact::draw::{Filter, FilterGraph, FilterInput, Layer, PaintShader, Transform};
use kivact::geometry::Point;
use kivact::motion::{
    Animated, At, Easing, FrameRate, Spring, Stagger, StaggerFrom, Timeline, TimelineTrack,
    Transition, animated, timeline,
};
use kivact::state::signal;
use kivact::style::{
    Align, BackfaceVisibility, ColorToken, Justify, Position, TransformStyle, pct, style, use_theme,
};
use kivact::ui::{button, heading};
use kivact::view::{
    IntoView, Node, NodeBuilder, NodeRef, Orientation, canvas, column, drawable, node, node_ref,
    pointer_position, row, scroll, scroll_progress, text,
};
use kivact_ui_game::{ColorName, Tokens, cut_corner, mono};

/// Scanline paint: a fragment of stripes that drifts with the clock.
const SCANLINE: &str = "
struct Params { speed: f32, tint: vec4<f32> }

fn paint(position: vec2<f32>, context: Context, params: Params) -> vec4<f32> {
    let wave = 0.5 + 0.5 * sin(position.y * 0.35 + context.time * params.speed);
    let scan = 0.8 + 0.2 * step(0.5, fract(position.y * 0.5));
    let alpha = params.tint.a * (0.25 + 0.35 * wave);
    return vec4<f32>(params.tint.rgb * scan, alpha);
}
";

/// A card's level-3 heading, given the height its text needs to stay inside its box.
fn sample_heading(label: &str) -> impl IntoView {
    heading(label.to_owned(), 3).style(style().min_height(28))
}

/// The seven Effects samples, in two columns of cards.
pub(super) fn effects() -> Node {
    cards(
        (
            specimen("FX-01", "パララックス HUD", parallax_hud()),
            specimen("FX-02", "3D の傾きカード", tilt_card()),
            specimen("FX-03", "3D の設定パネル", settings_panel()),
            specimen("FX-04", "フィルターグラフの glitch", glitch_text()),
            specimen("FX-05", "shader の scanline 背景", scanline_background()),
            specimen("FX-06", "タイムラインの boot sequence", boot_sequence()),
            specimen("FX-07", "scroll に連動するヘッダー", scroll_header()),
        ),
        460.,
    )
}

/// The pointer as offsets in -1..=1 from the centre of `frame`; zero outside the window.
fn pointer_normal(frame: NodeRef) -> impl Fn() -> Point + Copy {
    let pointer = pointer_position();
    move || match (pointer.get(), frame.rect().get()) {
        (Some(p), Some(r)) => Point::new(
            ratio(p.x - r.center().x, r.width / 2.),
            ratio(p.y - r.center().y, r.height / 2.),
        ),
        _ => Point::new(0., 0.),
    }
}

fn ratio(delta: f64, half: f64) -> f64 {
    if half > 0. {
        (delta / half).clamp(-1., 1.)
    } else {
        0.
    }
}

/// Two springs that follow the pointer, `depth` px at the edge of the frame.
fn offset(
    normal: impl Fn() -> Point + Copy + 'static,
    depth: f64,
) -> (Animated<f64>, Animated<f64>) {
    (
        animated(
            move || normal().x * depth,
            Transition::spring(Spring::GENTLE),
        ),
        animated(
            move || normal().y * depth,
            Transition::spring(Spring::GENTLE),
        ),
    )
}

fn parallax_hud() -> Node {
    let frame = node_ref();
    let normal = pointer_normal(frame);
    let (bx, by) = offset(normal, 6.);
    let (mx, my) = offset(normal, 14.);
    let (fx, fy) = offset(normal, 26.);
    node()
        .with_node(|n| n.node_ref(frame))
        .style(
            style()
                .height(180)
                .position(Position::Relative)
                .border(1, ColorToken::Border)
                .background(ColorToken::Surface),
        )
        .child((
            text("SECTOR 07").style(move || {
                style()
                    .position(Position::Absolute)
                    .top(14)
                    .left(18)
                    .font_family("Barlow Condensed")
                    .font_size(64)
                    .weight(800)
                    .italic(true)
                    .color(ColorName::LineStrong)
                    .translate(bx.get(), by.get())
            }),
            node()
                .style(move || {
                    style()
                        .position(Position::Absolute)
                        .top(84)
                        .left(180)
                        .width(160)
                        .height(44)
                        .background(ColorName::Cyan)
                        .clip_path(cut_corner(12.))
                        .translate(mx.get(), my.get())
                })
                .child(mono("TARGET LOCK").style(style().color(ColorName::Navy))),
            column((
                sample_heading("HUD のパララックス"),
                mono("pointer_position + spring"),
            ))
            .style(move || {
                style()
                    .position(Position::Absolute)
                    .bottom(16)
                    .left(18)
                    .gap(4)
                    .translate(fx.get(), fy.get())
            }),
        ))
}

fn tilt_card() -> Node {
    // The stage is not transformed; the card's own window rect would feed its tilt back.
    let stage = node_ref();
    let normal = pointer_normal(stage);
    let tilt_x = animated(
        move || -normal().y * 14.,
        Transition::spring(Spring::GENTLE),
    );
    let tilt_y = animated(move || normal().x * 18., Transition::spring(Spring::GENTLE));
    let flipped = signal(false);
    let turn = animated(
        move || if flipped.get() { 180. } else { 0. },
        Transition::tween(480., Easing::EaseOut),
    );
    let front = face(
        column((
            mono("CARD-07").style(style().color(ColorToken::Accent)),
            sample_heading("傾きと裏返し"),
            text("ポインタで傾き、ボタンで裏返す。").style(style().color(ColorToken::TextMuted)),
        ))
        .style(style().gap(8)),
    )
    .style(style().backface_visibility(BackfaceVisibility::Hidden));
    let back = face(
        column((
            mono("BACK"),
            text("裏面は backface-visibility で隠れている。"),
        ))
        .style(style().gap(8)),
    )
    .style(
        style()
            .rotate_y(180.)
            .backface_visibility(BackfaceVisibility::Hidden),
    );
    column((
        node()
            .with_node(|n| n.node_ref(stage))
            .style(style().height(190).perspective(900.))
            .child(
                node()
                    .style(move || {
                        style()
                            .height(pct(100))
                            .rotate_x(tilt_x.get())
                            .rotate_y(tilt_y.get() + turn.get())
                            .transform_style(TransformStyle::Preserve3d)
                    })
                    .child((front, back)),
            ),
        row(button("カードを裏返す").on_press(move || flipped.update(|v| *v = !*v)))
            .style(style().justify_content(Justify::End)),
    ))
    .style(style().gap(10))
}

/// A card face: the kit's surface with a clipped corner.
fn face(content: impl kivact::view::IntoView) -> Node {
    node()
        .style(
            style()
                .position(Position::Absolute)
                .top(0)
                .left(0)
                .width(pct(100))
                .height(pct(100))
                .padding(16)
                .background(ColorToken::SurfaceAlt)
                .border(1, ColorToken::Border)
                .clip_path(cut_corner(16.)),
        )
        .child(content)
}

fn settings_panel() -> Node {
    let panel = drawable();
    let open = signal(false);
    let angle = animated(
        move || if open.get() { 0. } else { -28. },
        Transition::spring(Spring::GENTLE),
    );
    let view = canvas(move |d, size| {
        let w = panel.size().get();
        let at = Point::new(((size.width - w.width) / 2.).max(0.), 24.);
        let transform = Transform::new().perspective(900.).rotate_y(angle.get());
        d.layer(Layer::new().transform(transform), |d| d.drawable(panel, at));
    })
    .style(style().height(260).width(pct(100)));
    column((
        view.drawable(panel, settings_body),
        row(button("パネルを回す").on_press(move || open.update(|v| *v = !*v)))
            .style(style().justify_content(Justify::End)),
    ))
    .style(style().gap(10))
}

/// The content of the settings panel, mounted as a drawable and drawn rotated.
fn settings_body() -> Node {
    column((
        mono("SETTINGS // PANEL-03"),
        sample_heading("表示の設定"),
        text("ポインタの入力も hit test も、回転した絵の位置で働く。"),
    ))
    .style(
        style()
            .width(340)
            .padding(18)
            .gap(8)
            .background(ColorToken::Surface)
            .border(1, ColorToken::Border)
            .clip_path(cut_corner(14.)),
    )
}

fn glitch_text() -> Node {
    let mut graph = FilterGraph::new();
    let noise = graph.turbulence(0.02, 2, 3.0);
    graph.displacement_map(FilterInput::SOURCE, noise, 12.0, Channel::R, Channel::G);
    let glitch = Filter::new().graph(graph);
    column((
        text("SIGNAL LOST").style(
            style()
                .font_family("Barlow Condensed")
                .font_size(56)
                .weight(800)
                .italic(true)
                .color(ColorToken::Accent)
                .filter(glitch),
        ),
        mono("filter graph: turbulence -> displacement_map")
            .style(style().color(ColorToken::TextMuted)),
    ))
    .style(style().gap(10).padding(12))
}

fn scanline_background() -> Node {
    let shader = PaintShader::wgsl(SCANLINE).expect("invariant: the scanline shader is valid WGSL");
    node()
        .style(move || {
            let theme = &use_theme().get();
            let tint = theme.ext::<Tokens>().cyan;
            let paint = shader
                .paint()
                .param("speed", 0.6)
                .param("tint", tint)
                .animate(FrameRate::Hz(30.))
                .fallback(ColorName::Navy);
            style()
                .height(160)
                .padding(16)
                .justify_content(Justify::End)
                .background(Paint::from(paint))
                .border(1, ColorToken::Border)
        })
        .child(column((
            mono("ShaderPaint // CPU は fallback を描く"),
            sample_heading("Scanline"),
        )))
}

fn boot_sequence() -> Node {
    let sequence: Timeline = timeline();
    let lines = ["CORE", "SHADERS", "TOKENS", "READY"];
    let stagger = Stagger::new(180.)
        .from(StaggerFrom::Start)
        .easing(Easing::EaseOut);
    let tracks: Vec<TimelineTrack<f64>> = lines
        .iter()
        .enumerate()
        .map(|(i, _)| {
            let track = sequence.track(0.0);
            sequence
                .to(track, 1.0, Transition::tween(260., Easing::EaseOut))
                .at(At::Time(stagger.delay(i, lines.len())));
            track
        })
        .collect();
    sequence.play();
    let rows: Vec<Node> = lines
        .iter()
        .zip(tracks)
        .enumerate()
        .map(|(i, (name, track))| boot_row(i, name, track))
        .collect();
    column((
        column(rows).style(style().gap(6)),
        row(button("再生し直す").on_press(move || sequence.restart()))
            .style(style().justify_content(Justify::End)),
    ))
    .style(style().gap(12))
}

fn boot_row(index: usize, name: &str, track: TimelineTrack<f64>) -> Node {
    row((
        mono(format!("{:02}", index + 1)).style(style().color(ColorToken::TextMuted)),
        text(name.to_owned()).style(style().weight(700)),
    ))
    .style(move || {
        let t = track.get();
        style()
            .gap(12)
            .opacity(t)
            .translate_x((1. - t) * -24.)
            .border_bottom(1, ColorToken::Border)
            .padding_y(6)
    })
}

fn scroll_header() -> Node {
    let scroller = node_ref();
    let progress = scroll_progress(scroller, Orientation::Vertical);
    let header = row((mono("HEADER"), sample_heading("スクロールに連動"))).style(move || {
        let p = progress.get();
        style()
            .height(56. - 24. * p)
            .align_items(Align::Center)
            .gap(12)
            .opacity(1. - 0.6 * p)
            .translate_y(-6. * p)
            .border_bottom(1, ColorToken::Border)
    });
    let items: Vec<_> = (1..=12)
        .map(|i| {
            mono(format!(
                "ROW {i:02}  scroll_progress の値は内側の scroll から読む"
            ))
        })
        .collect();
    column((
        header,
        scroll(column(items))
            .with_node(|n| n.node_ref(scroller))
            .style(style().height(120)),
    ))
    .style(style().gap(4))
}
