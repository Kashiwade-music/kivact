//! Role: Public Lyra Neutral color tokens read by every component style.

use kivact::style::{Color, Theme, ThemeExtension, ThemeMode, rgb, rgba};

/// Typed tokens corresponding to the shadcn/ui CSS variables of the Neutral base color.
///
/// [`theme`](crate::theme) installs them in the [`Theme`]; components read them
/// with `theme.get::<Tokens>()` inside their recipes and style closures, or
/// name a color with [`Color`](crate::Color), so a live theme switch restyles
/// mounted components. A theme without them reads the values of its
/// [`ThemeMode`].
#[derive(Clone, Debug, PartialEq)]
pub struct Tokens {
    /// The scheme these values belong to; `dark:` variants follow it.
    pub mode: ThemeMode,
    /// `--background`: the page.
    pub background: Color,
    /// `--foreground`: text on the page.
    pub foreground: Color,
    /// `--card`: a card's surface.
    pub card: Color,
    /// `--card-foreground`: text on a card.
    pub card_foreground: Color,
    /// `--popover`: menus, dialogs and other floating surfaces.
    pub popover: Color,
    /// `--popover-foreground`: text on a popover.
    pub popover_foreground: Color,
    /// `--primary`: the default button and checked controls.
    pub primary: Color,
    /// `--primary-foreground`: text on primary.
    pub primary_foreground: Color,
    /// `--secondary`: the secondary button.
    pub secondary: Color,
    /// `--secondary-foreground`: text on secondary.
    pub secondary_foreground: Color,
    /// `--muted`: quiet fills such as tab lists, skeletons and kbd.
    pub muted: Color,
    /// `--muted-foreground`: secondary text.
    pub muted_foreground: Color,
    /// `--accent`: the focused menu or select row.
    pub accent: Color,
    /// `--accent-foreground`: text on accent.
    pub accent_foreground: Color,
    /// `--destructive`: destructive actions and errors.
    pub destructive: Color,
    /// `--border`: separators and outlines.
    pub border: Color,
    /// `--input`: the borders of fields and unchecked controls.
    pub input: Color,
    /// `--ring`: the focus indication.
    pub ring: Color,
    /// `--chart-1` to `--chart-5`.
    pub chart: [Color; 5],
    /// `--sidebar`: the sidebar surface.
    pub sidebar: Color,
    /// `--sidebar-foreground`.
    pub sidebar_foreground: Color,
    /// `--sidebar-primary`.
    pub sidebar_primary: Color,
    /// `--sidebar-primary-foreground`.
    pub sidebar_primary_foreground: Color,
    /// `--sidebar-accent`.
    pub sidebar_accent: Color,
    /// `--sidebar-accent-foreground`.
    pub sidebar_accent_foreground: Color,
    /// `--sidebar-border`.
    pub sidebar_border: Color,
    /// `--sidebar-ring`.
    pub sidebar_ring: Color,
    /// The syntax colors of [`code_block`](crate::code_block).
    pub code: CodeColors,
}

/// The syntax colors of a code block, one per highlighted tone.
///
/// The defaults are GitHub's light and dark themes, the ones shadcn/ui's
/// documentation highlights its code blocks with; plain text and
/// punctuation keep the block's foreground.
#[derive(Clone, Debug, PartialEq)]
pub struct CodeColors {
    /// Keywords (`fn`, `let`, `pub`).
    pub keyword: Color,
    /// String and character literals.
    pub string: Color,
    /// Numeric literals.
    pub number: Color,
    /// Comments, line and block.
    pub comment: Color,
    /// Called or defined functions.
    pub function: Color,
    /// Type names.
    pub type_name: Color,
    /// Macro invocations (`println!`).
    pub macro_call: Color,
    /// Lifetimes and labels (`'a`).
    pub lifetime: Color,
    /// Attributes (`#[derive(..)]`).
    pub attribute: Color,
}

impl CodeColors {
    /// GitHub's light or dark syntax colors.
    pub fn new(mode: ThemeMode) -> Self {
        let dark = mode == ThemeMode::Dark;
        let pick = |light: u32, dark_value: u32| rgb(if dark { dark_value } else { light });
        Self {
            keyword: pick(0xcf222e, 0xff7b72),
            string: pick(0x0a3069, 0xa5d6ff),
            number: pick(0x0550ae, 0x79c0ff),
            comment: pick(0x6e7781, 0x8b949e),
            function: pick(0x8250df, 0xd2a8ff),
            type_name: pick(0x953800, 0xffa657),
            macro_call: pick(0x0550ae, 0x79c0ff),
            lifetime: pick(0x116329, 0x7ee787),
            attribute: pick(0x6639ba, 0xbc8cff),
        }
    }
}

impl Tokens {
    /// The Tailwind v4 Neutral values of the light or dark scheme.
    pub fn new(mode: ThemeMode) -> Self {
        let dark = mode == ThemeMode::Dark;
        let pick = |light: u32, dark_value: u32| rgb(if dark { dark_value } else { light });
        Self {
            mode,
            background: pick(0xffffff, 0x0a0a0a),
            foreground: pick(0x0a0a0a, 0xfafafa),
            card: pick(0xffffff, 0x171717),
            card_foreground: pick(0x0a0a0a, 0xfafafa),
            popover: pick(0xffffff, 0x171717),
            popover_foreground: pick(0x0a0a0a, 0xfafafa),
            primary: pick(0x171717, 0xe5e5e5),
            primary_foreground: pick(0xfafafa, 0x171717),
            secondary: pick(0xf5f5f5, 0x262626),
            secondary_foreground: pick(0x171717, 0xfafafa),
            muted: pick(0xf5f5f5, 0x262626),
            muted_foreground: pick(0x737373, 0xa1a1a1),
            accent: pick(0xf5f5f5, 0x262626),
            accent_foreground: pick(0x171717, 0xfafafa),
            destructive: pick(0xe7000b, 0xff6467),
            border: if dark {
                rgba(0xffffff, 0.10)
            } else {
                rgb(0xe5e5e5)
            },
            input: if dark {
                rgba(0xffffff, 0.15)
            } else {
                rgb(0xe5e5e5)
            },
            ring: pick(0xa1a1a1, 0x737373),
            chart: [
                rgb(0xd4d4d4),
                rgb(0x737373),
                rgb(0x525252),
                rgb(0x404040),
                rgb(0x262626),
            ],
            sidebar: pick(0xfafafa, 0x171717),
            sidebar_foreground: pick(0x0a0a0a, 0xfafafa),
            sidebar_primary: pick(0x171717, 0x1447e6),
            sidebar_primary_foreground: rgb(0xfafafa),
            sidebar_accent: pick(0xf5f5f5, 0x262626),
            sidebar_accent_foreground: pick(0x171717, 0xfafafa),
            sidebar_border: if dark {
                rgba(0xffffff, 0.10)
            } else {
                rgb(0xe5e5e5)
            },
            sidebar_ring: pick(0xa1a1a1, 0x737373),
            code: CodeColors::new(mode),
        }
    }

    /// Whether `dark:` variants apply.
    pub fn is_dark(&self) -> bool {
        self.mode == ThemeMode::Dark
    }
}

impl ThemeExtension for Tokens {
    fn fallback(theme: &Theme) -> Self {
        Self::new(theme.mode())
    }
}
