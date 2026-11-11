//! Role: Describe the properties a `Style` sets, for diagnostics.

use crate::sheet::{
    FlowFields, MotionFields, PaintFields, Sides, SizeFields, SpacingFields, StateFields, Style,
    TextFields, TransformFields, group_or_default,
};

/// What a hover, pressed, focus, focus-visible, group-hover, group-focus,
/// enter or exit style can change: the fields `paint_delta` reads.
const DELTA: &[&str] = &[
    "background",
    "border_width",
    "border_color",
    "border_style",
    "ring",
    "radius",
    "opacity",
    "shadow",
    "translate",
    "rotate",
    "scale",
    "transform",
    "transform_origin",
    "translate_z",
    "rotate_x",
    "rotate_y",
    "transform_origin_z",
    "perspective",
    "perspective_origin",
    "transform_style",
    "backface_visibility",
    "z_index",
    "scrollbar",
    "cursor",
    "filter",
    "backdrop_filter",
    "blend",
    "mask",
    "clip_path",
    "color",
];

/// Declares `set_properties`, which lists the properties `s` sets by the
/// name of the setter that sets them, and one classifier per group. Each
/// group of `Style` is read through `group_or_default`, so a field whose
/// group is unset reads its default.
macro_rules! classify_properties {
    (
        $(
            fn $group:ident {
                @groups $( $gname:ident = $gty:ident, )*
                @props $($value:expr => $set:expr, $name:literal;)*
            }
        )*
        @state fn $state_group:ident {
            @groups $( $sgname:ident = $sgty:ident, )*
            @props $($state_value:expr => $state_set:expr, $state_name:literal;)*
        }
    ) => {
        fn set_properties(s: &Style) -> Vec<&'static str> {
            let Style {
                size: _,
                spacing: _,
                flow: _,
                paint: _,
                transform: _,
                text: _,
                states: _,
                motion: _,
                duplicates: _,
            } = s;
            let mut out = Vec::new();
            $(out.extend($group(s));)*
            out.extend($state_group(s));
            out
        }

        /// Every property of the non-state groups that `s` sets, with the
        /// `Debug` of its value.
        fn describe_properties(s: &Style) -> Vec<(&'static str, String)> {
            let mut out = Vec::new();
            $({
                $(let $gname = group_or_default!(s.$gname, $gty);)*
                $(
                    if $set {
                        out.push(($name, format!("{:?}", $value)));
                    }
                )*
            })*
            out
        }

        $(
            fn $group(s: &Style) -> Vec<&'static str> {
                $(let $gname = group_or_default!(s.$gname, $gty);)*
                [$(($set, $name),)*]
                    .into_iter()
                    .filter_map(|(set, name)| set.then_some(name))
                    .collect()
            }
        )*
        fn $state_group(s: &Style) -> Vec<&'static str> {
            $(let $sgname = group_or_default!(s.$sgname, $sgty);)*
            [$(($state_set, $state_name),)*]
                .into_iter()
                .filter_map(|(set, name)| set.then_some(name))
                .collect()
        }
    };
}

classify_properties! {
    fn layout_properties {
        @groups size = SizeFields, spacing = SpacingFields, flow = FlowFields,
        paint = PaintFields, motion = MotionFields,
        @props motion.enter_size => motion.enter_size.is_some(), "enter_size";
        motion.exit_size => motion.exit_size.is_some(), "exit_size";
        paint.local_paint => paint.local_paint.is_some(), "paint";
        spacing.padding => spacing.padding != Sides::default(), "padding";
        spacing.margin => spacing.margin != Sides::default(), "margin";
        spacing.inset => spacing.inset != Sides::default(), "inset";
        spacing.row_gap => spacing.row_gap.is_some(), "row_gap";
        spacing.column_gap => spacing.column_gap.is_some(), "column_gap";
        size.width => size.width.is_some(), "width";
        size.height => size.height.is_some(), "height";
        size.min_width => size.min_width.is_some(), "min_width";
        size.min_height => size.min_height.is_some(), "min_height";
        size.max_width => size.max_width.is_some(), "max_width";
        size.max_height => size.max_height.is_some(), "max_height";
        size.grow => size.grow.is_some(), "grow";
        size.shrink => size.shrink.is_some(), "shrink";
        size.basis => size.basis.is_some(), "basis";
        size.aspect_ratio => size.aspect_ratio.is_some(), "aspect_ratio";
        flow.direction => flow.direction.is_some(), "direction";
        flow.align => flow.align.is_some(), "align_items";
        flow.align_self => flow.align_self.is_some(), "align_self";
        flow.justify => flow.justify.is_some(), "justify_content";
        flow.align_content => flow.align_content.is_some(), "align_content";
        flow.display => flow.display.is_some(), "display";
        flow.visibility => flow.visibility.is_some(), "visibility";
        flow.grid => flow.grid.is_some(), "grid";
        flow.wrap => flow.wrap.is_some(), "wrap";
    }
    fn position_properties {
        @groups flow = FlowFields, paint = PaintFields,
        @props flow.position => flow.position.is_some(), "position";
        flow.overflow => flow.overflow.is_some(), "overflow";
        flow.scroll_snap => flow.scroll_snap.is_some(), "scroll_snap";
        flow.scroll_axes => flow.scroll_axes.is_some(), "with_scroll_axes";
        paint.scrollbar => paint.scrollbar.is_some(), "scrollbar";
        paint.cursor => paint.cursor.is_some(), "cursor";
        flow.z_index => flow.z_index.is_some(), "z_index";
        paint.clip => paint.clip.is_some(), "clip";
    }
    fn paint_properties {
        @groups paint = PaintFields,
        @props paint.background => paint.background.is_some(), "background";
        paint.border_width => paint.border_width.iter().any(Option::is_some), "border_width";
        paint.border_color => paint.border_color.iter().any(Option::is_some), "border_color";
        paint.border_style => paint.border_style.iter().any(Option::is_some), "border_style";
        paint.ring => paint.ring.is_some(), "ring";
        paint.radius => paint.radius.iter().any(Option::is_some), "radius";
        paint.shadows => paint.shadows.is_some(), "shadow";
        paint.opacity => paint.opacity.is_some(), "opacity";
        paint.border_image => paint.border_image.is_some(), "border_image";
        paint.isolation => paint.isolation.is_some(), "isolation";
        paint.filter => paint.filter.is_some(), "filter";
        paint.backdrop_filter => paint.backdrop_filter.is_some(), "backdrop_filter";
        paint.blend => paint.blend.is_some(), "blend";
        paint.mask => paint.mask.is_some(), "mask";
        paint.clip_path => paint.clip_path.is_some(), "clip_path";
    }
    fn transform_properties {
        @groups transform = TransformFields,
        @props transform.translate => transform.translate.iter().any(Option::is_some), "translate";
        transform.rotate => transform.rotate.is_some(), "rotate";
        transform.scale => transform.scale.is_some(), "scale";
        transform.transform => transform.transform.is_some(), "transform";
        transform.transform_origin => transform.transform_origin.is_some(), "transform_origin";
        transform.translate_z => transform.translate_z.is_some(), "translate_z";
        transform.rotate_x => transform.rotate_x.is_some(), "rotate_x";
        transform.rotate_y => transform.rotate_y.is_some(), "rotate_y";
        transform.transform_origin_z => transform.transform_origin_z.is_some(), "transform_origin_z";
        transform.perspective => transform.perspective.is_some(), "perspective";
        transform.perspective_origin => transform.perspective_origin.is_some(), "perspective_origin";
        transform.transform_style => transform.transform_style.is_some(), "transform_style";
        transform.backface => transform.backface.is_some(), "backface_visibility";
        transform.will_change => transform.will_change.is_some(), "will_change";
    }
    fn text_properties {
        @groups text = TextFields,
        @props text.font_size => text.font_size.is_some(), "font_size";
        text.weight => text.weight.is_some(), "weight";
        text.color => text.color.is_some(), "color";
        text.line_height => text.line_height.is_some(), "line_height";
        text.text_wrap => text.text_wrap.is_some(), "text_wrap";
        text.text_align => text.text_align.is_some(), "text_align";
        text.font_family => text.font_family.is_some(), "font_family";
        text.italic => text.italic.is_some(), "italic";
        text.text => text.text.is_some(), "text";
    }
    @state fn state_properties {
        @groups states = StateFields, motion = MotionFields,
        @props states.hover => states.hover.is_some(), "hover";
        states.group => states.group, "group";
        states.group_hover => states.group_hover.is_some(), "group_hover";
        states.group_focus => states.group_focus.is_some(), "group_focus";
        states.pressed => states.pressed.is_some(), "pressed";
        states.focus => states.focus.is_some(), "focus";
        states.focus_visible => states.focus_visible.is_some(), "focus_visible";
        states.disabled => states.disabled.is_some(), "disabled";
        states.selected => states.selected.is_some(), "selected";
        states.enter => states.enter.is_some(), "enter";
        states.exit => states.exit.is_some(), "exit";
        motion.transitions => !motion.transitions.is_empty(), "transition";
        motion.layout_transition => motion.layout_transition.is_some(), "layout_transition";
    }
}

/// Names of the four edges of a `padding`, `margin` or `inset`.
const PADDING: [&str; 4] = [
    "padding_top",
    "padding_right",
    "padding_bottom",
    "padding_left",
];
const MARGIN: [&str; 4] = ["margin_top", "margin_right", "margin_bottom", "margin_left"];
const INSET: [&str; 4] = ["top", "right", "bottom", "left"];
/// Names of the four edges of a `border-width`, `border-color` or
/// `border-style` array.
const BORDER_WIDTH: [&str; 4] = [
    "border_top_width",
    "border_right_width",
    "border_bottom_width",
    "border_left_width",
];
const BORDER_COLOR: [&str; 4] = [
    "border_top_color",
    "border_right_color",
    "border_bottom_color",
    "border_left_color",
];
const BORDER_STYLE: [&str; 4] = [
    "border_top_style",
    "border_right_style",
    "border_bottom_style",
    "border_left_style",
];
/// Names of the four corners of a `radius`, in CSS's order.
const RADIUS: [&str; 4] = [
    "radius_top_left",
    "radius_top_right",
    "radius_bottom_right",
    "radius_bottom_left",
];
/// Names of the two axes of a `translate`.
const TRANSLATE: [&str; 2] = ["translate_x", "translate_y"];

/// Separator of the per-edge values a [`describe_style`] splits. A unit
/// separator never occurs in the `Debug` of a value.
const SEP: &str = "\u{1f}";

/// The `Debug` of each element that is set, separated by [`SEP`] (`""` for
/// an unset one).
fn options_value<T: std::fmt::Debug>(items: &[Option<T>]) -> String {
    items
        .iter()
        .map(|item| item.as_ref().map(|v| format!("{v:?}")).unwrap_or_default())
        .collect::<Vec<_>>()
        .join(SEP)
}

/// The `Debug` of each set edge of `sides`, separated by [`SEP`].
fn sides_value(sides: &Sides) -> String {
    options_value(&[sides.top, sides.right, sides.bottom, sides.left])
}

/// Split one set property's value into the names of its edges or corners.
fn split_sides(name: &'static str, value: String) -> Vec<(&'static str, String)> {
    let names: &[&'static str] = match name {
        "padding" => &PADDING,
        "margin" => &MARGIN,
        "inset" => &INSET,
        "border_width" => &BORDER_WIDTH,
        "border_color" => &BORDER_COLOR,
        "border_style" => &BORDER_STYLE,
        "radius" => &RADIUS,
        "translate" => &TRANSLATE,
        _ => return vec![(name, value)],
    };
    names
        .iter()
        .zip(value.split(SEP))
        .filter(|(_, part)| !part.is_empty())
        .map(|(name, part)| (*name, part.to_owned()))
        .collect()
}

/// The properties `s` sets, with each edge and corner under its own name
/// and the `Debug` of its value.
pub fn describe_style(s: &Style) -> Vec<(&'static str, String)> {
    let spacing = group_or_default!(s.spacing, SpacingFields);
    let paint = group_or_default!(s.paint, PaintFields);
    let transform = group_or_default!(s.transform, TransformFields);
    let mut out = Vec::new();
    for (name, value) in describe_properties(s) {
        let value = match name {
            "padding" => sides_value(&spacing.padding),
            "margin" => sides_value(&spacing.margin),
            "inset" => sides_value(&spacing.inset),
            "border_width" => options_value(&paint.border_width),
            "border_color" => options_value(&paint.border_color),
            "border_style" => options_value(&paint.border_style),
            "radius" => options_value(&paint.radius),
            "translate" => options_value(&transform.translate),
            _ => value,
        };
        out.extend(split_sides(name, value));
    }
    out
}

/// The properties of each state style `s` sets: `"hover"`, `"pressed"`,
/// `"focus"`, `"focus_visible"`, `"disabled"` and `"selected"`.
pub fn describe_state_styles(s: &Style) -> Vec<(&'static str, Vec<(&'static str, String)>)> {
    let states = group_or_default!(s.states, StateFields);
    [
        ("hover", &states.hover),
        ("pressed", &states.pressed),
        ("focus", &states.focus),
        ("focus_visible", &states.focus_visible),
        ("disabled", &states.disabled),
        ("selected", &states.selected),
    ]
    .into_iter()
    .filter_map(|(name, state)| state.as_deref().map(|state| (name, describe_style(state))))
    .collect()
}

/// Push `prefix` + `state.property` for every property the delta states
/// (hover, pressed, focus, ..) of `s` set that a delta cannot change.
fn delta_misuse(prefix: &str, s: &Style, out: &mut Vec<String>) {
    let states = group_or_default!(s.states, StateFields);
    for (name, state) in [
        ("hover", &states.hover),
        ("group_hover", &states.group_hover),
        ("group_focus", &states.group_focus),
        ("pressed", &states.pressed),
        ("focus", &states.focus),
        ("focus_visible", &states.focus_visible),
        ("enter", &states.enter),
        ("exit", &states.exit),
    ] {
        if let Some(state) = state {
            out.extend(
                set_properties(state)
                    .into_iter()
                    .filter(|property| !DELTA.contains(property))
                    .map(|property| format!("{prefix}{name}.{property}")),
            );
        }
    }
}

/// The properties set in `s`'s state styles that their state cannot change
/// (see `CompiledStyle::unsupported_in_states`).
pub(super) fn unsupported_in_states(s: &Style) -> Vec<String> {
    let states = group_or_default!(s.states, StateFields);
    let mut out = Vec::new();
    delta_misuse("", s, &mut out);
    if let Some(selected) = &states.selected {
        if selected
            .paint
            .as_deref()
            .is_some_and(|paint| paint.local_paint.is_some())
        {
            out.push("selected.paint".to_string());
        }
        if selected
            .states
            .as_deref()
            .is_some_and(|states| states.selected.is_some())
        {
            out.push("selected.selected".to_string());
        }
        delta_misuse("selected.", selected, &mut out);
    }
    if let Some(disabled) = &states.disabled {
        if disabled
            .paint
            .as_deref()
            .is_some_and(|paint| paint.local_paint.is_some())
        {
            out.push("disabled.paint".to_string());
        }
        if disabled
            .states
            .as_deref()
            .is_some_and(|states| states.disabled.is_some())
        {
            out.push("disabled.disabled".to_string());
        }
        if disabled
            .states
            .as_deref()
            .is_some_and(|states| states.selected.is_some())
        {
            out.push("disabled.selected".to_string());
        }
        delta_misuse("disabled.", disabled, &mut out);
    }
    out
}
