//! Role: The game kit's color theme variables and the values they take in each mode.
use kivact::style::{Color, Theme, ThemeMode, ThemeVar, color, rgb, rgba};

/// `--primary-hover`: the hover fill of the primary button.
pub static PRIMARY_HOVER_VAR: ThemeVar<Color> = ThemeVar::new("game.primary_hover", rgb(0x2b93f5));
/// `--primary-strong`: the shadow under the primary button.
pub static PRIMARY_STRONG_VAR: ThemeVar<Color> =
    ThemeVar::new("game.primary_strong", rgb(0x0a62c2));
/// `--primary-weak`: the selection and hover tint; the theme's selection by default.
pub static PRIMARY_WEAK_VAR: ThemeVar<Color> =
    ThemeVar::alias("game.primary_weak", &color::SELECTION_VAR);
/// `--success-weak`: the tint behind a success badge.
pub static SUCCESS_WEAK_VAR: ThemeVar<Color> = ThemeVar::new("game.success_weak", rgb(0xddf5eb));
/// `--warning-weak`: the tint behind a warning badge.
pub static WARNING_WEAK_VAR: ThemeVar<Color> = ThemeVar::new("game.warning_weak", rgb(0xfff1d6));
/// `--danger-weak`: the tint behind a danger badge.
pub static DANGER_WEAK_VAR: ThemeVar<Color> = ThemeVar::new("game.danger_weak", rgb(0xfde4e8));
/// `--on-dark`: text and marks on the navy panels.
pub static ON_DARK_VAR: ThemeVar<Color> = ThemeVar::new("game.on_dark", rgb(0xffffff));
/// `--dark-shadow`: the hard shadow of the navy button.
pub static DARK_SHADOW_VAR: ThemeVar<Color> = ThemeVar::new("game.dark_shadow", rgb(0x000000));
/// `--toast-text`: the body text of a toast.
pub static TOAST_TEXT_VAR: ThemeVar<Color> = ThemeVar::new("game.toast_text", rgb(0xb8c9dc));
/// `--tape-stripe`: the stripes of the banner's tape.
pub static TAPE_STRIPE_VAR: ThemeVar<Color> = ThemeVar::new("game.tape_stripe", rgb(0x1a1a1a));
/// `--dropzone-stripe`: the faint stripes of a drop target.
pub static DROPZONE_STRIPE_VAR: ThemeVar<Color> =
    ThemeVar::new("game.dropzone_stripe", rgba(0x1283f0, 0.04));
/// `--cyan`: the accent of the toast and the navy panels.
pub static CYAN_VAR: ThemeVar<Color> = ThemeVar::new("game.cyan", rgb(0x2ec4f0));
/// `--yellow`: the focus ring and the banner's warning fill; the theme's focus by default.
pub static YELLOW_VAR: ThemeVar<Color> = ThemeVar::alias("game.yellow", &color::FOCUS_VAR);
/// `--navy`: the console's dark panel.
pub static NAVY_VAR: ThemeVar<Color> = ThemeVar::new("game.navy", rgb(0x0c1f36));
/// `--canvas`: the page behind the panels.
pub static CANVAS_VAR: ThemeVar<Color> = ThemeVar::new("game.canvas", rgb(0xedf2f7));
/// `--surface-3`: the third surface: wells, the pressed row and the disabled field.
pub static SURFACE_3_VAR: ThemeVar<Color> = ThemeVar::alias("game.surface_3", &color::PRESSED_VAR);
/// `--line-strong`: the strong rule of a field, a panel and a card.
pub static LINE_STRONG_VAR: ThemeVar<Color> = ThemeVar::new("game.line_strong", rgb(0xafc0d2));
/// `--text-2`: the secondary text; the muted text by default.
pub static TEXT_2_VAR: ThemeVar<Color> = ThemeVar::alias("game.text_2", &color::TEXT_MUTED_VAR);
/// `--text-3`: the tertiary text.
pub static TEXT_3_VAR: ThemeVar<Color> = ThemeVar::new("game.text_3", rgb(0x8093a9));
/// `--sidebar`: the navigation panel's fill.
pub static SIDEBAR_VAR: ThemeVar<Color> = ThemeVar::new("game.sidebar", rgb(0x0c1f36));
/// `--sidebar-alt`: the navigation panel's hover fill.
pub static SIDEBAR_ALT_VAR: ThemeVar<Color> = ThemeVar::new("game.sidebar_alt", rgb(0x11294a));
/// `--sidebar-text`: the navigation panel's text.
pub static SIDEBAR_TEXT_VAR: ThemeVar<Color> = ThemeVar::new("game.sidebar_text", rgb(0xcfdceb));
/// `--sidebar-dim`: the navigation panel's quiet text.
pub static SIDEBAR_DIM_VAR: ThemeVar<Color> = ThemeVar::new("game.sidebar_dim", rgb(0x7890ac));

/// The kit's colors as values a style or a draw call takes; each follows the theme it is painted in.
pub const PRIMARY_HOVER: Color = Color::var(&PRIMARY_HOVER_VAR);
/// See [`PRIMARY_HOVER_VAR`].
pub const PRIMARY_STRONG: Color = Color::var(&PRIMARY_STRONG_VAR);
/// See [`PRIMARY_HOVER_VAR`].
pub const PRIMARY_WEAK: Color = Color::var(&PRIMARY_WEAK_VAR);
/// See [`PRIMARY_HOVER_VAR`].
pub const SUCCESS_WEAK: Color = Color::var(&SUCCESS_WEAK_VAR);
/// See [`PRIMARY_HOVER_VAR`].
pub const WARNING_WEAK: Color = Color::var(&WARNING_WEAK_VAR);
/// See [`PRIMARY_HOVER_VAR`].
pub const DANGER_WEAK: Color = Color::var(&DANGER_WEAK_VAR);
/// See [`PRIMARY_HOVER_VAR`].
pub const ON_DARK: Color = Color::var(&ON_DARK_VAR);
/// See [`PRIMARY_HOVER_VAR`].
pub const DARK_SHADOW: Color = Color::var(&DARK_SHADOW_VAR);
/// See [`PRIMARY_HOVER_VAR`].
pub const TOAST_TEXT: Color = Color::var(&TOAST_TEXT_VAR);
/// See [`PRIMARY_HOVER_VAR`].
pub const TAPE_STRIPE: Color = Color::var(&TAPE_STRIPE_VAR);
/// See [`PRIMARY_HOVER_VAR`].
pub const DROPZONE_STRIPE: Color = Color::var(&DROPZONE_STRIPE_VAR);
/// See [`PRIMARY_HOVER_VAR`].
pub const CYAN: Color = Color::var(&CYAN_VAR);
/// See [`PRIMARY_HOVER_VAR`].
pub const YELLOW: Color = Color::var(&YELLOW_VAR);
/// See [`PRIMARY_HOVER_VAR`].
pub const NAVY: Color = Color::var(&NAVY_VAR);
/// See [`PRIMARY_HOVER_VAR`].
pub const CANVAS: Color = Color::var(&CANVAS_VAR);
/// See [`PRIMARY_HOVER_VAR`].
pub const SURFACE_3: Color = Color::var(&SURFACE_3_VAR);
/// See [`PRIMARY_HOVER_VAR`].
pub const LINE_STRONG: Color = Color::var(&LINE_STRONG_VAR);
/// See [`PRIMARY_HOVER_VAR`].
pub const TEXT_2: Color = Color::var(&TEXT_2_VAR);
/// See [`PRIMARY_HOVER_VAR`].
pub const TEXT_3: Color = Color::var(&TEXT_3_VAR);
/// See [`PRIMARY_HOVER_VAR`].
pub const SIDEBAR: Color = Color::var(&SIDEBAR_VAR);
/// See [`PRIMARY_HOVER_VAR`].
pub const SIDEBAR_ALT: Color = Color::var(&SIDEBAR_ALT_VAR);
/// See [`PRIMARY_HOVER_VAR`].
pub const SIDEBAR_TEXT: Color = Color::var(&SIDEBAR_TEXT_VAR);
/// See [`PRIMARY_HOVER_VAR`].
pub const SIDEBAR_DIM: Color = Color::var(&SIDEBAR_DIM_VAR);

/// Set every color variable of the kit to its value in `mode`.
pub(crate) fn set_colors(theme: Theme, mode: ThemeMode) -> Theme {
    let dark = mode == ThemeMode::Dark;
    let pick = |light: u32, dark_value: u32| rgb(if dark { dark_value } else { light });
    theme
        .set(&PRIMARY_HOVER_VAR, pick(0x2b93f5, 0x469ff5))
        .set(&PRIMARY_STRONG_VAR, pick(0x0a62c2, 0x1a6cc4))
        .set(
            &PRIMARY_WEAK_VAR,
            if dark {
                rgba(0x3a9dff, 0.15)
            } else {
                rgb(0xe2f0ff)
            },
        )
        .set(
            &SUCCESS_WEAK_VAR,
            if dark {
                rgba(0x2cc48a, 0.14)
            } else {
                rgb(0xddf5eb)
            },
        )
        .set(
            &WARNING_WEAK_VAR,
            if dark {
                rgba(0xf5a524, 0.14)
            } else {
                rgb(0xfff1d6)
            },
        )
        .set(
            &DANGER_WEAK_VAR,
            if dark {
                rgba(0xff5a6e, 0.14)
            } else {
                rgb(0xfde4e8)
            },
        )
        .set(&YELLOW_VAR, rgb(0xffc928))
        .set(&NAVY_VAR, pick(0x0c1f36, 0x1a3150))
        .set(&CANVAS_VAR, pick(0xedf2f7, 0x08111c))
        .set(&SURFACE_3_VAR, pick(0xeaf0f6, 0x192d45))
        .set(&LINE_STRONG_VAR, pick(0xafc0d2, 0x33506f))
        .set(&TEXT_2_VAR, pick(0x4b5e76, 0x9db1c8))
        .set(&TEXT_3_VAR, pick(0x8093a9, 0x6880a0))
        .set(&SIDEBAR_VAR, pick(0x0c1f36, 0x050d17))
        .set(&SIDEBAR_ALT_VAR, pick(0x11294a, 0x0b1a2c))
}
