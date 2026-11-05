//! Role: Restyle standard action groups and toolbar surfaces with Pop tokens.
use crate::Tokens;
use kivact::style::{Theme, style};
use kivact::ui::{ButtonGroupRecipes, ToggleGroupRecipes, ToolbarRecipes};

pub(crate) fn toolbar(r: ToolbarRecipes) -> ToolbarRecipes {
    r.root(|prev| {
        move |t: &Theme, s: &_| prev(t, s).background(t.ext::<Tokens>().surface).radius(8)
    })
}
pub(crate) fn button_group(r: ButtonGroupRecipes) -> ButtonGroupRecipes {
    r.root(|prev| move |t: &Theme, s: &_| prev(t, s).radius(8))
        .separator(|prev| move |t: &Theme, s: &_| prev(t, s).background(t.ext::<Tokens>().line))
        .text(|prev| {
            move |t: &Theme, s: &_| {
                prev(t, s)
                    .background(t.ext::<Tokens>().surface_muted)
                    .color(t.ext::<Tokens>().text)
            }
        })
}
pub(crate) fn toggle_group(r: ToggleGroupRecipes) -> ToggleGroupRecipes {
    r.root(|prev| {
        move |t: &Theme, s: &_| {
            prev(t, s)
                .padding(4)
                .radius(8)
                .background(t.ext::<Tokens>().surface_muted)
        }
    })
    .item(|prev| {
        move |t: &Theme, s: &_| {
            prev(t, s)
                .radius(6)
                .hover(style().background(t.ext::<Tokens>().surface_active))
        }
    })
}
