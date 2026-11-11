//! Role: Resolve a `Style`'s paint, borders, effects and transforms.

use kivact_runtime::{Transition, UiError};
use kivact_types as p;
use kivact_types::{
    Border, Corners, Edges, Effects, ElementTransform, Fill, PaintDelta, PaintProps, Perspective,
    Repeat, Ring,
};

use crate::effect::{Shadow, clip_shape, filter_ops, filter_problems};
use crate::geometry::corner;
use crate::length::{Length, pct};
use crate::paint::{Background, fill_layer};
use crate::sheet::{
    FlowFields, MotionFields, PaintFields, ScrollbarWidth, Style, TextFields, TransformFields,
    group_or_default,
};
use crate::theme::value::Theme;
use crate::transform::Transform;
use kivact_types::Overflow;

/// A background as the engine fills it: one color covering the box stays a
/// plain color (the common case costs no allocation).
fn fill(b: &Background, theme: &Theme) -> Fill {
    match b.solid() {
        Some(c) => Fill::Solid(c.resolve(theme)),
        None => Fill::Layers(b.0.iter().map(|l| fill_layer(l, theme)).collect()),
    }
}

/// The border's widths (0 where unset): the layout's border, as the
/// paint's ([`border`]).
pub(super) fn border_widths(s: &Style) -> Edges<f64> {
    let paint = group_or_default!(s.paint, PaintFields);
    let w = |i: usize| paint.border_width[i].unwrap_or(0.0);
    Edges {
        top: w(0),
        right: w(1),
        bottom: w(2),
        left: w(3),
    }
}

/// The border, when the style sets any of its properties (a width without
/// a color takes the theme's border color).
fn border(s: &Style, theme: &Theme) -> Option<Border> {
    let paint = group_or_default!(s.paint, PaintFields);
    let set = paint.border_width.iter().any(Option::is_some)
        || paint.border_color.iter().any(Option::is_some)
        || paint.border_style.iter().any(Option::is_some);
    set.then(|| {
        let c = |i: usize| {
            paint.border_color[i]
                .unwrap_or(theme.colors().border)
                .resolve(theme)
        };
        Border {
            widths: border_widths(s),
            colors: p::Edges {
                top: c(0),
                right: c(1),
                bottom: c(2),
                left: c(3),
            },
            styles: p::Edges {
                top: paint.border_style[0].unwrap_or_default(),
                right: paint.border_style[1].unwrap_or_default(),
                bottom: paint.border_style[2].unwrap_or_default(),
                left: paint.border_style[3].unwrap_or_default(),
            },
        }
    })
}

fn corners(s: &Style) -> Corners {
    let paint = group_or_default!(s.paint, PaintFields);
    Corners {
        top_left: corner(paint.radius[0]),
        top_right: corner(paint.radius[1]),
        bottom_right: corner(paint.radius[2]),
        bottom_left: corner(paint.radius[3]),
    }
}

fn shadow(sh: &Shadow, theme: &Theme) -> p::Shadow {
    p::Shadow {
        offset: sh.offset,
        blur: sh.blur,
        spread: sh.spread,
        color: sh.color.resolve(theme),
        inset: sh.inset,
    }
}

/// The effects, when the style sets any (`None` otherwise, so a node
/// without effects carries none).
fn effects(s: &Style, theme: &Theme) -> Option<Box<Effects>> {
    let paint = group_or_default!(s.paint, PaintFields);
    let set = paint.filter.is_some()
        || paint.backdrop_filter.is_some()
        || paint.blend.is_some()
        || paint.mask.is_some()
        || paint.clip_path.is_some();
    set.then(|| {
        Box::new(Effects {
            filter: paint
                .filter
                .as_ref()
                .map(|f| filter_ops(f, theme))
                .unwrap_or_default(),
            backdrop: paint
                .backdrop_filter
                .as_ref()
                .map(|f| filter_ops(f, theme))
                .unwrap_or_default(),
            blend: paint.blend.unwrap_or_default(),
            masks: paint
                .mask
                .as_ref()
                .map(|m| m.wire(theme))
                .unwrap_or_default(),
            clip_path: paint.clip_path.as_ref().map(|c| clip_shape(&c.0)),
        })
    })
}

/// The transform properties of `s` as one transform: CSS's `translate`
/// (x, y, z), then `rotate` (z, then y, then x), then `scale`, then
/// `transform`, about `transform-origin` (the centre by default).
fn transform_of(s: &Style) -> Transform {
    let transform = group_or_default!(s.transform, TransformFields);
    let mut t = Transform::new();
    if transform.translate.iter().any(Option::is_some) || transform.translate_z.is_some() {
        // An axis left unset does not move the node.
        let [x, y] = transform
            .translate
            .map(|axis| axis.unwrap_or(Length::Px(0.0)));
        t = t.translate_3d(x, y, transform.translate_z.unwrap_or(0.0));
    }
    if let Some(deg) = transform.rotate {
        t = t.rotate_z(deg);
    }
    if let Some(deg) = transform.rotate_y {
        t = t.rotate_y(deg);
    }
    if let Some(deg) = transform.rotate_x {
        t = t.rotate_x(deg);
    }
    if let Some((x, y)) = transform.scale {
        t = t.scale_xy(x, y);
    }
    if let Some(list) = &transform.transform {
        t = t.then(list);
    }
    t
}

/// The transform properties of `s` as one element transform, about
/// `transform-origin` (the centre by default) and its z.
fn element_transform(s: &Style) -> ElementTransform {
    let transform = group_or_default!(s.transform, TransformFields);
    let (x, y) = transform.transform_origin.unwrap_or((pct(50), pct(50)));
    // `auto` is the default centre.
    transform_of(s).to_wire(
        [x, y].map(|l| match l {
            Length::Auto => p::Len::pct(50.0),
            l => l.to_len(),
        }),
        transform.transform_origin_z.unwrap_or(0.0),
    )
}

/// Whether `s` sets a transform property.
fn sets_transform(s: &Style) -> bool {
    let transform = group_or_default!(s.transform, TransformFields);
    transform.translate.iter().any(Option::is_some)
        || transform.translate_z.is_some()
        || transform.rotate.is_some()
        || transform.rotate_x.is_some()
        || transform.rotate_y.is_some()
        || transform.scale.is_some()
        || transform.transform.is_some()
        || transform.transform_origin.is_some()
        || transform.transform_origin_z.is_some()
}

/// The perspective of the node's children: the distance, about the
/// `perspective-origin` (the centre by default).
fn perspective(s: &Style) -> Option<Perspective> {
    let transform = group_or_default!(s.transform, TransformFields);
    transform.perspective.map(|distance| Perspective {
        distance,
        origin: transform
            .perspective_origin
            .map_or([p::Len::pct(50.0), p::Len::pct(50.0)], |(x, y)| {
                [x.to_len(), y.to_len()]
            }),
    })
}

pub(super) fn paint(s: &Style, theme: &Theme) -> PaintProps {
    let paint = group_or_default!(s.paint, PaintFields);
    let transform = group_or_default!(s.transform, TransformFields);
    let flow = group_or_default!(s.flow, FlowFields);
    PaintProps {
        transitions: None,
        background: paint.background.as_ref().map(|b| fill(b, theme)),
        border: border(s, theme),
        ring: paint.ring.map(|(width, offset, c)| Ring {
            width,
            offset,
            color: c.resolve(theme),
        }),
        radius: corners(s),
        shadows: paint
            .shadows
            .iter()
            .flatten()
            .map(|sh| shadow(sh, theme))
            .collect(),
        opacity: paint.opacity.unwrap_or(1.0),
        transform: element_transform(s),
        perspective: perspective(s),
        transform_style: transform.transform_style.unwrap_or_default(),
        backface: transform.backface.unwrap_or_default(),
        will_change: transform.will_change.unwrap_or_default(),
        isolation: paint.isolation.unwrap_or_default(),
        border_image: paint.border_image.as_ref().map(|b| Box::new(b.wire(theme))),
        z_index: flow.z_index.unwrap_or(0),
        scrollbar: scrollbar(s, theme),
        cursor: paint.cursor.unwrap_or_default(),
        clip: paint.clip.unwrap_or(false)
            || flow.scroll_axes.is_some()
            || flow.overflow.is_some_and(|o| o != Overflow::Visible),
        effects: effects(s, theme),
    }
}

/// The paint transitions of `s`: every property but the size, which the
/// layout takes.
pub(super) fn paint_transitions(
    s: &Style,
) -> Option<std::sync::Arc<[(crate::sheet::TransitionProp, Transition)]>> {
    let motion = group_or_default!(s.motion, MotionFields);
    let list: Vec<_> = motion
        .transitions
        .iter()
        .filter(|(prop, _)| *prop != crate::sheet::TransitionProp::Size)
        .cloned()
        .collect();
    (!list.is_empty()).then(|| list.into())
}

/// What `compile_style` reports about the style itself (see
/// [`CompiledStyle::reports`]).
pub(super) fn reports(s: &Style, theme: &Theme) -> Vec<UiError> {
    let paint = group_or_default!(s.paint, PaintFields);
    let motion = group_or_default!(s.motion, MotionFields);
    let mut out = Vec::new();
    let mut report = |error: UiError| out.push(error);
    let springs = motion
        .transitions
        .iter()
        .map(|(_, t)| t)
        .chain(motion.layout_transition.iter());
    for t in springs {
        if let p::Kind::Spring(spring) = &t.kind
            && let Some(problem) = spring.problem()
        {
            report(UiError::msg(format!("spring: {problem}")));
        }
    }
    if let Some(t) = &motion.layout_transition
        && (t.repeat != Repeat::Times(1) || t.direction != p::PlayDirection::Normal)
    {
        report(UiError::msg(
            "layout_transition ignores repeat and alternate",
        ));
    }
    if let Some(background) = &paint.background {
        for paint in &background.0 {
            for problem in paint.shader_problems() {
                report(problem);
            }
        }
    }
    if let Some(mask) = &paint.mask {
        for layer in &mask.0 {
            for problem in layer.paint().shader_problems() {
                report(problem);
            }
        }
    }
    for filter in [&paint.filter, &paint.backdrop_filter]
        .into_iter()
        .flatten()
    {
        for problem in filter_problems(filter, theme) {
            report(problem);
        }
    }
    let dropped = transform_of(s).unsupported();
    if dropped > 0 {
        report(UiError::msg(format!(
            "transform: {dropped} function(s) the form cannot keep"
        )));
    }
    out
}

/// The paint a state style `st` changes over the node's style `base`:
/// each group it sets, resolved over the base (a state that sets only a
/// border color keeps the base's widths).
pub(super) fn paint_delta(base: &Style, st: &Style, theme: &Theme) -> PaintDelta {
    let st_paint = group_or_default!(st.paint, PaintFields);
    let st_transform = group_or_default!(st.transform, TransformFields);
    let st_flow = group_or_default!(st.flow, FlowFields);
    let st_text = group_or_default!(st.text, TextFields);
    let merged = base.clone().merge(st);
    let full = paint(&merged, theme);
    let sets_border = st_paint.border_width.iter().any(Option::is_some)
        || st_paint.border_color.iter().any(Option::is_some)
        || st_paint.border_style.iter().any(Option::is_some);
    let sets_effects = st_paint.filter.is_some()
        || st_paint.backdrop_filter.is_some()
        || st_paint.blend.is_some()
        || st_paint.mask.is_some()
        || st_paint.clip_path.is_some();
    PaintDelta {
        background: st_paint.background.as_ref().and(full.background),
        border: if sets_border { full.border } else { None },
        ring: st_paint.ring.and(full.ring),
        radius: st_paint
            .radius
            .iter()
            .any(Option::is_some)
            .then_some(full.radius),
        opacity: st_paint.opacity,
        shadows: st_paint.shadows.as_ref().map(|_| full.shadows),
        transform: sets_transform(st).then_some(full.transform),
        perspective: (st_transform.perspective.is_some()
            || st_transform.perspective_origin.is_some())
        .then_some(full.perspective),
        transform_style: st_transform.transform_style.map(|_| full.transform_style),
        backface: st_transform.backface.map(|_| full.backface),
        z_index: st_flow.z_index,
        scrollbar: st_paint
            .scrollbar
            .as_ref()
            .zip(full.scrollbar.as_ref())
            .map(|(style, full)| {
                Box::new(p::ScrollbarDelta {
                    width: style.width.map(|_| full.width),
                    colors: style.colors.map(|_| (full.thumb, full.track)),
                })
            }),
        cursor: st_paint.cursor,
        effects: sets_effects.then_some(full.effects),
        text_color: st_text.color.map(|c| c.resolve(theme)),
    }
}

fn scrollbar(s: &Style, theme: &Theme) -> Option<Box<p::ScrollbarProps>> {
    let paint = group_or_default!(s.paint, PaintFields);
    let style = paint.scrollbar.as_ref()?;
    let mut props = p::ScrollbarProps {
        width: match style.width.unwrap_or_default() {
            ScrollbarWidth::Auto | ScrollbarWidth::Thin => 6.0,
            ScrollbarWidth::None => 0.0,
            ScrollbarWidth::Px(px) => {
                if px.is_finite() {
                    px.clamp(0.0, 64.0)
                } else {
                    6.0
                }
            }
        },
        ..Default::default()
    };
    if let Some((thumb, track)) = style.colors {
        props.thumb = thumb.resolve(theme);
        props.track = track.resolve(theme);
    }
    Some(Box::new(props))
}
