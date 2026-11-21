//! Role: A style's declarations folded into one record of authored values, which resolution reads.

use kivact_runtime::Transition;
use kivact_types::{
    Align, BackfaceVisibility, Cursor, FlexDirection, FontStyle, Isolation, Justify, Overflow,
    Position, ScrollAxes, StyleState, TextAlign, TransformStyle, TransitionProp, WillChange,
};

use super::decl::{Decl, PropGroup, Style};
use crate::color_value::Color;
use crate::draw::LocalPaint;
use crate::effect::{BlendMode, BorderStyle, CornerRadius, Filter, Shadow, Shape};
use crate::grid::GridStyle;
use crate::paint::{Background, BorderImage, Mask};
use crate::text::{LineHeight, TextRareFields};
use crate::transform::Transform;
use kivact_types::Length;

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub(crate) struct Sides {
    pub(crate) top: Option<Length>,
    pub(crate) right: Option<Length>,
    pub(crate) bottom: Option<Length>,
    pub(crate) left: Option<Length>,
}

/// The size properties of a folded style: the box's size and flex sizing.
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

/// The spacing properties of a folded style: padding, margin, insets and gaps.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct SpacingFields {
    pub(crate) padding: Sides,
    pub(crate) margin: Sides,
    pub(crate) inset: Sides,
    pub(crate) row_gap: Option<Length>,
    pub(crate) column_gap: Option<Length>,
}

/// The flow properties of a folded style: flex, grid, position and overflow.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct FlowFields {
    pub(crate) direction: Option<FlexDirection>,
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

/// The paint properties of a folded style: backgrounds, borders, effects
/// and the ways a box is drawn.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct PaintFields {
    pub(crate) local_paint: Option<LocalPaint>,
    pub(crate) background: Option<Background>,
    /// Border widths and colors, top, right, bottom, left.
    pub(crate) border_width: [Option<f64>; 4],
    pub(crate) border_color: [Option<Color>; 4],
    /// Border line styles, top, right, bottom, left.
    pub(crate) border_style: [Option<BorderStyle>; 4],
    pub(crate) ring_width: Option<f64>,
    pub(crate) ring_offset: Option<f64>,
    pub(crate) ring_color: Option<Color>,
    /// Corner radii, top left, top right, bottom right, bottom left.
    pub(crate) radius: [Option<CornerRadius>; 4],
    pub(crate) shadows: Option<Vec<Shadow>>,
    pub(crate) opacity: Option<f64>,
    pub(crate) isolation: Option<Isolation>,
    pub(crate) border_image: Option<Box<BorderImage>>,
    pub(crate) scrollbar_width: Option<ScrollbarWidth>,
    pub(crate) scrollbar_thumb: Option<Color>,
    pub(crate) scrollbar_track: Option<Color>,
    pub(crate) cursor: Option<Cursor>,
    pub(crate) clip: Option<bool>,
    pub(crate) filter: Option<Filter>,
    pub(crate) backdrop_filter: Option<Filter>,
    pub(crate) blend: Option<BlendMode>,
    pub(crate) mask: Option<Box<Mask>>,
    pub(crate) clip_path: Option<Box<Shape>>,
}

/// The transform properties of a folded style.
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

/// The text properties of a folded style: the text properties of a node
/// and its subtree.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct TextFields {
    pub(crate) font_size: Option<Length>,
    pub(crate) weight: Option<u16>,
    pub(crate) color: Option<Color>,
    pub(crate) line_height: Option<LineHeight>,
    pub(crate) text_wrap: Option<bool>,
    pub(crate) text_align: Option<TextAlign>,
    pub(crate) font_family: Option<String>,
    pub(crate) italic: Option<bool>,
    /// The rarer text properties (letter spacing, decorations, features,
    /// overflow, clamps, case, shadows, input colors: boxed).
    pub(crate) text: Option<Box<TextRareFields>>,
}

// xtask-allow(state): independent: each Option is the style of one interaction state and group marks a group container; any set of states may be styled, and merging combines each state's style on its own
/// The interaction styles of a folded style and the nesting of a group.
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
    /// Overlay the state `state` with `s`: the declarations of `s` come after
    /// the ones already folded in.
    fn merge_state(&mut self, state: StyleState, s: &Style) {
        let slot = match state {
            StyleState::Hover => &mut self.hover,
            StyleState::Active => &mut self.pressed,
            StyleState::Focus => &mut self.focus,
            StyleState::FocusVisible => &mut self.focus_visible,
            StyleState::GroupHover => &mut self.group_hover,
            StyleState::GroupFocus => &mut self.group_focus,
            StyleState::Disabled => &mut self.disabled,
            StyleState::Selected => &mut self.selected,
            StyleState::Enter => &mut self.enter,
            StyleState::Exit => &mut self.exit,
        };
        let merged = match slot.take() {
            Some(x) => (*x).merge(s),
            None => s.clone(),
        };
        *slot = Some(Box::new(merged));
    }
}

/// The motion properties of a folded style: the enter/exit sizes and the
/// transitions.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct MotionFields {
    pub(crate) enter_size: Option<kivact_types::Orientation>,
    pub(crate) exit_size: Option<kivact_types::Orientation>,
    pub(crate) layout_transition: Option<Transition>,
    pub(crate) transitions: Vec<(TransitionProp, Transition)>,
}

/// Overlay scrollbar width.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum ScrollbarWidth {
    /// The default (6 logical pixels).
    #[default]
    Auto,
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

/// A style's declarations folded into one record, the later winning: what
/// resolution reads.
#[derive(Clone, Debug, Default)]
pub(crate) struct Groups {
    pub(crate) size: SizeFields,
    pub(crate) spacing: SpacingFields,
    pub(crate) flow: FlowFields,
    pub(crate) paint: PaintFields,
    pub(crate) transform: TransformFields,
    pub(crate) text: TextFields,
    pub(crate) states: StateFields,
    pub(crate) motion: MotionFields,
}

impl Style {
    /// The declarations of this style folded into one record, the later winning.
    pub(crate) fn fold(&self) -> Groups {
        let mut g = Groups::default();
        for d in &self.decls {
            g.apply(d);
        }
        g
    }
}

impl Groups {
    /// Fold one declaration in: the group it belongs to takes its value.
    fn apply(&mut self, d: &Decl) {
        match d.group() {
            PropGroup::Size => self.apply_size(d),
            PropGroup::Spacing => self.apply_spacing(d),
            PropGroup::Flow => self.apply_flow(d),
            PropGroup::Paint => self.apply_paint(d),
            PropGroup::Transform => self.apply_transform(d),
            PropGroup::Text => apply_text(&mut self.text, d),
            PropGroup::States => self.apply_states(d),
            PropGroup::Motion => self.apply_motion(d),
        }
    }

    fn apply_size(&mut self, d: &Decl) {
        match d {
            Decl::Width(v) => self.size.width = Some(*v),
            Decl::Height(v) => self.size.height = Some(*v),
            Decl::MinWidth(v) => self.size.min_width = Some(*v),
            Decl::MinHeight(v) => self.size.min_height = Some(*v),
            Decl::MaxWidth(v) => self.size.max_width = Some(*v),
            Decl::MaxHeight(v) => self.size.max_height = Some(*v),
            Decl::FlexBasis(v) => self.size.basis = Some(*v),
            Decl::AspectRatio(v) => self.size.aspect_ratio = Some(*v),
            Decl::FlexGrow(v) => self.size.grow = Some(*v),
            Decl::FlexShrink(v) => self.size.shrink = Some(*v),
            _ => {}
        }
    }

    fn apply_spacing(&mut self, d: &Decl) {
        let sp = &mut self.spacing;
        match d {
            Decl::PaddingTop(v) => sp.padding.top = Some(*v),
            Decl::PaddingRight(v) => sp.padding.right = Some(*v),
            Decl::PaddingBottom(v) => sp.padding.bottom = Some(*v),
            Decl::PaddingLeft(v) => sp.padding.left = Some(*v),
            Decl::MarginTop(v) => sp.margin.top = Some(*v),
            Decl::MarginRight(v) => sp.margin.right = Some(*v),
            Decl::MarginBottom(v) => sp.margin.bottom = Some(*v),
            Decl::MarginLeft(v) => sp.margin.left = Some(*v),
            Decl::Top(v) => sp.inset.top = Some(*v),
            Decl::Right(v) => sp.inset.right = Some(*v),
            Decl::Bottom(v) => sp.inset.bottom = Some(*v),
            Decl::Left(v) => sp.inset.left = Some(*v),
            Decl::RowGap(v) => sp.row_gap = Some(*v),
            Decl::ColumnGap(v) => sp.column_gap = Some(*v),
            _ => {}
        }
    }

    fn apply_flow(&mut self, d: &Decl) {
        let flow = &mut self.flow;
        match d {
            Decl::FlexDirection(v) => flow.direction = Some(*v),
            Decl::AlignItems(v) => flow.align = Some(*v),
            Decl::AlignSelf(v) => flow.align_self = Some(*v),
            Decl::JustifyContent(v) => flow.justify = Some(*v),
            Decl::AlignContent(v) => flow.align_content = Some(*v),
            Decl::Display(v) => flow.display = Some(*v),
            Decl::Visibility(v) => flow.visibility = Some(*v),
            Decl::FlexWrap(v) => flow.wrap = Some(*v),
            Decl::Position(v) => flow.position = Some(*v),
            Decl::Overflow(v) => flow.overflow = Some(*v),
            Decl::ScrollAxes(v) => flow.scroll_axes = Some(*v),
            Decl::ScrollSnap(v) => flow.scroll_snap = Some(*v),
            Decl::ZIndex(v) => flow.z_index = Some(*v),
            _ => apply_grid(flow, d),
        }
    }

    fn apply_paint(&mut self, d: &Decl) {
        let p = &mut self.paint;
        match d {
            Decl::Background(v) => p.background = Some(v.clone()),
            Decl::BorderTopWidth(v) => p.border_width[0] = Some(*v),
            Decl::BorderRightWidth(v) => p.border_width[1] = Some(*v),
            Decl::BorderBottomWidth(v) => p.border_width[2] = Some(*v),
            Decl::BorderLeftWidth(v) => p.border_width[3] = Some(*v),
            Decl::BorderTopColor(v) => p.border_color[0] = Some(*v),
            Decl::BorderRightColor(v) => p.border_color[1] = Some(*v),
            Decl::BorderBottomColor(v) => p.border_color[2] = Some(*v),
            Decl::BorderLeftColor(v) => p.border_color[3] = Some(*v),
            Decl::BorderTopStyle(v) => p.border_style[0] = Some(*v),
            Decl::BorderRightStyle(v) => p.border_style[1] = Some(*v),
            Decl::BorderBottomStyle(v) => p.border_style[2] = Some(*v),
            Decl::BorderLeftStyle(v) => p.border_style[3] = Some(*v),
            Decl::BorderTopLeftRadius(v) => p.radius[0] = Some(**v),
            Decl::BorderTopRightRadius(v) => p.radius[1] = Some(**v),
            Decl::BorderBottomRightRadius(v) => p.radius[2] = Some(**v),
            Decl::BorderBottomLeftRadius(v) => p.radius[3] = Some(**v),
            _ => apply_paint_rest(p, d),
        }
    }

    fn apply_transform(&mut self, d: &Decl) {
        let t = &mut self.transform;
        match d {
            Decl::TranslateX(v) => t.translate[0] = Some(*v),
            Decl::TranslateY(v) => t.translate[1] = Some(*v),
            Decl::TranslateZ(v) => t.translate_z = Some(*v),
            Decl::Rotate(v) => t.rotate = Some(*v),
            Decl::RotateX(v) => t.rotate_x = Some(*v),
            Decl::RotateY(v) => t.rotate_y = Some(*v),
            Decl::Scale(v) => t.scale = Some(*v),
            Decl::Transform(v) => t.transform = Some(**v),
            Decl::TransformOrigin(v) => t.transform_origin = Some(**v),
            Decl::TransformOriginZ(v) => t.transform_origin_z = Some(*v),
            Decl::Perspective(v) => t.perspective = Some(*v),
            Decl::PerspectiveOrigin(v) => t.perspective_origin = Some(**v),
            Decl::TransformStyle(v) => t.transform_style = Some(*v),
            Decl::BackfaceVisibility(v) => t.backface = Some(*v),
            Decl::WillChange(v) => t.will_change = Some(*v),
            _ => {}
        }
    }

    fn apply_states(&mut self, d: &Decl) {
        match d {
            Decl::State(state, s) => self.states.merge_state(*state, s),
            Decl::Group => self.states.group = true,
            _ => {}
        }
    }

    fn apply_motion(&mut self, d: &Decl) {
        let m = &mut self.motion;
        match d {
            Decl::EnterSize(v) => m.enter_size = Some(*v),
            Decl::ExitSize(v) => m.exit_size = Some(*v),
            Decl::LayoutTransition(v) => m.layout_transition = Some((**v).clone()),
            Decl::Transition(prop, t) => {
                m.transitions.retain(|(p, _)| p != prop);
                m.transitions.push((*prop, (**t).clone()));
            }
            _ => {}
        }
    }
}

/// The paint declarations past the border sides and corners: the rest of
/// the paint group (rings, effects, scrollbars, shadows, the callback).
fn apply_paint_rest(p: &mut PaintFields, d: &Decl) {
    match d {
        Decl::OutlineWidth(v) => p.ring_width = Some(*v),
        Decl::OutlineOffset(v) => p.ring_offset = Some(*v),
        Decl::OutlineColor(v) => p.ring_color = Some(*v),
        Decl::Opacity(v) => p.opacity = Some(*v),
        Decl::Isolation(v) => p.isolation = Some(*v),
        Decl::BorderImage(v) => p.border_image = Some(v.clone()),
        Decl::ScrollbarWidth(v) => p.scrollbar_width = Some(*v),
        Decl::ScrollbarThumbColor(v) => p.scrollbar_thumb = Some(*v),
        Decl::ScrollbarTrackColor(v) => p.scrollbar_track = Some(*v),
        Decl::Cursor(v) => p.cursor = Some(*v),
        Decl::Clip(v) => p.clip = Some(*v),
        Decl::Filter(v) => p.filter = Some((**v).clone()),
        Decl::BackdropFilter(v) => p.backdrop_filter = Some((**v).clone()),
        Decl::MixBlendMode(v) => p.blend = Some(*v),
        Decl::Mask(v) => p.mask = Some(v.clone()),
        Decl::ClipPath(v) => p.clip_path = Some(v.clone()),
        Decl::LocalPaint(v) => p.local_paint = Some(v.clone()),
        Decl::BoxShadow(list) => p.shadows = Some(list.clone()),
        _ => {}
    }
}

/// The grid declarations, which live in the boxed `GridStyle` of a flow record.
fn apply_grid(flow: &mut FlowFields, d: &Decl) {
    if !matches!(
        d,
        Decl::GridTemplateColumns(_)
            | Decl::GridTemplateRows(_)
            | Decl::GridAutoColumns(_)
            | Decl::GridAutoRows(_)
            | Decl::GridAutoFlow(_)
            | Decl::GridColumn(_)
            | Decl::GridRow(_)
            | Decl::JustifyItems(_)
            | Decl::JustifySelf(_)
    ) {
        return;
    }
    let grid = flow.grid.get_or_insert_with(Default::default);
    match d {
        Decl::GridTemplateColumns(v) => grid.columns = Some(v.clone()),
        Decl::GridTemplateRows(v) => grid.rows = Some(v.clone()),
        Decl::GridAutoColumns(v) => grid.auto_columns = Some(v.clone()),
        Decl::GridAutoRows(v) => grid.auto_rows = Some(v.clone()),
        Decl::GridAutoFlow(v) => grid.flow = Some(*v),
        Decl::GridColumn(v) => grid.column = Some(**v),
        Decl::GridRow(v) => grid.row = Some(**v),
        Decl::JustifyItems(v) => grid.justify_items = Some(*v),
        Decl::JustifySelf(v) => grid.justify_self = Some(*v),
        _ => {}
    }
}

/// The text declarations: the common text properties, then the rare ones
/// (and the text shadows) in the boxed `TextRareFields`.
fn apply_text(t: &mut TextFields, d: &Decl) {
    match d {
        Decl::FontSize(v) => t.font_size = Some(*v),
        Decl::FontWeight(v) => t.weight = Some(*v),
        Decl::Color(v) => t.color = Some(*v),
        Decl::LineHeight(v) => t.line_height = Some(*v),
        Decl::TextWrap(v) => t.text_wrap = Some(*v),
        Decl::TextAlign(v) => t.text_align = Some(*v),
        Decl::FontFamily(v) => t.font_family = Some(v.clone()),
        Decl::FontStyle(v) => t.italic = Some(*v == FontStyle::Italic),
        _ => apply_text_extras(t.text.get_or_insert_with(Default::default), d),
    }
}

/// The rare text properties, which the fold keeps in one boxed record.
fn apply_text_extras(x: &mut TextRareFields, d: &Decl) {
    match d {
        Decl::LetterSpacing(v) => x.letter_spacing = Some(*v),
        Decl::WordSpacing(v) => x.word_spacing = Some(*v),
        Decl::FontKerning(v) => x.kerning = Some(*v),
        Decl::FontFeatureSettings(v) => x.features = Some(v.clone()),
        Decl::TabularNums(v) => x.tabular_nums = Some(*v),
        Decl::TextDecorationLine(v) => x.decoration_line = Some(*v),
        Decl::TextDecorationStyle(v) => x.decoration_style = Some(*v),
        Decl::TextDecorationColor(v) => x.decoration_color = Some(*v),
        Decl::TextDecorationThickness(v) => x.decoration_thickness = Some(*v),
        Decl::TextUnderlineOffset(v) => x.underline_offset = Some(*v),
        Decl::TextOverflow(v) => x.overflow = Some(*v),
        Decl::LineClamp(v) => x.line_clamp = Some(*v),
        Decl::TextCapsize(v) => x.capsize = Some(*v),
        Decl::TextTransform(v) => x.transform = Some(*v),
        Decl::TextStrokeWidth(v) => x.stroke_width = Some(*v),
        Decl::TextStrokeColor(v) => x.stroke_color = Some(*v),
        Decl::PaintOrder(v) => x.paint_order = Some(*v),
        Decl::FontVariationSettings(v) => x.variations = Some(v.clone()),
        Decl::FontOpticalSizing(v) => x.optical_sizing = Some(*v),
        Decl::FontWidth(v) => x.font_width = Some(*v),
        Decl::CaretColor(v) => x.caret_color = Some(*v),
        Decl::SelectionColor(v) => x.selection_color = Some(*v),
        Decl::PlaceholderColor(v) => x.placeholder_color = Some(*v),
        Decl::TextShadow(list) => x.shadows = Some(list.clone()),
        _ => {}
    }
}
