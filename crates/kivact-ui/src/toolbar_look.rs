//! Role: Derive toolbar layout and divider styles from passive theme metrics.
use crate::collection_values::{ToolbarEdge, ToolbarState};
use kivact_core::style::{Align, Direction, Justify, Length, Track, pct, style};
use kivact_core::view::Orientation;

pub(crate) fn root_style(
    theme: &kivact_core::style::Theme,
    look: &crate::Look<ToolbarState>,
) -> kivact_core::style::Style {
    let t = &theme.ext::<crate::UiTokens>().toolbar;
    let vertical = look.orientation == Orientation::Vertical;
    let mut base = style()
        .background(t.background)
        .padding(t.padding)
        .min_height(t.min_height)
        .width(pct(100))
        .align_items(if vertical {
            Align::Stretch
        } else {
            Align::Center
        })
        .direction(if vertical {
            Direction::Column
        } else if look.rtl {
            Direction::RowReverse
        } else {
            Direction::Row
        })
        .justify_content(Justify::SpaceBetween);
    if look.centered && !vertical {
        base = base
            .display(kivact_core::style::Display::Grid)
            .grid_template_columns((
                Track::minmax(0, Track::fr(1)),
                Length::Auto,
                Track::minmax(0, Track::fr(1)),
            ));
    }
    for edge in &look.dividers {
        base = match edge {
            ToolbarEdge::Top => base.border_top(1, t.border),
            ToolbarEdge::Bottom => base.border_bottom(1, t.border),
            ToolbarEdge::Start if !look.rtl => base.border_left(1, t.border),
            ToolbarEdge::End if look.rtl => base.border_left(1, t.border),
            _ => base.border_right(1, t.border),
        };
    }
    base
}
