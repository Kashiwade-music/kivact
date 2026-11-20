//! Role: The one table of style properties: their declarations, setters, classes and groups.

use kivact_runtime::Transition;
use kivact_types::{
    Align, BackfaceVisibility, Cursor, Display, FlexDirection as Direction, Isolation, Justify,
    Length, Orientation, Overflow, Position, ScrollAxes, ScrollSnap, TextAlign, TransformStyle,
    TransitionProp, Visibility, WillChange,
};

use super::Style;
use super::decl::{PropClass, PropGroup};
use super::edit_text::{EditText, StyleProperty};
use super::fields::ScrollbarWidth;
use crate::color_value::Color;
use crate::draw::LocalPaint;
use crate::effect::{BlendMode, BorderStyle, Filter, Shadow, Shape};
use crate::geometry::CornerRadius;
use crate::grid::{GridAutoFlow, GridPlacement, Tracks};
use crate::paint::{Background, BorderImage, Mask};
use crate::text::{
    FontFeature, FontKerning, FontVariation, LineHeight, PaintOrder, TextDecorationLine,
    TextDecorationStyle, TextOverflow, TextTransform,
};
use crate::transform::Transform;

/// Declares the property table: `Decl` (with the hand-written variants of
/// `sheet::decl`), one setter per row that has one, and each declaration's
/// name, class and group.
macro_rules! style_props {
    (
        setters { $( $(#[$doc:meta])* $setter:ident ( $v:ident : $arg:ty ) => $variant:ident ( $ty:ty ) = $value:expr , $class:ident , $group:ident ; )* }
        longhands { $( $lvariant:ident ( $lty:ty ) = $lname:literal , $lclass:ident , $lgroup:ident ; )* }
    ) => {
        /// One declaration of a style: a property and its value.
        #[derive(Clone, Debug, PartialEq)]
        pub(crate) enum Decl {
            $( $variant($ty), )*
            $( $lvariant($lty), )*
            BoxShadow(Vec<Shadow>),
            TextShadow(Vec<Shadow>),
            Transition(TransitionProp, Box<Transition>),
            State(kivact_types::StyleState, Box<Style>),
            Group,
        }

        impl Decl {
            /// The name of the property (the setter's, or the longhand's).
            pub(crate) fn name(&self) -> &'static str {
                match self {
                    $( Decl::$variant(_) => stringify!($setter), )*
                    $( Decl::$lvariant(_) => $lname, )*
                    other => other.extra_name(),
                }
            }

            /// Whether the host can apply it as a state's paint delta.
            pub(crate) fn class(&self) -> PropClass {
                match self {
                    $( Decl::$variant(_) => PropClass::$class, )*
                    $( Decl::$lvariant(_) => PropClass::$lclass, )*
                    other => other.extra_class(),
                }
            }

            /// The group it belongs to.
            pub(crate) fn group(&self) -> PropGroup {
                match self {
                    $( Decl::$variant(_) => PropGroup::$group, )*
                    $( Decl::$lvariant(_) => PropGroup::$lgroup, )*
                    other => other.extra_group(),
                }
            }

            /// The `Debug` of its value (diagnostics, DevTools).
            pub(crate) fn value_debug(&self) -> String {
                match self {
                    $( Decl::$variant(v) => format!("{v:?}"), )*
                    $( Decl::$lvariant(v) => format!("{v:?}"), )*
                    other => other.extra_value_debug(),
                }
            }
        }

        impl Style {
            $(
                $(#[$doc])*
                pub fn $setter(self, $v: $arg) -> Self {
                    self.push(Decl::$variant($value))
                }
            )*
        }

        /// Every property of the table in DevTools form (`internal::style_property`).
        pub(crate) static STYLE_PROPERTIES: &[StyleProperty] = &[
            $( StyleProperty {
                name: stringify!($setter),
                describe: |s: &Style| s.decls.iter().rev().find_map(|d| match d {
                    Decl::$variant(v) => Some(EditText::to_edit_text(v)),
                    _ => None,
                }),
                parse: |text: &str| <$ty as EditText>::from_edit_text(text)
                    .map(|v| Style::default().push(Decl::$variant(v))),
            }, )*
            $( StyleProperty {
                name: $lname,
                describe: |s: &Style| s.decls.iter().rev().find_map(|d| match d {
                    Decl::$lvariant(v) => Some(EditText::to_edit_text(v)),
                    _ => None,
                }),
                parse: |text: &str| <$lty as EditText>::from_edit_text(text)
                    .map(|v| Style::default().push(Decl::$lvariant(v))),
            }, )*
        ];
    };
}

style_props! {
    setters {
        width(v: impl Into<Length>) => Width(Length) = v.into(), Rest, Size;
        height(v: impl Into<Length>) => Height(Length) = v.into(), Rest, Size;
        min_width(v: impl Into<Length>) => MinWidth(Length) = v.into(), Rest, Size;
        min_height(v: impl Into<Length>) => MinHeight(Length) = v.into(), Rest, Size;
        max_width(v: impl Into<Length>) => MaxWidth(Length) = v.into(), Rest, Size;
        max_height(v: impl Into<Length>) => MaxHeight(Length) = v.into(), Rest, Size;
        basis(v: impl Into<Length>) => FlexBasis(Length) = v.into(), Rest, Size;
        /// CSS `aspect-ratio`: width / height (`aspect_ratio(16.0 / 9.0)`). With
        /// one of width and height set (or stretched), the other follows.
        aspect_ratio(v: impl Into<f64>) => AspectRatio(f64) = v.into(), Rest, Size;
        grow(v: impl Into<f64>) => FlexGrow(f64) = v.into(), Rest, Size;
        shrink(v: impl Into<f64>) => FlexShrink(f64) = v.into(), Rest, Size;
        /// CSS `padding-top`; other edges remain unset.
        padding_top(v: impl Into<Length>) => PaddingTop(Length) = v.into(), Rest, Spacing;
        /// CSS `padding-right`; other edges remain unset.
        padding_right(v: impl Into<Length>) => PaddingRight(Length) = v.into(), Rest, Spacing;
        /// CSS `padding-bottom`; other edges remain unset.
        padding_bottom(v: impl Into<Length>) => PaddingBottom(Length) = v.into(), Rest, Spacing;
        /// CSS `padding-left`; other edges remain unset.
        padding_left(v: impl Into<Length>) => PaddingLeft(Length) = v.into(), Rest, Spacing;
        /// CSS `margin-top`; other edges remain unset.
        margin_top(v: impl Into<Length>) => MarginTop(Length) = v.into(), Rest, Spacing;
        /// CSS `margin-right`; other edges remain unset.
        margin_right(v: impl Into<Length>) => MarginRight(Length) = v.into(), Rest, Spacing;
        /// CSS `margin-bottom`; other edges remain unset.
        margin_bottom(v: impl Into<Length>) => MarginBottom(Length) = v.into(), Rest, Spacing;
        /// CSS `margin-left`; other edges remain unset.
        margin_left(v: impl Into<Length>) => MarginLeft(Length) = v.into(), Rest, Spacing;
        top(v: impl Into<Length>) => Top(Length) = v.into(), Rest, Spacing;
        right(v: impl Into<Length>) => Right(Length) = v.into(), Rest, Spacing;
        bottom(v: impl Into<Length>) => Bottom(Length) = v.into(), Rest, Spacing;
        left(v: impl Into<Length>) => Left(Length) = v.into(), Rest, Spacing;
        /// CSS `row-gap`: between the rows of a grid or wrapped flex lines (a
        /// column's children).
        row_gap(v: impl Into<Length>) => RowGap(Length) = v.into(), Rest, Spacing;
        /// CSS `column-gap`: between the columns of a grid (a row's
        /// children).
        column_gap(v: impl Into<Length>) => ColumnGap(Length) = v.into(), Rest, Spacing;
        direction(v: Direction) => FlexDirection(Direction) = v, Rest, Flow;
        /// Cross-axis alignment of children.
        align_items(v: Align) => AlignItems(Align) = v, Rest, Flow;
        align_self(v: Align) => AlignSelf(Align) = v, Rest, Flow;
        /// Main-axis distribution of children.
        justify_content(v: Justify) => JustifyContent(Justify) = v, Rest, Flow;
        /// CSS `align-content`: how a grid's rows (or a wrapping flex box's
        /// lines) share the free space along the cross axis.
        align_content(v: Justify) => AlignContent(Justify) = v, Rest, Flow;
        /// Select flex, grid, or a transparent container that contributes no box.
        display(v: Display) => Display(Display) = v, Rest, Flow;
        /// Keep layout space while hiding this subtree from painting and interaction.
        visibility(v: Visibility) => Visibility(Visibility) = v, Rest, Flow;
        wrap(v: bool) => FlexWrap(bool) = v, Rest, Flow;
        position(v: Position) => Position(Position) = v, Rest, Flow;
        /// Layout overflow on both axes; Hidden also clips layout overflow.
        /// For an axis-specific accessible scroll container, use `scroll`.
        overflow(v: Overflow) => Overflow(Overflow) = v, Rest, Flow;
        /// Align direct in-flow children to scroll stops after touch scrolling.
        scroll_snap(v: ScrollSnap) => ScrollSnap(ScrollSnap) = v, Rest, Flow;
        /// CSS `z-index`: the order the node is painted and hit in among
        /// its siblings (higher on top; equal values keep the tree order).
        /// Every container isolates its children's z-indices (as CSS
        /// `isolation: isolate`): a negative one paints after the parent's
        /// own background, below its other children. Tab order stays the
        /// tree order.
        z_index(v: i32) => ZIndex(i32) = v, Delta, Flow;
        /// CSS `grid-template-columns`: the grid's columns, a [`Track`] or a
        /// tuple of them: `(248, Track::fr(1))`,
        /// `Track::repeat_auto_fill(Track::minmax(330, Track::fr(1)))`,
        /// `(Track::minmax(70, pct(38)), Track::fr(1), 20)`. For a `grid`
        /// box.
        grid_template_columns(v: impl Into<Tracks>) => GridTemplateColumns(Tracks) = v.into(), Rest, Flow;
        /// CSS `grid-template-rows`.
        grid_template_rows(v: impl Into<Tracks>) => GridTemplateRows(Tracks) = v.into(), Rest, Flow;
        /// CSS `grid-auto-columns`: the size of the columns auto-placed
        /// items make beyond the template (cycling through the list).
        grid_auto_columns(v: impl Into<Tracks>) => GridAutoColumns(Tracks) = v.into(), Rest, Flow;
        /// CSS `grid-auto-rows`: the size of the rows beyond the template
        /// (`grid_auto_rows(120)` makes every new row 120 px high).
        grid_auto_rows(v: impl Into<Tracks>) => GridAutoRows(Tracks) = v.into(), Rest, Flow;
        /// CSS `grid-auto-flow`.
        grid_auto_flow(v: GridAutoFlow) => GridAutoFlow(GridAutoFlow) = v, Rest, Flow;
        /// CSS `grid-column` of a grid item: `GridPlacement::span(4)`,
        /// `(1, -1)` (the whole row), `2`, `(2, GridPlacement::span(3))`.
        grid_column(v: impl Into<GridPlacement>) => GridColumn(Box<GridPlacement>) = Box::new(v.into()), Rest, Flow;
        /// CSS `grid-row` of a grid item.
        grid_row(v: impl Into<GridPlacement>) => GridRow(Box<GridPlacement>) = Box::new(v.into()), Rest, Flow;
        /// CSS `justify-items`: how a grid's items sit in their areas along
        /// the row (default: stretch).
        justify_items(v: Align) => JustifyItems(Align) = v, Rest, Flow;
        /// CSS `justify-self`: how this grid item sits in its area along
        /// the row.
        justify_self(v: Align) => JustifySelf(Align) = v, Rest, Flow;
        /// The background (CSS `background`): a color, a gradient, a
        /// [`Paint`](crate::paint::Paint) or a tuple of them stacked like CSS layers, the first
        /// on top (see [`Background`]). Layers are placed in the border box
        /// and clipped to its rounded shape.
        background(v: impl Into<Background>) => Background(Background) = v.into(), Delta, Paint;
        /// The top border's line style alone (CSS `border-top-style`).
        border_top_style(v: BorderStyle) => BorderTopStyle(BorderStyle) = v, Delta, Paint;
        /// The right border's line style alone (CSS `border-right-style`).
        border_right_style(v: BorderStyle) => BorderRightStyle(BorderStyle) = v, Delta, Paint;
        /// The bottom border's line style alone (CSS `border-bottom-style`).
        border_bottom_style(v: BorderStyle) => BorderBottomStyle(BorderStyle) = v, Delta, Paint;
        /// The left border's line style alone (CSS `border-left-style`).
        border_left_style(v: BorderStyle) => BorderLeftStyle(BorderStyle) = v, Delta, Paint;
        /// CSS `border-top-left-radius`.
        radius_top_left(v: impl Into<CornerRadius>) => RadiusTopLeft(Box<CornerRadius>) = Box::new(v.into()), Delta, Paint;
        /// CSS `border-top-right-radius`.
        radius_top_right(v: impl Into<CornerRadius>) => RadiusTopRight(Box<CornerRadius>) = Box::new(v.into()), Delta, Paint;
        /// CSS `border-bottom-right-radius`.
        radius_bottom_right(v: impl Into<CornerRadius>) => RadiusBottomRight(Box<CornerRadius>) = Box::new(v.into()), Delta, Paint;
        /// CSS `border-bottom-left-radius`.
        radius_bottom_left(v: impl Into<CornerRadius>) => RadiusBottomLeft(Box<CornerRadius>) = Box::new(v.into()), Delta, Paint;
        /// The opacity of the node and its subtree, as one group (CSS
        /// `opacity`: overlapping children do not show through each
        /// other).
        opacity(v: impl Into<f64>) => Opacity(f64) = f64::clamp(v.into(), 0.0, 1.0), Delta, Paint;
        /// CSS `isolation`.
        isolation(v: Isolation) => Isolation(Isolation) = v, Rest, Paint;
        /// CSS `border-image`: a picture or a gradient cut into nine pieces
        /// that fill the border.
        border_image(v: BorderImage) => BorderImage(Box<BorderImage>) = Box::new(v), Rest, Paint;
        /// Set overlay scrollbar width; no layout space is reserved.
        scrollbar_width(v: impl Into<ScrollbarWidth>) => ScrollbarWidth(ScrollbarWidth) = v.into(), Delta, Paint;
        cursor(v: Cursor) => Cursor(Cursor) = v, Delta, Paint;
        /// Clip drawing to the box without changing layout overflow.
        /// Use `overflow(Overflow::Hidden)` to affect layout overflow too.
        clip(v: bool) => Clip(bool) = v, Rest, Paint;
        /// CSS `filter`: color functions recolor everything the subtree
        /// draws; `blur` and `drop_shadow` filter the subtree's pixels.
        filter(v: Filter) => Filter(Box<Filter>) = Box::new(v), Delta, Paint;
        /// CSS `backdrop-filter`: blur and recolor what lies behind the
        /// border box (glass), beneath the node's own background, which is
        /// then usually translucent.
        backdrop_filter(v: Filter) => BackdropFilter(Box<Filter>) = Box::new(v), Delta, Paint;
        /// CSS `mix-blend-mode`.
        mix_blend_mode(v: BlendMode) => MixBlendMode(BlendMode) = v, Delta, Paint;
        /// CSS `mask`: the node and its subtree keep the alpha of these
        /// layers (`mask(linear_gradient(180).colors([Color::BLACK,
        /// Color::TRANSPARENT]))` fades them out downwards).
        mask(v: impl Into<Mask>) => Mask(Box<Mask>) = Box::new(v.into()), Delta, Paint;
        /// CSS `clip-path`: the node's own paint (background, border,
        /// shadows) and its subtree are clipped to `shape`, and so are
        /// hit testing and pointer input: a press outside the shape does
        /// not reach the node.
        clip_path(v: Shape) => ClipPath(Box<Shape>) = Box::new(v), Delta, Paint;
        /// CSS `transform`: the functions of `t` in their order
        /// (`Transform::new().skew_x(-12).translate_y(2)`), after the
        /// `translate`, `rotate` and `scale` properties, about the
        /// `transform_origin` (the box's centre by default). It maps the
        /// box and its whole subtree, which is hit, pointed at
        /// (`PointerEvent::pos` stays in the untransformed local space),
        /// reported (`NodeRef::rect`: the bounds as drawn) and exposed
        /// where it is drawn; layout ignores it (siblings do not move).
        transform(v: Transform) => Transform(Box<Transform>) = Box::new(v), Delta, Transform;
        /// The horizontal part of [`translate`](Self::translate) alone
        /// (Tailwind's `translate-x-*`). Each axis is a property of its own: a
        /// style merged over this one (a later layer, or a `hover`, `pressed`
        /// or `focus` state) that sets only the other axis keeps this one, so a
        /// press that nudges a node down keeps its horizontal centring by
        /// translate. A layer or state that sets this axis replaces it, as in
        /// CSS.
        translate_x(v: impl Into<Length>) => TranslateX(Length) = v.into(), Delta, Transform;
        /// The vertical part of [`translate`](Self::translate) alone
        /// (Tailwind's `translate-y-*`), merged on its own like
        /// [`translate_x`](Self::translate_x).
        translate_y(v: impl Into<Length>) => TranslateY(Length) = v.into(), Delta, Transform;
        /// CSS `translate` on the z axis, in px (`translateZ`).
        translate_z(v: impl Into<f64>) => TranslateZ(f64) = v.into(), Delta, Transform;
        /// CSS `rotate`: clockwise, in degrees, after `translate` and
        /// before `scale` and `transform` (CSS's order for the individual
        /// transform properties, whatever the call order).
        rotate(v: impl Into<f64>) => Rotate(f64) = v.into(), Delta, Transform;
        /// CSS `rotate` about the x axis, in degrees (`rotateX`).
        rotate_x(v: impl Into<f64>) => RotateX(f64) = v.into(), Delta, Transform;
        /// CSS `rotate` about the y axis, in degrees (`rotateY`).
        rotate_y(v: impl Into<f64>) => RotateY(f64) = v.into(), Delta, Transform;
        /// The z of the transform origin, in px (the third value of CSS `transform-origin`).
        transform_origin_z(v: impl Into<f64>) => TransformOriginZ(f64) = v.into(), Delta, Transform;
        /// CSS `perspective` of this node's children, in px: the distance of
        /// the viewer. The origin is the center of the box unless
        /// [`perspective_origin`](Self::perspective_origin) says otherwise.
        perspective(v: impl Into<f64>) => Perspective(f64) = v.into(), Delta, Transform;
        /// CSS `transform-style`: `Preserve3d` keeps the children in one 3D space.
        transform_style(v: TransformStyle) => TransformStyle(TransformStyle) = v, Delta, Transform;
        /// CSS `backface-visibility`.
        backface_visibility(v: BackfaceVisibility) => BackfaceVisibility(BackfaceVisibility) = v, Delta, Transform;
        /// CSS `will-change`.
        will_change(v: WillChange) => WillChange(WillChange) = v, Rest, Transform;
        /// CSS `font-size` of the text in this node and below.
        /// A number is px; `font::SIZE` and the other font size variables of the theme are lengths too.
        font_size(v: impl Into<Length>) => FontSize(Length) = v.into(), Rest, Text;
        /// CSS `font-weight` (1 to 1000) of the text in this node and below.
        weight(v: u16) => FontWeight(u16) = v, Rest, Text;
        /// CSS `color` of the text in this node and below. As in CSS, each
        /// text property is inherited on its own: a descendant that sets
        /// only a `weight` or a `font_size` keeps this color. Without one
        /// on any ancestor, text takes the App theme's
        /// [`kivact::style::color::TEXT`](kivact::style::color::TEXT); a
        /// `provide_theme` subtree gets its theme's by a
        /// `color(kivact::style::color::TEXT)` on its top node.
        color(v: impl Into<Color>) => Color(Color) = v.into(), Delta, Text;
        /// CSS `line-height` of the text in this node and below. A number is
        /// a multiple of the font size, as CSS's unitless value:
        /// `line_height(1.5)` is `line-height: 1.5`. A length in px is
        /// `line_height(LineHeight::px(20))` (`line-height: 20px`), and
        /// `LineHeight::Normal` is `normal`: the font size times 1.3,
        /// rounded (a value like the others: it does not inherit the
        /// theme's). Text that sets none inherits the nearest ancestor's,
        /// else `Theme.text.line_height`. Not finite or not above zero is
        /// `normal`.
        line_height(v: impl Into<LineHeight>) => LineHeight(LineHeight) = v.into(), Rest, Text;
        /// Whether the text wraps at the box's width.
        text_wrap(v: bool) => TextWrap(bool) = v, Rest, Text;
        text_align(v: TextAlign) => TextAlign(TextAlign) = v, Rest, Text;
        /// Font family of the text in this node and below (CSS
        /// `font-family`): a family name (`"Inter"`, a font
        /// registered with `App::font` or installed on the system) or a
        /// generic family (`"system-ui"`, `"sans-serif"`, `"serif"`,
        /// `"monospace"`). Descendants inherit it, also those that set
        /// other text properties. A family that is not installed falls back
        /// to the system UI font and is reported once as `ErrorKind::Asset`.
        /// An ordered comma-separated list (such as
        /// `"Inter, 'Noto Sans JP', sans-serif"`) selects a font per glyph
        /// before system fallback; unavailable entries are skipped without
        /// diagnostics. Quote names containing commas. Malformed lists use
        /// system fallback. Without one, text uses `Theme.text.family`.
        font_family(v: impl Into<String>) => FontFamily(String) = v.into(), Rest, Text;
        /// Italic text (the family's italic face when it has one).
        italic(v: bool) => Italic(bool) = v, Rest, Text;
        /// CSS `letter-spacing`: extra space after every character, in px
        /// (`letter_spacing(1)`) or em (`letter_spacing(em(0.08))`, of the
        /// text's font size; negative tightens).
        letter_spacing(v: impl Into<Length>) => LetterSpacing(Length) = v.into(), Rest, Text;
        /// CSS `word-spacing`: extra space after every space, in px
        /// (`word_spacing(4)`) or em (`word_spacing(em(0.25))`, of the
        /// text's font size; negative tightens). It adds to
        /// `text_align(TextAlign::Justify)`'s widening.
        word_spacing(v: impl Into<Length>) => WordSpacing(Length) = v.into(), Rest, Text;
        /// CSS `font-kerning`: `FontKerning::None` turns the OpenType `kern`
        /// feature off, `Normal` on, `Auto` (the default) leaves it to the
        /// font. A `("kern", ..)` in `font_feature_settings` wins.
        font_kerning(v: FontKerning) => FontKerning(FontKerning) = v, Rest, Text;
        /// CSS `text-decoration-line` (and the `text-decoration: underline`
        /// shorthand): `text_decoration(TextDecorationLine::UNDERLINE)`.
        /// The lines are drawn in the text's color unless
        /// `text_decoration_color` says otherwise; a span of rich text can
        /// set its own.
        text_decoration(v: TextDecorationLine) => TextDecorationLine(TextDecorationLine) = v, Rest, Text;
        /// CSS `text-decoration-style`: the pattern of the decoration lines.
        text_decoration_style(v: TextDecorationStyle) => TextDecorationStyle(TextDecorationStyle) = v, Rest, Text;
        /// CSS `text-decoration-color` (default: the text's color).
        text_decoration_color(v: impl Into<Color>) => TextDecorationColor(Color) = v.into(), Rest, Text;
        /// CSS `text-decoration-thickness` in px or em (default: the font's).
        text_decoration_thickness(v: impl Into<Length>) => TextDecorationThickness(Length) = v.into(), Rest, Text;
        /// CSS `text-underline-offset`: the underline this much further
        /// from the text than the font puts it (`text_underline_offset(3)`).
        text_underline_offset(v: impl Into<Length>) => TextUnderlineOffset(Length) = v.into(), Rest, Text;
        /// CSS `text-overflow`: `TextOverflow::Ellipsis` cuts a line that
        /// does not fit its box and ends it in "…" (with
        /// `text_wrap(false)`, CSS's `white-space: nowrap`; such a text can
        /// shrink down to the ellipsis in a row). The accessible name is
        /// the whole text.
        text_overflow(v: TextOverflow) => TextOverflow(TextOverflow) = v, Rest, Text;
        /// CSS `line-clamp`: show at most `n` lines, the last ending in
        /// "…" when more follow (the box is as high as the lines shown).
        line_clamp(v: u32) => LineClamp(u32) = v, Rest, Text;
        /// Trim the first line to the measured cap height and the last to its baseline.
        text_capsize(v: bool) => TextCapsize(bool) = v, Rest, Text;
        /// CSS `text-transform`: draw the text in upper case, lower case
        /// or capitalized; its accessible name keeps it as written.
        text_transform(v: TextTransform) => TextTransform(TextTransform) = v, Rest, Text;
        /// CSS `paint-order`: whether the fill or the stroke of the glyphs is drawn first.
        paint_order(v: PaintOrder) => PaintOrder(PaintOrder) = v, Rest, Text;
        /// CSS `font-optical-sizing`: whether the optical size axis follows the font size.
        font_optical_sizing(v: bool) => FontOpticalSizing(bool) = v, Rest, Text;
        /// CSS `font-stretch` as a percentage of the normal width (50 to 200).
        font_width(v: impl Into<f64>) => FontWidth(f64) = v.into(), Rest, Text;
        /// CSS `caret-color` of a text field.
        caret_color(v: impl Into<Color>) => CaretColor(Color) = v.into(), Rest, Text;
        /// The band behind a text field's selected text (CSS
        /// `::selection { background }`).
        selection_color(v: impl Into<Color>) => SelectionColor(Color) = v.into(), Rest, Text;
        /// A text field's placeholder color (CSS `::placeholder { color }`;
        /// default: the text color at half its alpha).
        placeholder_color(v: impl Into<Color>) => PlaceholderColor(Color) = v.into(), Rest, Text;
        /// Reveal the selected axis at entrance when Size has a transition.
        enter_size(v: Orientation) => EnterSize(Orientation) = v, Rest, Motion;
        /// Collapse the selected axis before removal when Size has a transition.
        exit_size(v: Orientation) => ExitSize(Orientation) = v, Rest, Motion;
        /// Animate position and size changes after layout, using a visual transform.
        /// Layout is computed once; painting interpolates from the previous rectangle.
        layout_transition(v: Transition) => LayoutTransition(Box<Transition>) = Box::new(v), Rest, Motion;
    }
    longhands {
        BorderTopWidth(f64) = "border_top_width", Rest, Paint;
        BorderRightWidth(f64) = "border_right_width", Rest, Paint;
        BorderBottomWidth(f64) = "border_bottom_width", Rest, Paint;
        BorderLeftWidth(f64) = "border_left_width", Rest, Paint;
        BorderTopColor(Color) = "border_top_color", Delta, Paint;
        BorderRightColor(Color) = "border_right_color", Delta, Paint;
        BorderBottomColor(Color) = "border_bottom_color", Delta, Paint;
        BorderLeftColor(Color) = "border_left_color", Delta, Paint;
        RingWidth(f64) = "ring_width", Delta, Paint;
        RingOffset(f64) = "ring_offset", Delta, Paint;
        RingColor(Color) = "ring_color", Delta, Paint;
        ScrollbarThumbColor(Color) = "scrollbar_thumb_color", Delta, Paint;
        ScrollbarTrackColor(Color) = "scrollbar_track_color", Delta, Paint;
        Scale((f64, f64)) = "scale", Delta, Transform;
        TransformOrigin(Box<(Length, Length)>) = "transform_origin", Delta, Transform;
        PerspectiveOrigin(Box<(Length, Length)>) = "perspective_origin", Delta, Transform;
        LocalPaint(LocalPaint) = "paint", Rest, Paint;
        ScrollAxes(ScrollAxes) = "scroll_axes", Rest, Flow;
        TextStrokeWidth(Length) = "text_stroke_width", Rest, Text;
        TextStrokeColor(Color) = "text_stroke_color", Rest, Text;
        FontFeatureSettings(Vec<FontFeature>) = "font_feature_settings", Rest, Text;
        FontVariationSettings(Vec<FontVariation>) = "font_variation_settings", Rest, Text;
        TabularNums(bool) = "tabular_nums", Rest, Text;
    }
}
