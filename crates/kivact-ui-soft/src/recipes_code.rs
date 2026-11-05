//! Role: Style canonical source blocks using Soft visual variants.
use crate::Tokens;
use kivact::state::{IntoValue, Value};
use kivact::style::{Direction, Justify, Style, Theme, pct, style};
use kivact::ui::{CodeBlockRecipes, CodeBlockState, Look};
/// Source text size.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CodeBlockSize {
    /// Use small source typography.
    Sm,
    #[default]
    /// Use standard source typography.
    Md,
}
/// Outer width policy.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum CodeBlockWidth {
    #[default]
    /// Fit the source block within the available width.
    Fit,
    /// Fill the available width.
    Fill,
    /// Use the specified outer width in pixels.
    Px(f64),
}
/// Source block framing.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CodeBlockContainer {
    #[default]
    /// Paint a bordered, rounded source card.
    Card,
    /// Render source without the outer card frame.
    Section,
}
/// Read-only reactive appearance of a standard code block.
#[derive(Clone, Debug, PartialEq)]
pub struct CodeAppearance {
    /// Reactive source typography size.
    pub size: Value<CodeBlockSize>,
    /// Reactive outer width policy.
    pub width: Value<CodeBlockWidth>,
    /// Reactive card or frameless treatment.
    pub container: Value<CodeBlockContainer>,
}
impl Default for CodeAppearance {
    fn default() -> Self {
        Self {
            size: CodeBlockSize::Md.into_value(),
            width: CodeBlockWidth::Fit.into_value(),
            container: CodeBlockContainer::Card.into_value(),
        }
    }
}
impl kivact::ui::Variant for CodeAppearance {}
impl kivact::ui::Variant for CodeBlockSize {}
impl kivact::ui::Variant for CodeBlockWidth {}
impl kivact::ui::Variant for CodeBlockContainer {}
kivact::state::constant_value!(CodeBlockSize, CodeBlockWidth, CodeBlockContainer);
pub(crate) fn code(r: CodeBlockRecipes) -> CodeBlockRecipes {
    r.root(|_| root)
        .header(|_| header)
        .code(|_| source)
        .gutter(|_| gutter)
}
fn root(t: &Theme, s: &Look<CodeBlockState>) -> Style {
    let p = t.ext::<Tokens>();
    let width = s
        .variant::<CodeAppearance>()
        .map(|a| a.width.get())
        .or_else(|| s.variant::<CodeBlockWidth>().copied())
        .unwrap_or_default();
    let container = s
        .variant::<CodeAppearance>()
        .map(|a| a.container.get())
        .or_else(|| s.variant::<CodeBlockContainer>().copied())
        .unwrap_or_default();
    let look = style().direction(Direction::Column).min_width(0);
    let look = match width {
        CodeBlockWidth::Fit => look.max_width(pct(100)),
        CodeBlockWidth::Fill => look.width(pct(100)),
        CodeBlockWidth::Px(w) => look.width(w),
    };
    if container == CodeBlockContainer::Card {
        look.background(p.colors.syntax.background)
            .border(p.sizes.border_width, p.colors.border)
            .radius(p.radius.element)
            .clip(true)
    } else {
        look
    }
}
fn header(t: &Theme, _: &Look<CodeBlockState>) -> Style {
    let p = t.ext::<Tokens>();
    style()
        .justify_content(Justify::SpaceBetween)
        .padding_x(p.spacing.s4)
        .padding_y(p.spacing.s2)
        .gap(p.spacing.s2)
        .font_family(p.typography.family_code)
        .font_size(12)
        .color(p.colors.text_secondary)
}
fn source(t: &Theme, s: &Look<CodeBlockState>) -> Style {
    let p = t.ext::<Tokens>();
    let size = s
        .variant::<CodeAppearance>()
        .map(|a| a.size.get())
        .or_else(|| s.variant::<CodeBlockSize>().copied())
        .unwrap_or_default();
    style()
        .font_family(p.typography.family_code)
        .font_size(if size == CodeBlockSize::Sm { 12 } else { 14 })
        .color(p.colors.text_primary)
}
fn gutter(t: &Theme, s: &Look<CodeBlockState>) -> Style {
    source(t, s).color(t.ext::<Tokens>().colors.text_secondary)
}
