//! Role: Painting of text nodes and scene text runs with their decorations.
//!
//! Text painted: a text node's paragraph (plain or rich) and a scene's
//! text runs, with their highlight bands, shadows and decorations.
//!
//! Invariants:
//! - Plain text is shaped without its color and colored as it is drawn
//!   (a hover's new color reuses the shaping), and so are its decoration
//!   lines in the text's color (`currentColor`). Rich text is shaped with
//!   its colors drawn through the node's tint (its spans color their own
//!   glyphs); colors never change a measurement either way.
//! - The paint order is CSS's: the spans' backgrounds (bands, one per span
//!   per line, filled like a box background), the text shadows (the last
//!   lowest; glyphs and decoration lines in the shadow's color, offset,
//!   blurred in a layer by half the blur radius), underlines and
//!   overlines, the glyphs, then line-throughs.
//! - The paint list counts a text's shadows in its reach (`text_reach`),
//!   so culling never drops a shadow's pixels.

use crate::lower::effects::Tint;
use kivact_types::render::RenderCmd as D;
use kivact_types::{ComputedTextStyle, Rect, TextMetrics, TextRun, TextShadow};
use kivact_types::{Point, Rgba};

// xtask-allow(state-fields): Glyph and decoration overrides are independent optional drawing parameters.
/// How a shaped text is colored as it is drawn.
pub(crate) struct Colors {
    /// Every glyph run's color (plain text shaped without its color).
    pub(crate) glyphs: Option<Rgba>,
    /// Every decoration line's color (plain text's `currentColor`).
    pub(crate) decorations: Option<Rgba>,
}

/// `shaped` at `origin`: its shadows, decorations and glyphs in paint
/// order (see the module invariants), colored by `colors`.
pub(crate) fn push_text(
    out: &mut crate::lower::SceneOut,
    shaped: &kivact_types::render::ShapedText,
    origin: Point,
    colors: &Colors,
    shadows: &[TextShadow],
    tint: &Tint,
    device: f64,
) {
    for shadow in shadows.iter().rev() {
        let color = tint.color(shadow.color);
        if color.a <= 0.0 || !(shadow.x.is_finite() && shadow.y.is_finite()) {
            continue;
        }
        let at = Point {
            x: origin.x + shadow.x,
            y: origin.y + shadow.y,
        };
        let blurred = shadow.blur.is_finite() && shadow.blur > 0.0;
        if blurred {
            out.push(D::PushLayer(kivact_types::render::Layer {
                filter: Some(kivact_types::render::LayerFilter::Blur {
                    std_dev: shadow.blur / 2.0,
                }),
                ..kivact_types::render::Layer::default()
            }));
        }
        let all = Colors {
            glyphs: Some(color),
            decorations: Some(color),
        };
        lines(out, shaped, at, &all, false, device);
        glyphs(out, shaped, at, all.glyphs);
        lines(out, shaped, at, &all, true, device);
        if blurred {
            out.push(D::PopLayer);
        }
    }
    lines(out, shaped, origin, colors, false, device);
    glyphs(out, shaped, origin, colors.glyphs);
    lines(out, shaped, origin, colors, true, device);
}

/// The glyph runs of `shaped` at `origin`, in `color` when given.
fn glyphs(
    out: &mut crate::lower::SceneOut,
    shaped: &kivact_types::render::ShapedText,
    origin: Point,
    color: Option<Rgba>,
) {
    for run in &shaped.runs {
        out.push(D::Glyphs {
            run: std::sync::Arc::clone(run),
            origin,
            color,
        });
    }
}

/// The decoration lines of `shaped` drawn after the glyphs (`through`:
/// line-throughs) or before them (underlines and overlines), their top
/// and thickness on whole `device` pixels (crisp, like the baselines the
/// renderers align).
fn lines(
    out: &mut crate::lower::SceneOut,
    shaped: &kivact_types::render::ShapedText,
    origin: Point,
    colors: &Colors,
    through: bool,
    device: f64,
) {
    let device = if device.is_finite() && device > 0.0 {
        device
    } else {
        1.0
    };
    for line in &shaped.decorations {
        if (line.line == kivact_types::DecorationLines::LINE_THROUGH) != through {
            continue;
        }
        let rect = Rect {
            x: line.rect.x + origin.x,
            y: ((line.rect.y + origin.y) * device).round() / device,
            width: line.rect.width,
            height: (line.rect.height * device).round().max(1.0) / device,
        };
        let paint = kivact_types::render::Paint::Solid(colors.decorations.unwrap_or(line.color));
        decoration(out, rect, paint, line.style);
    }
}

/// How far a text's paint reaches out of its box: a font size (glyphs
/// overhang their lines), and its shadows' offsets and blurs.
pub(crate) fn text_reach(font_size: f64, shadows: &[TextShadow]) -> f64 {
    let shadows = shadows
        .iter()
        .map(|s| {
            s.x.abs().max(s.y.abs()) + s.blur.max(0.0) / 2.0 * kivact_types::render::SHADOW_REACH
        })
        .fold(0.0f64, f64::max);
    font_size + shadows
}

/// Whether `run` needs its node's inherited text style (it leaves its
/// family or its line height unset).
pub(crate) fn inherits(run: &TextRun) -> bool {
    run.family.is_none() || run.line_height.is_none()
}

/// The Arvane style of a scene's text `run` (its color as written), a
/// run without a family or a line height in the `inherited` text style's
/// (`None`: the defaults).
pub(crate) fn run_style(
    engine: &mut arvane::Engine,
    run: &TextRun,
    inherited: Option<&ComputedTextStyle>,
) -> kivact_types::ComputedTextStyle {
    let c = kivact_types::Rgba::sanitize;
    let family = run
        .family
        .as_deref()
        .or(inherited.and_then(|t| t.family.as_deref()));
    let line_height = run.line_height.or(inherited.map(|t| t.line_height));
    kivact_types::ComputedTextStyle {
        font_size: run.font_size,
        weight: run.weight,
        color: c(run.color),
        line_height: crate::lower::conversion::line_height(line_height),
        family: crate::lower::assets::family(engine, family),
        italic: run.italic,
        letter_spacing: if run.letter_spacing.is_finite() {
            kivact_types::TextLen::Px(run.letter_spacing)
        } else {
            kivact_types::TextLen::Px(0.0)
        },
        word_spacing: if run.word_spacing.is_finite() {
            kivact_types::TextLen::Px(run.word_spacing)
        } else {
            kivact_types::TextLen::Px(0.0)
        },
        kerning: run.kerning,
        features: run.features.clone(),
        decoration: kivact_types::TextDecoration {
            color: run.decoration.color.map(kivact_types::Rgba::sanitize),
            ..run.decoration
        },
        ..Default::default()
    }
}

/// The extent a scene's text `run` is drawn with (zero when it cannot be
/// shaped): what `Draw::measure_text` answers. The run is shaped with the
/// style it is painted with (`run_style`: its family and line height, else
/// the node's inherited ones), so the size and first baseline are those of
/// what is drawn; `TextMeasurer` and `LeafTextMeasurer` measure the same.
pub(crate) fn measure_run(
    engine: &mut arvane::Engine,
    run: &TextRun,
    inherited: Option<&ComputedTextStyle>,
) -> TextMetrics {
    let style = run_style(engine, run, inherited);
    if !(style.font_size.is_finite() && style.font_size > 0.0) {
        return TextMetrics::default();
    }
    engine
        .text_metrics(run.text.as_str(), &style, None)
        .unwrap_or_default()
}

/// `run`'s text shaped as a paragraph with its carets and lines, wrapped at
/// `max_width` (`None`, or a width that is not finite: one line per hard
/// line), in the style `measure_run` measures it with. The shaping is
/// the engine's cached one (colors aside), so equal runs share one `Arc`
/// and what `DrawCmd::Paragraph` draws is what was shaped. `None` when it
/// cannot be shaped.
pub(crate) fn shape_run(
    engine: &mut arvane::Engine,
    run: &TextRun,
    inherited: Option<&ComputedTextStyle>,
    max_width: Option<f64>,
) -> Option<std::sync::Arc<kivact_types::render::ShapedText>> {
    let style = run_style(engine, run, inherited);
    if !(style.font_size.is_finite() && style.font_size > 0.0) {
        return None;
    }
    let max_width = max_width.filter(|w| w.is_finite()).map(|w| w.max(0.0));
    engine.shape(run.text.as_str(), &style, max_width).ok()
}

/// Shape a scene's text around measured objects using its inherited typography.
pub(crate) fn shape_inline_run(
    engine: &mut arvane::Engine,
    run: &TextRun,
    inherited: Option<&ComputedTextStyle>,
    max_width: Option<f64>,
    boxes: &[kivact_types::InlineBox],
) -> Option<kivact_types::ShapedInlineText> {
    let style = run_style(engine, run, inherited);
    engine
        .shape_inline(run.text.as_str(), &style, max_width, boxes)
        .ok()
}

/// Paint one resolved decoration band, retaining its pattern.
pub(crate) fn decoration(
    out: &mut crate::lower::SceneOut,
    rect: Rect,
    paint: kivact_types::render::Paint,
    style: kivact_types::TextDecorationStyle,
) {
    use kivact_types::scene::Shape;
    use kivact_types::{FillRule, LineCap, PathCmd, Stroke, TextDecorationStyle};
    if style == TextDecorationStyle::Solid {
        out.push(D::Rect { rect, fill: paint });
        return;
    }
    let width = rect.height;
    let y = rect.y + width / 2.0;
    let mut cmds = vec![PathCmd::MoveTo(Point::new(rect.x, y))];
    let mut stroke = Stroke::new(width);
    match style {
        TextDecorationStyle::Dashed => stroke.dash = vec![3.0 * width, 3.0 * width],
        TextDecorationStyle::Dotted => {
            stroke.cap = LineCap::Round;
            stroke.dash = vec![0.0, 2.0 * width];
        }
        TextDecorationStyle::Wavy => {
            let mut x = rect.x;
            let end = rect.x + rect.width;
            let mut direction = 1.0;
            while x < end {
                let next = (x + 4.0 * width).min(end);
                let step = next - x;
                cmds.push(PathCmd::CubicTo(
                    Point::new(x + step / 3.0, y + direction * width * 2.0),
                    Point::new(x + step * 2.0 / 3.0, y + direction * width * 2.0),
                    Point::new(next, y),
                ));
                x = next;
                direction = -direction;
            }
        }
        TextDecorationStyle::Solid => {}
    }
    if style != TextDecorationStyle::Wavy {
        cmds.push(PathCmd::LineTo(Point::new(rect.x + rect.width, y)));
    }
    out.push(D::StrokeShape {
        shape: Shape::Path {
            cmds,
            rule: FillRule::NonZero,
        },
        stroke,
        paint,
    });
}
