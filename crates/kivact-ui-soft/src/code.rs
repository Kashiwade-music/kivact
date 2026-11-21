//! Role: Display an inline code fragment in the Neutral monospace treatment.

use crate::color as c;
use crate::radius as r;
use crate::space as s;
use crate::type_scale as t;
use kivact::Prop;
use kivact::component;
use kivact::state::Value;
use kivact::style::style;
use kivact::view::Element;
use kivact::view::text;

/// The ink used for an inline code fragment.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Prop)]
pub enum CodeColor {
    /// Primary text ink, the default.
    #[default]
    Primary,
    /// Secondary text ink.
    Secondary,
    /// Inherit the surrounding text ink.
    Inherit,
}

/// A monospace fragment with a muted background.
#[component]
pub fn code(
    value: Value<String>,
    #[prop(default = CodeColor::Primary)] color: Value<CodeColor>,
    #[prop(default = false)] size_inherit: Value<bool>,
) -> impl Element {
    text(value).with_node(move |node| {
        node.base_style(move || {
            let look = style()
                .font_family(t::FAMILY_CODE)
                .background(c::BG_MUTED)
                .padding_x(s::SPACE_S1)
                .border_radius(r::RADIUS_INNER);
            let look = if size_inherit.get() {
                look
            } else {
                look.font_size(t::CODE.size_px)
            };
            match color.get() {
                CodeColor::Primary => look.color(c::TEXT_PRIMARY),
                CodeColor::Secondary => look.color(c::TEXT_SECONDARY),
                CodeColor::Inherit => look,
            }
        })
    })
}
