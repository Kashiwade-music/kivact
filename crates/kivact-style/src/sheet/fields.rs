//! Role: The `Style` property groups: their fields and merge rules.

use kivact_runtime::Transition;
use kivact_types::{
    Align, BackfaceVisibility, Cursor, FlexDirection as Direction, Isolation, Justify, Overflow,
    Position, ScrollAxes, TextAlign, TransformStyle, TransitionProp, WillChange,
};

use crate::color::Color;
use crate::draw::LocalPaint;
use crate::effect::{BlendMode, BorderStyle, CornerRadius, Filter, Shadow, Shape};
use crate::grid::GridStyle;
use crate::length::Length;
use crate::paint::{Background, BorderImage, Mask};
use crate::text::{LineHeight, TextExtras};
use crate::transform::Transform;

// xtask-allow(state): legacy: top, right, bottom, left; one authored property; absent means the inherited or default value
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub(crate) struct Sides {
    pub(crate) top: Option<Length>,
    pub(crate) right: Option<Length>,
    pub(crate) bottom: Option<Length>,
    pub(crate) left: Option<Length>,
}

impl Sides {
    fn merge(&mut self, o: &Sides) {
        self.top = o.top.or(self.top);
        self.right = o.right.or(self.right);
        self.bottom = o.bottom.or(self.bottom);
        self.left = o.left.or(self.left);
    }
}

// xtask-allow(state): legacy: width, height, min_width, min_height, max_width, max_height, basis, aspect_ratio, grow, shrink; one authored property; absent means the inherited or
/// The `size` group of a [`Style`]: the box's size and flex sizing.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct SizeFields {
    pub(crate) width: Option<Length>,
    pub(crate) height: Option<Length>,
    pub(crate) min_width: Option<Length>,
    pub(crate) min_height: Option<Length>,
    pub(crate) max_width: Option<Length>,
    pub(crate) max_height: Option<Length>,
    pub(crate) basis: Option<Length>,
    pub(crate) aspect_ratio: Option<f64>,
    pub(crate) grow: Option<f64>,
    pub(crate) shrink: Option<f64>,
}

impl SizeFields {
    pub(super) fn merge(&mut self, o: &Self) {
        macro_rules! take {
            ($($f:ident),*) => {$(
                if o.$f.is_some() {
                    self.$f = o.$f;
                }
            )*};
        }
        take!(
            width,
            height,
            min_width,
            min_height,
            max_width,
            max_height,
            basis,
            aspect_ratio,
            grow,
            shrink
        );
    }
}

// xtask-allow(state): legacy: row_gap, column_gap; one authored property; absent means the inherited or default value
/// The `spacing` group of a [`Style`]: padding, margin, insets and gaps.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct SpacingFields {
    pub(crate) padding: Sides,
    pub(crate) margin: Sides,
    pub(crate) inset: Sides,
    pub(crate) row_gap: Option<Length>,
    pub(crate) column_gap: Option<Length>,
}

impl SpacingFields {
    pub(super) fn merge(&mut self, o: &Self) {
        self.padding.merge(&o.padding);
        self.margin.merge(&o.margin);
        self.inset.merge(&o.inset);
        if o.row_gap.is_some() {
            self.row_gap = o.row_gap;
        }
        if o.column_gap.is_some() {
            self.column_gap = o.column_gap;
        }
    }
}

// xtask-allow(state): legacy: direction, align, align_self, justify, align_content, display, visibility, grid, wrap, position, overflow, scroll_axes, scroll_snap, z_index; one
/// The `flow` group of a [`Style`]: flex, grid, position and overflow.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct FlowFields {
    pub(crate) direction: Option<Direction>,
    pub(crate) align: Option<Align>,
    pub(crate) align_self: Option<Align>,
    pub(crate) justify: Option<Justify>,
    pub(crate) align_content: Option<Justify>,
    /// The display mode: flex (default), grid or a transparent container (`display(..)`, `grid(..)`).
    pub(crate) display: Option<kivact_types::Display>,
    pub(crate) visibility: Option<kivact_types::Visibility>,
    /// Grid tracks and placement (rare: boxed).
    pub(crate) grid: Option<Box<GridStyle>>,
    pub(crate) wrap: Option<bool>,
    pub(crate) position: Option<Position>,
    pub(crate) overflow: Option<Overflow>,
    /// Scroll axes set by `scroll(..)`.
    pub(crate) scroll_axes: Option<ScrollAxes>,
    pub(crate) scroll_snap: Option<kivact_types::ScrollSnap>,
    pub(crate) z_index: Option<i32>,
}

impl FlowFields {
    pub(super) fn merge(&mut self, o: &Self) {
        macro_rules! take {
            ($($f:ident),*) => {$(
                if o.$f.is_some() {
                    self.$f = o.$f.clone();
                }
            )*};
        }
        take!(
            direction,
            align,
            align_self,
            justify,
            align_content,
            display,
            visibility,
            wrap,
            position,
            overflow,
            scroll_axes,
            scroll_snap,
            z_index
        );
        if let Some(g) = &o.grid {
            self.grid.get_or_insert_with(Default::default).merge(g);
        }
    }
}

// xtask-allow(state): legacy: local_paint, background, ring, shadows, opacity, isolation, border_image, scrollbar, cursor, clip, filter, backdrop_filter, blend, mask, clip_path; o
/// The `paint` group of a [`Style`]: backgrounds, borders, effects and
/// the ways a box is drawn.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct PaintFields {
    pub(crate) local_paint: Option<LocalPaint>,
    pub(crate) background: Option<Background>,
    /// Border widths and colors, top, right, bottom, left.
    pub(crate) border_width: [Option<f64>; 4],
    pub(crate) border_color: [Option<Color>; 4],
    /// Border line styles, top, right, bottom, left.
    pub(crate) border_style: [Option<BorderStyle>; 4],
    pub(crate) ring: Option<(f64, f64, Color)>,
    /// Corner radii, top left, top right, bottom right, bottom left.
    pub(crate) radius: [Option<CornerRadius>; 4],
    pub(crate) shadows: Option<Vec<Shadow>>,
    pub(crate) opacity: Option<f64>,
    pub(crate) isolation: Option<Isolation>,
    pub(crate) border_image: Option<Box<BorderImage>>,
    pub(crate) scrollbar: Option<Box<ScrollbarStyle>>,
    pub(crate) cursor: Option<Cursor>,
    pub(crate) clip: Option<bool>,
    pub(crate) filter: Option<Filter>,
    pub(crate) backdrop_filter: Option<Filter>,
    pub(crate) blend: Option<BlendMode>,
    pub(crate) mask: Option<Box<Mask>>,
    pub(crate) clip_path: Option<Box<Shape>>,
}

impl PaintFields {
    pub(super) fn merge(&mut self, o: &Self) {
        macro_rules! take {
            ($($f:ident),*) => {$(
                if o.$f.is_some() {
                    self.$f = o.$f.clone();
                }
            )*};
        }
        take!(
            local_paint,
            background,
            ring,
            shadows,
            opacity,
            isolation,
            border_image,
            cursor,
            clip,
            filter,
            backdrop_filter,
            blend,
            mask,
            clip_path
        );
        if let Some(other) = &o.scrollbar {
            let mine = self.scrollbar.get_or_insert_with(Default::default);
            mine.width = other.width.or(mine.width);
            mine.colors = other.colors.or(mine.colors);
        }
        for (a, b) in self.border_width.iter_mut().zip(o.border_width) {
            *a = b.or(*a);
        }
        for (a, b) in self.border_style.iter_mut().zip(o.border_style) {
            *a = b.or(*a);
        }
        for (a, b) in self.border_color.iter_mut().zip(o.border_color) {
            *a = b.or(*a);
        }
        for (a, b) in self.radius.iter_mut().zip(o.radius) {
            *a = b.or(*a);
        }
    }
}

// xtask-allow(state): legacy: rotate, scale, transform, transform_origin, translate_z, rotate_x, rotate_y, transform_origin_z, perspective, perspective_origin, transform_style, backface, will_change
/// The `transform` group of a [`Style`]: the transform properties.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct TransformFields {
    /// CSS `translate` (x, then y: each axis is set and merged on its own),
    /// `rotate`, `scale`, `transform`, `transform-origin`.
    pub(crate) translate: [Option<Length>; 2],
    pub(crate) rotate: Option<f64>,
    pub(crate) scale: Option<(f64, f64)>,
    pub(crate) transform: Option<Transform>,
    pub(crate) transform_origin: Option<(Length, Length)>,
    pub(crate) translate_z: Option<f64>,
    pub(crate) rotate_x: Option<f64>,
    pub(crate) rotate_y: Option<f64>,
    pub(crate) transform_origin_z: Option<f64>,
    pub(crate) perspective: Option<f64>,
    pub(crate) perspective_origin: Option<(Length, Length)>,
    pub(crate) transform_style: Option<TransformStyle>,
    pub(crate) backface: Option<BackfaceVisibility>,
    pub(crate) will_change: Option<WillChange>,
}

impl TransformFields {
    pub(super) fn merge(&mut self, o: &Self) {
        macro_rules! take {
            ($($f:ident),*) => {$(
                if o.$f.is_some() {
                    self.$f = o.$f.clone();
                }
            )*};
        }
        take!(
            rotate,
            scale,
            transform,
            transform_origin,
            translate_z,
            rotate_x,
            rotate_y,
            transform_origin_z,
            perspective,
            perspective_origin,
            transform_style,
            backface,
            will_change
        );
        for (a, b) in self.translate.iter_mut().zip(o.translate) {
            *a = b.or(*a);
        }
    }
}

// xtask-allow(state): legacy: font_size, weight, color, line_height, text_wrap, text_align, font_family, italic, text; one authored property; absent means the inherited or
/// The `text` group of a [`Style`]: the text properties of a node and its
/// subtree.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct TextFields {
    pub(crate) font_size: Option<f64>,
    pub(crate) weight: Option<u16>,
    pub(crate) color: Option<Color>,
    pub(crate) line_height: Option<LineHeight>,
    pub(crate) text_wrap: Option<bool>,
    pub(crate) text_align: Option<TextAlign>,
    pub(crate) font_family: Option<String>,
    pub(crate) italic: Option<bool>,
    /// The rarer text properties (letter spacing, decorations, features,
    /// overflow, clamps, case, shadows, input colors: boxed).
    pub(crate) text: Option<Box<TextExtras>>,
}

impl TextFields {
    pub(super) fn merge(&mut self, o: &Self) {
        macro_rules! take {
            ($($f:ident),*) => {$(
                if o.$f.is_some() {
                    self.$f = o.$f.clone();
                }
            )*};
        }
        take!(
            font_size,
            weight,
            color,
            line_height,
            text_wrap,
            text_align,
            font_family,
            italic
        );
        if let Some(t) = &o.text {
            self.text.get_or_insert_with(Default::default).merge(t);
        }
    }
}

// xtask-allow(state): legacy: hover, group, group_hover, group_focus, pressed, focus, focus_visible, disabled, selected, enter, exit; one authored property; absent means the
/// The `states` group of a [`Style`]: the interaction styles and the
/// nesting of a group.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct StateFields {
    pub(crate) hover: Option<Box<Style>>,
    pub(crate) group: bool,
    pub(crate) group_hover: Option<Box<Style>>,
    pub(crate) group_focus: Option<Box<Style>>,
    pub(crate) pressed: Option<Box<Style>>,
    pub(crate) focus: Option<Box<Style>>,
    pub(crate) focus_visible: Option<Box<Style>>,
    pub(crate) disabled: Option<Box<Style>>,
    pub(crate) selected: Option<Box<Style>>,
    pub(crate) enter: Option<Box<Style>>,
    pub(crate) exit: Option<Box<Style>>,
}

impl StateFields {
    pub(super) fn merge(&mut self, o: &Self) {
        self.group |= o.group;
        for (a, b) in [
            (&mut self.hover, &o.hover),
            (&mut self.group_hover, &o.group_hover),
            (&mut self.group_focus, &o.group_focus),
            (&mut self.pressed, &o.pressed),
            (&mut self.focus, &o.focus),
            (&mut self.focus_visible, &o.focus_visible),
            (&mut self.disabled, &o.disabled),
            (&mut self.selected, &o.selected),
            (&mut self.enter, &o.enter),
            (&mut self.exit, &o.exit),
        ] {
            if let Some(b) = b {
                *a = Some(Box::new(match a.take() {
                    Some(x) => (*x).merge(b),
                    None => (**b).clone(),
                }));
            }
        }
    }
}

// xtask-allow(state): legacy: enter_size, exit_size, layout_transition; one authored property; absent means the inherited or default value
/// The `motion` group of a [`Style`]: the enter/exit sizes and the
/// transitions.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct MotionFields {
    pub(crate) enter_size: Option<kivact_types::Orientation>,
    pub(crate) exit_size: Option<kivact_types::Orientation>,
    pub(crate) layout_transition: Option<Transition>,
    pub(crate) transitions: Vec<(TransitionProp, Transition)>,
}

impl MotionFields {
    pub(super) fn merge(&mut self, o: &Self) {
        if o.enter_size.is_some() {
            self.enter_size = o.enter_size;
        }
        if o.exit_size.is_some() {
            self.exit_size = o.exit_size;
        }
        if o.layout_transition.is_some() {
            self.layout_transition = o.layout_transition.clone();
        }
        for t in &o.transitions {
            self.transitions.retain(|(p, _)| *p != t.0);
            self.transitions.push(t.clone());
        }
    }
}
/// Overlay scrollbar width.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum ScrollbarWidth {
    /// Thin default (6 logical pixels).
    #[default]
    Auto,
    /// Thin (6 logical pixels).
    Thin,
    /// Hidden; scrolling still works.
    None,
    /// Explicit logical pixel width.
    Px(f64),
}
impl From<f64> for ScrollbarWidth {
    fn from(px: f64) -> Self {
        Self::Px(px)
    }
}
impl From<i32> for ScrollbarWidth {
    fn from(px: i32) -> Self {
        Self::Px(px as f64)
    }
}
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct ScrollbarStyle {
    pub(crate) width: Option<ScrollbarWidth>,
    pub(crate) colors: Option<(Color, Color)>,
}
// xtask-allow(state): legacy: size, spacing, flow, paint, transform, text, states, motion; one group of authored properties; absent means the group's defaults
/// A style value. Build with [`style()`](crate::style).
///
/// An authored style: each group of properties is allocated only when one
/// of its properties is set. Values and merge rules are the dense style's.
#[derive(Clone, Debug, Default)]
pub struct Style {
    pub(crate) size: Option<Box<SizeFields>>,
    pub(crate) spacing: Option<Box<SpacingFields>>,
    pub(crate) flow: Option<Box<FlowFields>>,
    pub(crate) paint: Option<Box<PaintFields>>,
    pub(crate) transform: Option<Box<TransformFields>>,
    pub(crate) text: Option<Box<TextFields>>,
    pub(crate) states: Option<Box<StateFields>>,
    pub(crate) motion: Option<Box<MotionFields>>,
    /// Setter calls that replaced an earlier value (the diagnostics count).
    pub(crate) duplicates: u32,
}

const _: () = assert!(
    std::mem::size_of::<Style>() <= 72,
    "Style keeps its groups boxed"
);

/// Whether two optional groups are equal, an unset group counting as its defaults.
fn group_eq<T: Default + PartialEq>(a: Option<&T>, b: Option<&T>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => a == b,
        (None, None) => true,
        (Some(set), None) | (None, Some(set)) => *set == T::default(),
    }
}

impl PartialEq for Style {
    fn eq(&self, other: &Self) -> bool {
        group_eq(self.size.as_deref(), other.size.as_deref())
            && group_eq(self.spacing.as_deref(), other.spacing.as_deref())
            && group_eq(self.flow.as_deref(), other.flow.as_deref())
            && group_eq(self.paint.as_deref(), other.paint.as_deref())
            && group_eq(self.transform.as_deref(), other.transform.as_deref())
            && group_eq(self.text.as_deref(), other.text.as_deref())
            && group_eq(self.states.as_deref(), other.states.as_deref())
            && group_eq(self.motion.as_deref(), other.motion.as_deref())
            && self.duplicates == other.duplicates
    }
}
impl Style {
    pub(crate) fn size_mut(&mut self) -> &mut SizeFields {
        self.size.get_or_insert_with(Default::default)
    }
    pub(crate) fn spacing_mut(&mut self) -> &mut SpacingFields {
        self.spacing.get_or_insert_with(Default::default)
    }
    pub(crate) fn flow_mut(&mut self) -> &mut FlowFields {
        self.flow.get_or_insert_with(Default::default)
    }
    pub(crate) fn paint_mut(&mut self) -> &mut PaintFields {
        self.paint.get_or_insert_with(Default::default)
    }
    pub(crate) fn transform_mut(&mut self) -> &mut TransformFields {
        self.transform.get_or_insert_with(Default::default)
    }
    pub(crate) fn text_mut(&mut self) -> &mut TextFields {
        self.text.get_or_insert_with(Default::default)
    }
    pub(crate) fn states_mut(&mut self) -> &mut StateFields {
        self.states.get_or_insert_with(Default::default)
    }
    pub(crate) fn motion_mut(&mut self) -> &mut MotionFields {
        self.motion.get_or_insert_with(Default::default)
    }
}
kivact_runtime::constant_value!(Style);

#[cfg(test)]
mod eq_tests {
    //! Role: Tests of Style equality: an unset group equals its defaults.
    use super::*;

    #[test]
    fn an_unset_group_equals_its_defaults() {
        let mut set = Style::default();
        set.size_mut();
        assert_eq!(set, Style::default());
        assert_eq!(Style::default(), set);
    }
}
