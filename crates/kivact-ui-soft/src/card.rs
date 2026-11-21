//! Role: Supply a canonical card's layout inset context without a card wrapper.
use crate::layout::LayoutInset;
use crate::metrics as m;
use crate::space::SPACE_CARD_PADDING_VAR;
use crate::stack::ContainerInset;
use crate::variants::{CardPadding, CardVariant};
use kivact::state::provide;
use kivact::style::use_theme;
use kivact::view::IntoView;

/// Provide the card's content inset to Soft layout children, without adding a control or node.
pub fn with_card_insets<V: IntoView>(
    variant: CardVariant,
    padding: Option<CardPadding>,
    content: impl FnOnce() -> V + 'static,
) -> impl IntoView {
    let logical = padding.map_or_else(
        || use_theme().get().get(&SPACE_CARD_PADDING_VAR),
        |padding| padding.0.px(),
    );
    let border = if variant == CardVariant::Default {
        m::BORDER_WIDTH
    } else {
        0.0
    };
    let physical = (logical - border).max(0.0);
    provide(ContainerInset::uniform(physical), move || {
        provide(
            LayoutInset::from_padding(ContainerInset::uniform(logical)),
            content,
        )
    })
}
