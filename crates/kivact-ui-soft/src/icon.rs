//! Role: Render themed icons with Neutral sizing and optional color.

use kivact::state::IntoText;
use kivact::state::IntoValue;
use kivact::style::Icon as IconSource;
use kivact::style::Style;
use kivact::style::style;
use kivact::view::Element;
use kivact::view::IntoView;
use kivact::view::Node;
use kivact::view::View;
use kivact::view::icon as control_icon;
use std::cell::Cell;
use std::rc::Rc;

/// The icon size option.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum IconSize {
    /// Use the extra-small icon dimensions from the theme.
    Xs,
    /// Use the small icon dimensions from the theme.
    Sm,
    #[default]
    /// Use the standard icon dimensions from the theme.
    Md,
    /// Use the large icon dimensions from the theme.
    Lg,
}

/// The icon color option.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum IconColor {
    /// Use primary icon ink.
    Primary,
    /// Use secondary icon ink.
    Secondary,
    /// Use the secondary ink for a tertiary treatment.
    Tertiary,
    /// Use disabled icon ink.
    Disabled,
    /// Use accent icon ink.
    Accent,
    /// Use informational text ink.
    Info,
    /// Use successful text ink.
    Success,
    /// Use error text ink.
    Error,
    /// Use warning text ink.
    Warning,
    #[default]
    /// Inherit foreground ink from the surrounding content.
    Inherit,
    /// Use the blue categorical icon ink.
    Blue,
    /// Use the red categorical icon ink.
    Red,
    /// Use the green categorical icon ink.
    Green,
    /// Use the gray categorical icon ink.
    Gray,
    /// Use the cyan categorical icon ink.
    Cyan,
    /// Use the teal categorical icon ink.
    Teal,
    /// Use the yellow categorical icon ink.
    Yellow,
    /// Use the orange categorical icon ink.
    Orange,
    /// Use the pink categorical icon ink.
    Pink,
    /// Use the purple categorical icon ink.
    Purple,
}

/// A theme icon created with [`icon_view`].
pub struct IconView {
    root: Node,
    options: Rc<Cell<(IconSize, IconColor)>>,
}

/// Show an icon from the current theme.
#[track_caller]
pub fn icon_view(source: impl IntoValue<IconSource>) -> IconView {
    let options = Rc::new(Cell::new((IconSize::default(), IconColor::default())));
    let look = options.clone();
    // Override the primitive's ordinary style layer before callers add theirs.
    let root = control_icon(source).style(move || {
        let (size, color) = look.get();
        icon_style(size, color)
    });
    IconView { root, options }
}

impl IconView {
    /// Set the size.
    pub fn size(self, value: IconSize) -> Self {
        self.options.set((value, self.options.get().1));
        self
    }

    /// Set the color.
    pub fn color(self, value: IconColor) -> Self {
        self.options.set((self.options.get().0, value));
        self
    }

    /// Give a meaningful icon an accessible name.
    pub fn label(self, label: impl IntoText) -> Self {
        self.name(label)
    }
}

impl Element for IconView {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for IconView {
    fn into_view(self) -> View {
        self.root.into_view()
    }
}

/// Resolve the Neutral icon size and color against the current theme.
pub fn icon_style(size: IconSize, color: IconColor) -> Style {
    let tokens = kivact::style::use_tokens::<crate::Tokens>();
    let s = &tokens.sizes;
    let size = match size {
        IconSize::Xs => s.icon_xsm,
        IconSize::Sm => s.icon_sm,
        IconSize::Md => s.icon_md,
        IconSize::Lg => s.icon_lg,
    };
    let look = style().size(size, size).shrink(0);
    let p = &tokens.colors;
    match color {
        IconColor::Primary => look.color(p.icon_primary),
        IconColor::Secondary | IconColor::Tertiary => look.color(p.icon_secondary),
        IconColor::Disabled => look.color(p.icon_disabled),
        IconColor::Accent => look.color(p.icon_accent),
        IconColor::Info => look.color(p.text_info),
        IconColor::Success => look.color(p.text_success),
        IconColor::Error => look.color(p.text_error),
        IconColor::Warning => look.color(p.text_warning),
        IconColor::Inherit => look,
        IconColor::Blue => look.color(p.hue.blue.icon),
        IconColor::Red => look.color(p.hue.red.icon),
        IconColor::Green => look.color(p.hue.green.icon),
        IconColor::Gray => look.color(p.hue.gray.icon),
        IconColor::Cyan => look.color(p.hue.cyan.icon),
        IconColor::Teal => look.color(p.hue.teal.icon),
        IconColor::Yellow => look.color(p.hue.yellow.icon),
        IconColor::Orange => look.color(p.hue.orange.icon),
        IconColor::Pink => look.color(p.hue.pink.icon),
        IconColor::Purple => look.color(p.hue.purple.icon),
    }
}
