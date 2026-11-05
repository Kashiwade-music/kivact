//! Role: Present a document source as an inline label or numbered citation.
use crate::{IconSize, icon};
use kivact::i18n::message_with;
use kivact::motion::Transition;
use kivact::state::IntoValue;
use kivact::state::Value;
use kivact::style::Align;
use kivact::style::Cursor;
use kivact::style::Direction;
use kivact::style::Icon;
use kivact::style::Justify;
use kivact::style::TransitionProp;
use kivact::style::style;
use kivact::view::Asset;
use kivact::view::HandlerResult;
use kivact::view::ImageFit;
use kivact::view::IntoView;
use kivact::view::Node;
use kivact::view::NodeBuilder;
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
    pub title: Option<String>,
    pub url: Option<String>,
    pub image: Option<Asset>,
    pub icon: Option<Icon>,
}

/// The visual treatment of a citation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CitationVariant {
    #[default]
    Label,
    Number,
}

type OpenSource = kivact::state::Callback<String>;

/// A citation with an optional application-provided link action.
pub struct Citation {
    root: Node,
    number: u32,
    source: CitationSource,
    variant: Value<CitationVariant>,
    open: Option<OpenSource>,
}

/// Render a numbered reference to a source.
pub fn citation(number: u32, source: CitationSource) -> Citation {
    Citation {
        root: node(),
        number,
        source,
        variant: (CitationVariant::default()).into_value(),
        open: None,
    }
}

impl Citation {
    /// Select a label or a number.
    pub fn variant(mut self, variant: impl IntoValue<CitationVariant>) -> Self {
        self.variant = variant.into_value();
        self
    }
    /// Pass the source URL to the application when activated.
    pub fn on_open<R: HandlerResult>(mut self, action: impl Fn(&str) -> R + 'static) -> Self {
        self.open = Some(kivact::state::callback(move |value: String| action(&value)));
        self
    }
}

impl NodeBuilder for Citation {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for Citation {
    fn into_view(self) -> View {
        let Self {
            mut root,
            number,
            source,
            variant,
            open,
        } = self;
        let number_only = variant.get() == CitationVariant::Number;
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
        let rtl = use_direction().get() == ReadingDirection::Rtl;
        let media = if number_only {
            None
        } else if let Some(asset) = source.image {
            Some(
                image(asset)
                    .fit(ImageFit::Cover)
                    .style(|| {
                        let size = kivact::style::use_tokens::<crate::Tokens>().sizes.icon_xsm;
                        style().size(size, size)
                    })
                    .into_view(),
            )
        } else {
            source
                .icon
                .map(|glyph| icon(glyph).size(IconSize::Sm).into_view())
        };
        let has_media = media.is_some();
        let interactive = source.url.is_some() && open.is_some();
        if let (Some(url), Some(open)) = (source.url, open) {
            root = root
                .with_node(move |node| node.role(Role::Link))
                .focusable(true)
                .on_press(move || open.call(url.clone()));
        }
        root.name(move || title.clone().unwrap_or_else(|| name.get()))
            .with_node(move |node| {
                node.base_style(move || {
                    let p = kivact::style::use_tokens::<crate::Tokens>();
                    let look = style()
                        .direction(if rtl {
                            Direction::RowReverse
                        } else {
                            Direction::Row
                        })
                        .align_items(Align::Center)
                        .justify_content(Justify::Center)
                        .height(p.spacing.s5)
                        .gap(p.spacing.s1)
                        .font_size(p.typography.supporting.size_px)
                        .line_height(
                            p.typography.supporting.line_height_px
                                / p.typography.supporting.size_px,
                        )
                        .color(p.colors.text_secondary);
                    let look = if number_only {
                        look.min_width(p.spacing.s5)
                            .radius(p.radius.full)
                            .padding_x(p.spacing.s1)
                            .background(p.colors.accent_muted)
                            .weight(p.typography.weight_semibold)
                            .translate_y(-p.spacing.s1)
                    } else {
                        let start = if has_media {
                            p.spacing.s0_5
                        } else {
                            p.spacing.s2
                        };
                        let look = look
                            .radius(p.radius.element)
                            .max_width(p.typography.supporting.size_px * 15.0)
                            .border(p.sizes.border_width, p.colors.border)
                            .padding_x(p.spacing.s2);
                        if rtl {
                            look.padding_right(start).margin_right(p.spacing.s0_5)
                        } else {
                            look.padding_left(start).margin_left(p.spacing.s0_5)
                        }
                    };
                    if interactive {
                        look.cursor(Cursor::Pointer)
                            .hover(
                                style()
                                    .background(p.colors.overlay_hover)
                                    .color(p.colors.text_primary),
                            )
                            .focus_visible(style().ring_with_offset(
                                p.sizes.focus_ring_width,
                                p.sizes.focus_ring_offset,
                                p.colors.focus_ring,
                            ))
                            .transition(
                                TransitionProp::Paint,
                                Transition::tween(p.motion.fast_max as f64, p.motion.ease_standard),
                            )
                    } else {
                        look
                    }
                })
            })
            .child(media.map(|media| {
                node()
                    .style(|| {
                        let p = kivact::style::use_tokens::<crate::Tokens>();
                        let side = p.sizes.icon_sm;
                        style()
                            .size(side, side)
                            .shrink(0)
                            .clip(true)
                            .radius(p.radius.full)
                            .background(p.colors.bg_surface)
                            .border(p.sizes.border_width, p.colors.border)
                            .align_items(Align::Center)
                            .justify_content(Justify::Center)
                    })
                    .child(media)
            }))
            .child(label.style(style().single_line()))
            .into_view()
    }
}

kivact::state::constant_value!(CitationVariant);
