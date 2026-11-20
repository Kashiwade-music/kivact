//! Role: Style layers and their origins, kept for the DevTools style inspector.

use std::panic::Location;
use std::rc::Rc;

use kivact_style::Style;

/// One style layer: a constant or reactive style (`.style(..)`). Signals
/// read inside, the theme from `use_theme()` included, are tracked by
/// the node's style effect.
#[derive(Clone)]
pub(crate) struct StyleLayer {
    pub(crate) f: LayerFn,
    #[allow(
        dead_code,
        reason = "DevTools reads the layer's call site once the styles hook lands"
    )]
    pub(crate) at: Option<&'static Location<'static>>,
    pub(crate) origin: LayerOrigin,
}

/// What a style layer computes from.
#[derive(Clone)]
pub(crate) enum LayerFn {
    /// A constant or reactive style (`.style(..)`).
    Plain(Rc<dyn Fn() -> Style>),
    /// A recipe that reads the node's final variants.
    Variants(Rc<dyn Fn(&crate::view::Variants) -> Style>),
}

impl LayerFn {
    pub(crate) fn style(&self, variants: &crate::view::Variants) -> Style {
        match self {
            LayerFn::Plain(f) => f(),
            LayerFn::Variants(f) => f(variants),
        }
    }
}

/// Where one style layer came from.
#[derive(Clone)]
pub(crate) enum LayerOrigin {
    /// A builder's own base style.
    Base,
    /// A kit recipe's base style.
    BaseRecipe(#[allow(dead_code, reason = "DevTools reads the recipe name")] RecipeName),
    /// A kit recipe's style.
    Recipe(#[allow(dead_code, reason = "DevTools reads the recipe name")] RecipeName),
    /// A caller's `.style(..)`.
    Caller,
}

/// The name of a kit recipe, for DevTools; call it only to show it.
#[doc(hidden)]
// xtask-allow(api-shape): hidden internal contract - kit recipes tag their style layers with a recipe name for DevTools
pub type RecipeName = Rc<dyn Fn() -> &'static str>;
