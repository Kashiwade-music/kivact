//! Role: The breadcrumb recipe state, tokens, recipe table and default looks.

use crate::Look;
use crate::recipe::recipe_table;
use kivact_core::style::{FlexDirection, Style, Theme, style};
pub(crate) fn breadcrumb_root(t: &Theme, _: &Look<()>) -> Style {
    let p = &t.ext::<crate::UiTokens>().breadcrumb;
    style()
        .flex_direction(FlexDirection::Row)
        .gap(p.gap)
        .padding(p.padding)
        .font_size(t.get(&kivact_core::style::font::SIZE_VAR))
        .color(kivact_core::style::color::TEXT)
}

pub(crate) fn breadcrumb_list(_: &Theme, _: &Look<()>) -> Style {
    style().flex_direction(FlexDirection::Row).gap(8)
}

pub(crate) fn breadcrumb_item(t: &Theme, s: &Look<crate::BreadcrumbItemState>) -> Style {
    style()
        .padding(0)
        .height(kivact_core::style::auto())
        .min_width(0)
        .border(0, kivact_core::style::Color::TRANSPARENT)
        .background(kivact_core::style::Color::TRANSPARENT)
        .color(if s.is_current() {
            t.get(&kivact_core::style::color::TEXT_VAR)
        } else {
            t.get(&kivact_core::style::color::TEXT_MUTED_VAR)
        })
        .font_weight(400)
}

/// What a breadcrumb entry is, with its link's actual input.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BreadcrumbItemState {
    /// The current page's label (`aria-current="page"`); not a link.
    Current,
    /// A label the caller gave no activation.
    Label,
    /// A link and the actual input of its activation part.
    Link(kivact_core::custom::PartState),
}

impl BreadcrumbItemState {
    /// Whether this is the current page's entry.
    pub fn is_current(&self) -> bool {
        matches!(self, Self::Current)
    }
    /// The link's actual input; `None` for the current page and plain labels.
    pub fn input(&self) -> Option<kivact_core::custom::PartState> {
        match self {
            Self::Link(input) => Some(*input),
            Self::Current | Self::Label => None,
        }
    }
}

recipe_table! {
    /// Breadcrumb landmark, item and decorative separator recipes.
    BreadcrumbRecipes {
        root(root_recipe): () = breadcrumb_root,
        list(list_recipe): () = breadcrumb_list,
        item(item_recipe): crate::BreadcrumbItemState = breadcrumb_item,
        separator(separator_recipe): () = crate::look_common::part,
    }
}
