//! Role: Style setters outside the property table: shorthands, states, entries.
//!
//! The setters that are not one table row: shorthands over several properties,
//! state styles and the entry points.

use kivact_types::{Align, Length, Overflow, StyleState};

use super::props::{Decl, Style};
use crate::color_value::Color;
use crate::draw::{Draw, LocalPaint, Size};
use crate::effect::{Filter, Shadow};
use crate::text::{FontFeature, FontVariation, TextOverflow};
use kivact_runtime::Transition;

/// Start a style chain.
pub fn style() -> Style {
    Style::default()
}

/// Make this a scroll container along `axes`: content overflowing a
/// scrolling axis scrolls, the other axis clips (the base style of the
/// `scroll(..)` container).
pub fn with_scroll_axes(style: Style, axes: kivact_types::ScrollAxes) -> Style {
    style.push(Decl::ScrollAxes(axes))
}

impl Style {
    /// CSS `padding`: all four sides (`padding(8)`, `padding(pct(5))`).
    pub fn padding(self, v: impl Into<Length>) -> Self {
        let v: Length = v.into();
        self.padding_top(v)
            .padding_right(v)
            .padding_bottom(v)
            .padding_left(v)
    }

    /// CSS `padding` on the left and right sides (`padding_x`).
    pub fn padding_x(self, v: impl Into<Length>) -> Self {
        let v: Length = v.into();
        self.padding_left(v).padding_right(v)
    }

    /// CSS `padding` on the top and bottom sides (`padding_y`).
    pub fn padding_y(self, v: impl Into<Length>) -> Self {
        let v: Length = v.into();
        self.padding_top(v).padding_bottom(v)
    }

    /// CSS `margin`: all four sides.
    pub fn margin(self, v: impl Into<Length>) -> Self {
        let v: Length = v.into();
        self.margin_top(v)
            .margin_right(v)
            .margin_bottom(v)
            .margin_left(v)
    }

    /// CSS `margin` on the left and right sides.
    pub fn margin_x(self, v: impl Into<Length>) -> Self {
        let v: Length = v.into();
        self.margin_left(v).margin_right(v)
    }

    /// CSS `margin` on the top and bottom sides.
    pub fn margin_y(self, v: impl Into<Length>) -> Self {
        let v: Length = v.into();
        self.margin_top(v).margin_bottom(v)
    }

    /// The offsets of a positioned box on all four sides (CSS `inset`).
    pub fn inset(self, v: impl Into<Length>) -> Self {
        let v: Length = v.into();
        self.top(v).right(v).bottom(v).left(v)
    }

    /// CSS `gap`: between rows and between columns (`row_gap` and
    /// `column_gap` at once; CSS `gap: 6px 16px` is
    /// `.row_gap(6).column_gap(16)`).
    pub fn gap(self, v: impl Into<Length>) -> Self {
        let v: Length = v.into();
        self.row_gap(v).column_gap(v)
    }

    /// A label that stays on one line and ends in "…" when its box is too
    /// narrow. In CSS: `white-space: nowrap; overflow: hidden;
    /// text-overflow: ellipsis; min-width: 0` (so that a text in a flex row
    /// can shrink below its content width). It is
    /// `text_wrap(false)`, `text_overflow(TextOverflow::Ellipsis)`,
    /// `min_width(0)` and `overflow(Overflow::Hidden)`, which clips both
    /// axes of the text's own box. The accessible name stays the whole text.
    ///
    /// A parent that is a flex row also needs `min_width(0)` for the text to
    /// shrink.
    pub fn single_line(self) -> Self {
        self.text_wrap(false)
            .text_overflow(TextOverflow::Ellipsis)
            .min_width(0)
            .overflow(Overflow::Hidden)
    }

    /// Both width and height.
    pub fn size(self, w: impl Into<Length>, h: impl Into<Length>) -> Self {
        self.width(w).height(h)
    }

    /// CSS `translate: x y`: a move in px or a percentage of the node's
    /// own box (`translate(pct(-50), pct(-50))` centres it on its
    /// position), applied first of the transform properties. It sets both
    /// axes, like [`translate_x`](Self::translate_x) and
    /// [`translate_y`](Self::translate_y) together.
    pub fn translate(self, x: impl Into<Length>, y: impl Into<Length>) -> Self {
        self.translate_x(x).translate_y(y)
    }

    /// CSS `scale` (uniform), applied after `translate` and `rotate`
    /// and before `transform`. A state style can set it alone:
    /// `.hover(style().scale(1.05))` keeps the node's rotation.
    pub fn scale(self, v: impl Into<f64>) -> Self {
        let v: f64 = v.into();
        self.push(Decl::Scale((v, v)))
    }

    /// CSS `scale: x y` (non-uniform).
    pub fn scale_xy(self, x: impl Into<f64>, y: impl Into<f64>) -> Self {
        self.push(Decl::Scale((x.into(), y.into())))
    }

    /// CSS `transform-origin: x y`, the point the transform properties
    /// apply about: px or a percentage of the box (`left top` is
    /// `(0, 0)`, `right bottom` is `(pct(100), pct(100))`; the default is
    /// the centre, `(pct(50), pct(50))`).
    pub fn transform_origin(self, x: impl Into<Length>, y: impl Into<Length>) -> Self {
        self.push(Decl::TransformOrigin(Box::new((x.into(), y.into()))))
    }

    /// CSS `perspective-origin`: where the viewer looks at, against the box.
    pub fn perspective_origin(self, x: impl Into<Length>, y: impl Into<Length>) -> Self {
        self.push(Decl::PerspectiveOrigin(Box::new((x.into(), y.into()))))
    }

    /// A border on every side (CSS `border: w solid c`). Like padding, it
    /// takes room inside the box (CSS `box-sizing: border-box`): the
    /// children and the text sit inside the border and the padding, and a
    /// width or height is the border box's. A change of its color or style
    /// alone does not lay out again. In a state style (`hover`, `focus`,
    /// ...) a width lays out again while the state applies, as CSS does.
    pub fn border(self, width: impl Into<f64>, color: impl Into<Color>) -> Self {
        let (w, c) = (width.into(), color.into());
        self.border_top(w, c)
            .border_right(w, c)
            .border_bottom(w, c)
            .border_left(w, c)
    }

    /// The top border (CSS `border-top: w solid c`).
    pub fn border_top(self, width: impl Into<f64>, color: impl Into<Color>) -> Self {
        self.push(Decl::BorderTopWidth(width.into()))
            .push(Decl::BorderTopColor(color.into()))
    }

    /// The right border (CSS `border-right`).
    pub fn border_right(self, width: impl Into<f64>, color: impl Into<Color>) -> Self {
        self.push(Decl::BorderRightWidth(width.into()))
            .push(Decl::BorderRightColor(color.into()))
    }

    /// The bottom border (CSS `border-bottom`).
    pub fn border_bottom(self, width: impl Into<f64>, color: impl Into<Color>) -> Self {
        self.push(Decl::BorderBottomWidth(width.into()))
            .push(Decl::BorderBottomColor(color.into()))
    }

    /// The left border (CSS `border-left`).
    pub fn border_left(self, width: impl Into<f64>, color: impl Into<Color>) -> Self {
        self.push(Decl::BorderLeftWidth(width.into()))
            .push(Decl::BorderLeftColor(color.into()))
    }

    /// Every side's border width (CSS `border-width`); a side without a
    /// color takes `kivact::style::color::BORDER`.
    pub fn border_width(self, width: impl Into<f64>) -> Self {
        let w: f64 = width.into();
        self.push(Decl::BorderTopWidth(w))
            .push(Decl::BorderRightWidth(w))
            .push(Decl::BorderBottomWidth(w))
            .push(Decl::BorderLeftWidth(w))
    }

    /// Every side's border color (CSS `border-color`), keeping the widths
    /// (handy in state styles: `.hover(style().border_color(..))`).
    pub fn border_color(self, color: impl Into<Color>) -> Self {
        let c: Color = color.into();
        self.push(Decl::BorderTopColor(c))
            .push(Decl::BorderRightColor(c))
            .push(Decl::BorderBottomColor(c))
            .push(Decl::BorderLeftColor(c))
    }

    /// How every side's border is drawn (CSS `border-style`).
    pub fn border_style(self, s: crate::effect::BorderStyle) -> Self {
        self.border_top_style(s)
            .border_right_style(s)
            .border_bottom_style(s)
            .border_left_style(s)
    }

    /// Round every corner (CSS `border-radius`): `8`, `pct(50)` (an
    /// ellipse on a box that is not square) or `(20, 10)` (elliptical).
    pub fn border_radius(self, r: impl Into<crate::geometry::CornerRadius>) -> Self {
        let r: crate::geometry::CornerRadius = r.into();
        self.border_top_left_radius(r)
            .border_top_right_radius(r)
            .border_bottom_right_radius(r)
            .border_bottom_left_radius(r)
    }

    /// Round each corner: top left, top right, bottom right, bottom left
    /// (CSS `border-radius: tl tr br bl`).
    pub fn border_radius_corners(
        self,
        top_left: impl Into<crate::geometry::CornerRadius>,
        top_right: impl Into<crate::geometry::CornerRadius>,
        bottom_right: impl Into<crate::geometry::CornerRadius>,
        bottom_left: impl Into<crate::geometry::CornerRadius>,
    ) -> Self {
        self.border_top_left_radius(top_left)
            .border_top_right_radius(top_right)
            .border_bottom_right_radius(bottom_right)
            .border_bottom_left_radius(bottom_left)
    }

    /// The outline: a line drawn outside the border box, in `width` logical
    /// pixels of `color` (CSS `outline`). Usually inside `focus_visible(..)`;
    /// a focusable element without a focus look of its own draws the theme's
    /// ([`Theme::focus_ring`](crate::Theme::focus_ring)). Its gap is
    /// [`outline_offset`](Self::outline_offset).
    pub fn outline(self, width: impl Into<f64>, color: impl Into<Color>) -> Self {
        self.outline_width(width).outline_color(color)
    }

    /// No outline (CSS `outline: none`): in a [`focus_visible`](Self::focus_visible)
    /// style it turns the theme's focus ring off.
    pub fn outline_none(self) -> Self {
        self.outline(0.0, Color::TRANSPARENT)
    }

    /// Set thumb and track colors (CSS scrollbar-color).
    pub fn scrollbar_color(self, thumb: impl Into<Color>, track: impl Into<Color>) -> Self {
        self.push(Decl::ScrollbarThumbColor(thumb.into()))
            .push(Decl::ScrollbarTrackColor(track.into()))
    }

    /// Add a box shadow (CSS `box-shadow`); several stack, the first on
    /// top. It follows the corner radii, elliptical ones too.
    pub fn shadow(mut self, s: Shadow) -> Self {
        if let Some(Decl::BoxShadow(list)) = self
            .decls
            .iter_mut()
            .rev()
            .find(|d| matches!(d, Decl::BoxShadow(_)))
        {
            list.push(s);
            return self;
        }
        self.push(Decl::BoxShadow(vec![s]))
    }

    /// Add a text shadow (CSS `text-shadow: x y blur color`); several
    /// stack, the first on top. They shadow the glyphs and decoration
    /// lines (a shadow's spread and inset do not apply to text and are
    /// reported as unsupported).
    pub fn text_shadow(mut self, s: Shadow) -> Self {
        if let Some(Decl::TextShadow(list)) = self
            .decls
            .iter_mut()
            .rev()
            .find(|d| matches!(d, Decl::TextShadow(_)))
        {
            list.push(s);
            return self;
        }
        self.push(Decl::TextShadow(vec![s]))
    }

    /// CSS `-webkit-text-stroke`: the outline of the glyphs, in px or em, in
    /// `color`, drawn over or under the fill as `paint_order` says.
    pub fn text_stroke(self, width: impl Into<Length>, color: impl Into<Color>) -> Self {
        self.push(Decl::TextStrokeWidth(width.into()))
            .push(Decl::TextStrokeColor(color.into()))
    }

    /// CSS `font-feature-settings`: OpenType features of the font,
    /// `font_feature_settings([("tnum", 1), ("ss01", 1)])` (a tag alone,
    /// `"cv11"`, is on). The font must have them; others do nothing.
    pub fn font_feature_settings(
        self,
        features: impl IntoIterator<Item = impl Into<FontFeature>>,
    ) -> Self {
        self.push(Decl::FontFeatureSettings(
            features.into_iter().map(Into::into).collect(),
        ))
    }

    /// CSS `font-variation-settings`: axis settings such as `wght`.
    pub fn font_variation_settings(
        self,
        settings: impl IntoIterator<Item = impl Into<FontVariation>>,
    ) -> Self {
        self.push(Decl::FontVariationSettings(
            settings.into_iter().map(Into::into).collect(),
        ))
    }

    /// CSS `font-variant-numeric: tabular-nums`: digits of one width (the
    /// font's `tnum` feature), so numbers in a column or a timer do not
    /// jiggle.
    pub fn tabular_nums(self) -> Self {
        self.push(Decl::TabularNums(true))
    }

    /// Animate changes of a property group with `t`.
    pub fn transition(self, prop: kivact_types::TransitionProp, t: Transition) -> Self {
        self.push(Decl::Transition(prop, Box::new(t)))
    }

    /// The style while the pointer is over the node or a node below it
    /// (CSS `:hover`, which a hovered child keeps on its parents).
    ///
    /// Every property applies; see the module doc.
    pub fn hover(self, s: Style) -> Self {
        self.push(Decl::State(StyleState::Hover, Box::new(s)))
    }

    /// Make the node a group for the [`group_hover`](Self::group_hover)
    /// and [`group_focus`](Self::group_focus) styles below it (Tailwind's
    /// `group`).
    pub fn group(self) -> Self {
        self.push(Decl::Group)
    }

    /// The style while the nearest ancestor made a [`group`](Self::group)
    /// is hovered (Tailwind's `group-hover:`), under the node's own
    /// [`hover`](Self::hover): a table row's actions that show while the
    /// pointer is anywhere over the row.
    ///
    /// ```
    /// use kivact::style::style;
    /// let row = style().group();
    /// let actions = style().opacity(0.0).group_hover(style().opacity(1.0));
    /// # let _ = (row, actions);
    /// ```
    pub fn group_hover(self, s: Style) -> Self {
        self.push(Decl::State(StyleState::GroupHover, Box::new(s)))
    }

    /// The style while the keyboard focus is on the nearest ancestor made a
    /// [`group`](Self::group) or on a node below it (Tailwind's
    /// `group-focus-within:`, CSS `:focus-within` read from the group),
    /// under the node's own [`focus`](Self::focus): a table row's actions
    /// that show while a control in the row has the focus.
    ///
    /// ```
    /// use kivact::style::style;
    /// let row = style().group();
    /// let actions = style().opacity(0.0).group_focus(style().opacity(1.0));
    /// # let _ = (row, actions);
    /// ```
    pub fn group_focus(self, s: Style) -> Self {
        self.push(Decl::State(StyleState::GroupFocus, Box::new(s)))
    }

    /// The style while the node is pressed (CSS `:active`).
    pub fn active(self, s: Style) -> Self {
        self.push(Decl::State(StyleState::Active, Box::new(s)))
    }

    /// The style while the element has the focus, however it got it.
    pub fn focus(self, s: Style) -> Self {
        self.push(Decl::State(StyleState::Focus, Box::new(s)))
    }

    /// The style while the element has the focus and the host asks for a
    /// focus indication (CSS `:focus-visible`): focus from the keyboard or
    /// from a program, not from a pointer press. It applies over
    /// [`focus`](Self::focus). A focusable element whose style has neither
    /// state draws the theme's focus ring here.
    pub fn focus_visible(self, s: Style) -> Self {
        self.push(Decl::State(StyleState::FocusVisible, Box::new(s)))
    }

    /// The style of a disabled node.
    pub fn disabled(self, s: Style) -> Self {
        self.push(Decl::State(StyleState::Disabled, Box::new(s)))
    }

    /// The style of a selected node.
    pub fn selected(self, s: Style) -> Self {
        self.push(Decl::State(StyleState::Selected, Box::new(s)))
    }

    /// The paint the node starts from when it is mounted (CSS
    /// `@starting-style`, Framer Motion's `initial`): its
    /// `TransitionProp::All` transition moves it from there to its
    /// paint. Only the paint properties the transition animates move; the
    /// others take their value at once.
    ///
    /// ```
    /// use kivact::motion::{Easing, Transition};
    /// use kivact::style::{TransitionProp, style};
    /// // A toast slides in from the right and fades in.
    /// let toast = style()
    ///     .enter(style().translate_x(40).opacity(0.0))
    ///     .exit(style().translate_x(40).opacity(0.0))
    ///     .transition(TransitionProp::All, Transition::tween(260, Easing::EaseOut));
    /// # let _ = toast;
    /// ```
    ///
    /// Without a paint transition, and under reduced motion with a
    /// decorative one, the node shows its paint at once.
    pub fn enter(self, s: Style) -> Self {
        self.push(Decl::State(StyleState::Enter, Box::new(s)))
    }

    /// The paint a removed node moves to before it goes (CSS exit
    /// transitions, Framer Motion's `exit`): when the node or an ancestor
    /// is removed, the removed subtree is drawn where it was until every
    /// such transition in it ends, then it goes. From the removal on, it
    /// takes no pointer, focus or accessibility node: it is gone for
    /// everything but the eye.
    ///
    /// A leaving node keeps its place in the layout until it goes, as with
    /// a CSS exit transition or Framer Motion's `AnimatePresence`: the
    /// nodes after it move into its place when it goes, not when it is
    /// removed. The node builder's `exit_layout(ExitLayout::Pop)` modifier
    /// releases that space at removal while retaining its last local box.
    ///
    /// Without a paint transition, and under reduced motion with a
    /// decorative one, the node goes at once.
    pub fn exit(self, s: Style) -> Self {
        self.push(Decl::State(StyleState::Exit, Box::new(s)))
    }

    /// CSS `backdrop-filter: blur(px)`.
    pub fn backdrop_blur(self, px: impl Into<f64>) -> Self {
        self.backdrop_filter(Filter::new().blur(px))
    }

    /// Paint locally after the node's drawing and before its children.
    /// Recipes and themes use this layer; callers use the node's `draw` layer
    /// so style merging retains the recipe's paint.
    ///
    /// The callback follows size, theme and signals it reads. A later paint
    /// replaces this callback when styles merge; unspecified paint retains it.
    /// This does not replace the node's `draw` or `draw_over` callbacks.
    /// Painting requires a box, such as `node` or `canvas`. On text, image or
    /// text-input leaves it is ignored and reported as unsupported; paint
    /// their containing box instead.
    /// Paint belongs on the base style. A paint in a state style (`hover`,
    /// `focus`, `disabled`, ...) replaces the base paint while that state
    /// applies, and the base paint comes back after it; `enter` and `exit`
    /// ignore paint. Return a base paint from a reactive style to follow
    /// interaction state.
    pub fn paint(self, f: impl Fn(&mut Draw, Size) + 'static) -> Self {
        self.push(Decl::LocalPaint(LocalPaint::new(f)))
    }

    /// CSS `place-items`: `align_items` and `justify_items` at once
    /// (`place_items(Align::Center)` centers a grid's items in their
    /// areas).
    pub fn place_items(self, a: Align) -> Self {
        self.align_items(a).justify_items(a)
    }
}
