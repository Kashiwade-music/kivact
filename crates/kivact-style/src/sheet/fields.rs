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

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub(crate) struct Sides {
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) top: Option<Length>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) right: Option<Length>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) bottom: Option<Length>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
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

/// The `size` group of a [`Style`]: the box's size and flex sizing.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct SizeFields {
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) width: Option<Length>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) height: Option<Length>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) min_width: Option<Length>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) min_height: Option<Length>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) max_width: Option<Length>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) max_height: Option<Length>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) basis: Option<Length>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) aspect_ratio: Option<f64>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) grow: Option<f64>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
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

/// The `spacing` group of a [`Style`]: padding, margin, insets and gaps.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct SpacingFields {
    pub(crate) padding: Sides,
    pub(crate) margin: Sides,
    pub(crate) inset: Sides,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) row_gap: Option<Length>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
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

/// The `flow` group of a [`Style`]: flex, grid, position and overflow.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct FlowFields {
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) direction: Option<Direction>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) align: Option<Align>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) align_self: Option<Align>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) justify: Option<Justify>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) align_content: Option<Justify>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    /// The display mode: flex (default), grid or a transparent container (`display(..)`, `grid(..)`).
    pub(crate) display: Option<kivact_types::Display>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) visibility: Option<kivact_types::Visibility>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    /// Grid tracks and placement (rare: boxed).
    pub(crate) grid: Option<Box<GridStyle>>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) wrap: Option<bool>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) position: Option<Position>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) overflow: Option<Overflow>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    /// Scroll axes set by `scroll(..)`.
    pub(crate) scroll_axes: Option<ScrollAxes>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) scroll_snap: Option<kivact_types::ScrollSnap>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
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

/// The `paint` group of a [`Style`]: backgrounds, borders, effects and
/// the ways a box is drawn.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct PaintFields {
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) local_paint: Option<LocalPaint>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) background: Option<Background>,
    /// Border widths and colors, top, right, bottom, left.
    pub(crate) border_width: [Option<f64>; 4],
    pub(crate) border_color: [Option<Color>; 4],
    /// Border line styles, top, right, bottom, left.
    pub(crate) border_style: [Option<BorderStyle>; 4],
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) ring: Option<(f64, f64, Color)>,
    /// Corner radii, top left, top right, bottom right, bottom left.
    pub(crate) radius: [Option<CornerRadius>; 4],
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) shadows: Option<Vec<Shadow>>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) opacity: Option<f64>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) isolation: Option<Isolation>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) border_image: Option<Box<BorderImage>>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) scrollbar: Option<Box<ScrollbarStyle>>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) cursor: Option<Cursor>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) clip: Option<bool>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) filter: Option<Filter>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) backdrop_filter: Option<Filter>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) blend: Option<BlendMode>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) mask: Option<Box<Mask>>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
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

/// The `transform` group of a [`Style`]: the transform properties.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct TransformFields {
    /// CSS `translate` (x, then y: each axis is set and merged on its own),
    /// `rotate`, `scale`, `transform`, `transform-origin`.
    pub(crate) translate: [Option<Length>; 2],
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) rotate: Option<f64>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) scale: Option<(f64, f64)>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) transform: Option<Transform>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) transform_origin: Option<(Length, Length)>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) translate_z: Option<f64>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) rotate_x: Option<f64>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) rotate_y: Option<f64>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) transform_origin_z: Option<f64>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) perspective: Option<f64>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) perspective_origin: Option<(Length, Length)>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) transform_style: Option<TransformStyle>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) backface: Option<BackfaceVisibility>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
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

/// The `text` group of a [`Style`]: the text properties of a node and its
/// subtree.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct TextFields {
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) font_size: Option<f64>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) weight: Option<u16>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) color: Option<Color>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) line_height: Option<LineHeight>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) text_wrap: Option<bool>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) text_align: Option<TextAlign>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) font_family: Option<String>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) italic: Option<bool>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
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

/// The `states` group of a [`Style`]: the interaction styles and the
/// nesting of a group.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct StateFields {
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) hover: Option<Box<Style>>,
    // xtask-allow(state-fields): option: one authored flag (style().group()); false means the node is not a group, and group_hover/group_focus read the nearest ancestor group, not this flag
    pub(crate) group: bool,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) group_hover: Option<Box<Style>>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) group_focus: Option<Box<Style>>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) pressed: Option<Box<Style>>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) focus: Option<Box<Style>>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) focus_visible: Option<Box<Style>>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) disabled: Option<Box<Style>>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) selected: Option<Box<Style>>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) enter: Option<Box<Style>>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
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

/// The `motion` group of a [`Style`]: the enter/exit sizes and the
/// transitions.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct MotionFields {
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) enter_size: Option<kivact_types::Orientation>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) exit_size: Option<kivact_types::Orientation>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
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
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) width: Option<ScrollbarWidth>,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) colors: Option<(Color, Color)>,
}
/// A style value. Build with [`style()`](crate::style).
///
/// An authored style: each group of properties is allocated only when one
/// of its properties is set. Values and merge rules are the dense style's.
#[derive(Clone, Debug, Default)]
pub struct Style {
    // xtask-allow(state-fields): option: one group of authored properties; absent means the group's defaults
    pub(crate) size: Option<Box<SizeFields>>,
    // xtask-allow(state-fields): option: one group of authored properties; absent means the group's defaults
    pub(crate) spacing: Option<Box<SpacingFields>>,
    // xtask-allow(state-fields): option: one group of authored properties; absent means the group's defaults
    pub(crate) flow: Option<Box<FlowFields>>,
    // xtask-allow(state-fields): option: one group of authored properties; absent means the group's defaults
    pub(crate) paint: Option<Box<PaintFields>>,
    // xtask-allow(state-fields): option: one group of authored properties; absent means the group's defaults
    pub(crate) transform: Option<Box<TransformFields>>,
    // xtask-allow(state-fields): option: one group of authored properties; absent means the group's defaults
    pub(crate) text: Option<Box<TextFields>>,
    // xtask-allow(state-fields): option: one group of authored properties; absent means the group's defaults
    pub(crate) states: Option<Box<StateFields>>,
    // xtask-allow(state-fields): option: one group of authored properties; absent means the group's defaults
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
