//! Role: Render themed icons with Neutral sizing and optional color.
use crate::tokens;
use kivact::control::{Node, NodeBuilder, icon as control_icon};
use kivact::style::{Icon as IconSource, Style, style};
use kivact::view::{IntoText, IntoValue, IntoView, View};
use std::cell::Cell;
use std::rc::Rc;

/// The icon size option.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum IconSize {
    Xs,
    Sm,
    #[default]
    Md,
    Lg,
}

/// The icon color option.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum IconColor {
    Primary,
    Secondary,
    Tertiary,
    Disabled,
    Accent,
    Info,
    Success,
    Error,
    Warning,
    #[default]
    Inherit,
    Blue,
    Red,
    Green,
    Gray,
    Cyan,
    Teal,
    Yellow,
    Orange,
    Pink,
    Purple,
}

/// A theme icon created with [`icon`].
pub struct Icon {
    root: Node,
    options: Rc<Cell<(IconSize, IconColor)>>,
}

/// Show an icon from the current theme.
#[track_caller]
pub fn icon(source: impl IntoValue<IconSource>) -> Icon {
    let options = Rc::new(Cell::new((IconSize::default(), IconColor::default())));
    let look = options.clone();
    // Override the primitive's ordinary style layer before callers add theirs.
    let root = control_icon(source).style(move || {
        let (size, color) = look.get();
        icon_style(size, color)
    });
    Icon { root, options }
}

impl Icon {
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

impl NodeBuilder for Icon {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for Icon {
    fn into_view(self) -> View {
        self.root.into_view()
    }
}

/// Resolve the Neutral icon size and color against the current theme.
pub fn icon_style(size: IconSize, color: IconColor) -> Style {
    let tokens = tokens();
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
