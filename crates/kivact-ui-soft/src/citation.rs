//! Role: Present a document source as an inline label or numbered citation.
use crate::color as c;
use crate::icon::{IconSize, icon_view};
use crate::metrics as m;
use crate::motion::{EASE_STANDARD, FAST_MAX};
use crate::radius as r;
use crate::recipe_parts::ring;
use crate::space as s;
use crate::type_scale as t;

use kivact::Prop;
use kivact::component;
use kivact::custom::Caller;
use kivact::i18n::message_with;
use kivact::motion::Transition;
use kivact::state::Callback;
use kivact::style::Align;
use kivact::style::Cursor;
use kivact::style::FlexDirection;
use kivact::style::Icon;
use kivact::style::Justify;
use kivact::style::TransitionProp;
use kivact::style::style;
use kivact::style::use_theme;
use kivact::view::Asset;
use kivact::view::Element;
use kivact::view::ImageFit;
use kivact::view::IntoView;
use kivact::view::Node;
use kivact::view::ReadingDirection;
use kivact::view::Role;
use kivact::view::View;
use kivact::view::image;
use kivact::view::node;
use kivact::view::text;
use kivact::view::use_direction;

/// Source metadata used to label and open a citation.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CitationSource {
    /// Optional title used as the citation label.
    pub title: Option<String>,
    /// Destination opened when the citation is activated.
    pub url: Option<String>,
    /// Optional image shown beside the citation label.
    pub image: Option<Asset>,
    /// Optional glyph shown when no source image is available.
    pub icon: Option<Icon>,
}

/// The visual treatment of a citation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Prop)]
pub enum CitationVariant {
    #[default]
    /// Show the source title or URL, falling back to a labelled source number.
    Label,
    /// Show the citation number as the trigger label.
    Number,
}

impl kivact::ui::theme::Variant for CitationVariant {}

/// Render a numbered reference to a source; `on_open` receives the source URL when activated.
#[component]
pub fn citation(
    number: u32,
    source: CitationSource,
    on_open: Option<Callback<String>>,
    #[caller] caller: Caller,
) -> impl Element {
    let number_only = caller
        .variants()
        .get::<CitationVariant>()
        .unwrap_or_default()
        == CitationVariant::Number;
    let title = source.title.or_else(|| source.url.clone());
    let name = message_with("soft.citation.source", "Source {number}", move || {
        [("number", number.to_string())]
    });
    let label = if number_only {
        text(number.to_string())
    } else if let Some(title) = title.clone() {
        text(title)
    } else {
        text(name)
    };
    let rtl = use_direction() == ReadingDirection::Rtl;
    let media = citation_media(number_only, source.image, source.icon);
    let has_media = media.is_some();
    let interactive = source.url.is_some() && on_open.is_some();
    let mut root = node();
    if let (Some(url), Some(open)) = (source.url, on_open) {
        root = root
            .with_node(move |node| node.role(Role::Link))
            .focusable(true)
            .on_press(move || open.call(url.clone()));
    }
    root.name(move || title.clone().unwrap_or_else(|| name.get()))
        .with_node(move |node| {
            node.base_style(move || citation_style(number_only, has_media, interactive, rtl))
        })
        .child(media.map(citation_media_frame))
        .child(label.style(style().single_line()))
}

fn citation_media(number_only: bool, asset: Option<Asset>, glyph: Option<Icon>) -> Option<View> {
    if number_only {
        None
    } else if let Some(asset) = asset {
        Some(
            image(asset)
                .fit(ImageFit::Cover)
                .style(|| style().size(m::ICON_XSM, m::ICON_XSM))
                .into_view(),
        )
    } else {
        glyph.map(|glyph| icon_view(glyph).size(IconSize::Sm).into_view())
    }
}

fn citation_media_frame(media: View) -> Node {
    node()
        .style(|| {
            style()
                .size(m::ICON_SM, m::ICON_SM)
                .flex_shrink(0)
                .clip(true)
                .border_radius(r::RADIUS_FULL)
                .background(c::BG_SURFACE)
                .border(m::BORDER_WIDTH, c::BORDER)
                .align_items(Align::Center)
                .justify_content(Justify::Center)
        })
        .child(media)
}

fn citation_style(
    number_only: bool,
    has_media: bool,
    interactive: bool,
    rtl: bool,
) -> kivact::style::Style {
    let look = style()
        .flex_direction(if rtl {
            FlexDirection::RowReverse
        } else {
            FlexDirection::Row
        })
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .height(s::SPACE_S5)
        .gap(s::SPACE_S1)
        .font_size(t::SUPPORTING.size_px)
        .line_height(t::SUPPORTING.line_height_px / t::SUPPORTING.size_px)
        .color(c::TEXT_SECONDARY);
    let look = citation_shape(look, number_only, has_media, rtl);
    if interactive {
        look.cursor(Cursor::Pointer)
            .hover(style().background(c::OVERLAY_HOVER).color(c::TEXT_PRIMARY))
            .focus_visible(ring(c::FOCUS_RING))
            .transition(
                TransitionProp::All,
                Transition::tween(FAST_MAX as f64, EASE_STANDARD),
            )
    } else {
        look
    }
}

fn citation_shape(
    look: kivact::style::Style,
    number_only: bool,
    has_media: bool,
    rtl: bool,
) -> kivact::style::Style {
    if number_only {
        look.min_width(s::SPACE_S5)
            .border_radius(r::RADIUS_FULL)
            .padding_x(s::SPACE_S1)
            .background(c::ACCENT_MUTED)
            .font_weight(t::WEIGHT_SEMIBOLD)
            .translate_y(-use_theme().get().get(&crate::space::SPACE_S1_VAR))
    } else {
        let start = if has_media {
            s::SPACE_S0_5
        } else {
            s::SPACE_S2
        };
        let look = look
            .border_radius(r::RADIUS_ELEMENT)
            .max_width(t::SUPPORTING.size_px * 15.0)
            .border(m::BORDER_WIDTH, c::BORDER)
            .padding_x(s::SPACE_S2);
        if rtl {
            look.padding_right(start).margin_right(s::SPACE_S0_5)
        } else {
            look.padding_left(start).margin_left(s::SPACE_S0_5)
        }
    }
}
