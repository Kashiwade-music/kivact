//! Role: Style canonical breadcrumb targets with Neutral trail typography.
use kivact::style::{Align, Cursor, Direction, Style, TextDecorationLine, Theme, style};
use kivact::ui::{BreadcrumbItemState, BreadcrumbRecipes, Look};
/// Typography of a breadcrumb trail.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BreadcrumbsVariant {
    #[default]
    Default,
    Supporting,
}
impl kivact::ui::Variant for BreadcrumbsVariant {}
fn direction() -> Direction {
    if crate::stack::rtl() {
        Direction::RowReverse
    } else {
        Direction::Row
    }
}
pub(crate) fn breadcrumb(r: BreadcrumbRecipes) -> BreadcrumbRecipes {
    r.root(|_| |_: &Theme, _: &Look<()>| style().min_width(0))
        .list(|_| {
            |theme: &Theme, s: &Look<()>| {
                let t = theme.ext::<crate::Tokens>();
                let font =
                    if s.variant::<BreadcrumbsVariant>() == Some(&BreadcrumbsVariant::Supporting) {
                        t.typography.supporting
                    } else {
                        t.typography.body
                    };
                style()
                    .direction(direction())
                    .wrap(true)
                    .align_items(Align::Center)
                    .gap(t.spacing.s1)
                    .font_size(font.size_px)
                    .line_height(font.line_height_px / font.size_px)
                    .color(t.colors.text_secondary)
            }
        })
        .item(|_| item)
        .separator(|_| {
            |_: &Theme, _: &Look<()>| {
                style().scale_xy(if crate::stack::rtl() { -1.0 } else { 1.0 }, 1.0)
            }
        })
}
fn item(theme: &Theme, s: &Look<BreadcrumbItemState>) -> Style {
    let t = theme.ext::<crate::Tokens>();
    let supporting = s.variant::<BreadcrumbsVariant>() == Some(&BreadcrumbsVariant::Supporting);
    let base = style()
        .direction(direction())
        .align_items(Align::Center)
        .gap(t.spacing.s1)
        .padding(0)
        .height(kivact::style::auto())
        .min_width(0)
        .border(0, kivact::style::Color::TRANSPARENT)
        .background(kivact::style::Color::TRANSPARENT)
        .color(if (s.current || s.input.is_none()) && !supporting {
            t.colors.text_primary
        } else {
            t.colors.text_secondary
        });
    if s.current || s.input.is_none() {
        base.weight(t.typography.weight_semibold)
    } else {
        let base = if s.input.is_some_and(|input| input.hovered) {
            base.text_decoration(TextDecorationLine::UNDERLINE)
        } else {
            base
        };
        base.padding_y(t.spacing.s1)
            .cursor(Cursor::Pointer)
            .focus_visible(crate::recipe_style::ring(&t, t.colors.accent))
    }
}
