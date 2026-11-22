//! Role: The Astryx Neutral colors: a theme variable for each color of the kit,
//! set for each appearance by `set_colors`.
use kivact::style::{Color, Theme, ThemeMode, ThemeVar, rgb, rgba};

pub(crate) static BG_BODY_VAR: ThemeVar<Color> = ThemeVar::new("soft.bg_body", rgb(0xf1f1f1));
pub const BG_BODY: Color = Color::var(&BG_BODY_VAR);

pub(crate) static BG_SURFACE_VAR: ThemeVar<Color> =
    ThemeVar::alias("soft.bg_surface", &kivact::style::color::SURFACE_VAR);
pub const BG_SURFACE: Color = Color::var(&BG_SURFACE_VAR);

pub(crate) static BG_CARD_VAR: ThemeVar<Color> = ThemeVar::new("soft.bg_card", rgb(0xffffff));
pub const BG_CARD: Color = Color::var(&BG_CARD_VAR);

pub(crate) static BG_POPOVER_VAR: ThemeVar<Color> =
    ThemeVar::alias("soft.bg_popover", &kivact::style::color::RAISED_VAR);
pub const BG_POPOVER: Color = Color::var(&BG_POPOVER_VAR);

pub(crate) static BG_MUTED_VAR: ThemeVar<Color> =
    ThemeVar::alias("soft.bg_muted", &kivact::style::color::SURFACE_ALT_VAR);
pub const BG_MUTED: Color = Color::var(&BG_MUTED_VAR);

pub(crate) static BG_INVERTED_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.bg_inverted", rgb(0x0a1317));
pub const BG_INVERTED: Color = Color::var(&BG_INVERTED_VAR);

pub(crate) static BG_ERROR_INVERTED_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.bg_error_inverted", rgb(0xaa071e));
pub const BG_ERROR_INVERTED: Color = Color::var(&BG_ERROR_INVERTED_VAR);

pub(crate) static NEUTRAL_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.neutral", rgba(0x000000, 15.0 / 255.0));
pub const NEUTRAL: Color = Color::var(&NEUTRAL_VAR);

pub(crate) static ACCENT_VAR: ThemeVar<Color> =
    ThemeVar::alias("soft.accent", &kivact::style::color::ACCENT_VAR);
pub const ACCENT: Color = Color::var(&ACCENT_VAR);

pub(crate) static ACCENT_HOVER_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.accent_hover", rgb(0x171717));
pub const ACCENT_HOVER: Color = Color::var(&ACCENT_HOVER_VAR);

pub(crate) static CHOICE_BORDER_HOVER_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.choice_border_hover", rgb(0xaaaaaa));
pub const CHOICE_BORDER_HOVER: Color = Color::var(&CHOICE_BORDER_HOVER_VAR);

pub(crate) static CHOICE_SURFACE_HOVER_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.choice_surface_hover", rgb(0xf2f2f2));
pub const CHOICE_SURFACE_HOVER: Color = Color::var(&CHOICE_SURFACE_HOVER_VAR);

pub(crate) static SWITCH_OFF_HOVER_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.switch_off_hover", rgb(0xc9c9c9));
pub const SWITCH_OFF_HOVER: Color = Color::var(&SWITCH_OFF_HOVER_VAR);

pub(crate) static ACCENT_MUTED_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.accent_muted", rgb(0xf1f1f1));
pub const ACCENT_MUTED: Color = Color::var(&ACCENT_MUTED_VAR);

pub(crate) static ON_ACCENT_VAR: ThemeVar<Color> =
    ThemeVar::alias("soft.on_accent", &kivact::style::color::ACCENT_TEXT_VAR);
pub const ON_ACCENT: Color = Color::var(&ON_ACCENT_VAR);

pub(crate) static ON_DARK_VAR: ThemeVar<Color> = ThemeVar::new("soft.on_dark", rgb(0xffffff));
pub const ON_DARK: Color = Color::var(&ON_DARK_VAR);

pub(crate) static WHITE_VAR: ThemeVar<Color> = ThemeVar::new("soft.white", rgb(0xffffff));

pub(crate) static ON_LIGHT_VAR: ThemeVar<Color> = ThemeVar::new("soft.on_light", rgb(0x111111));
pub const ON_LIGHT: Color = Color::var(&ON_LIGHT_VAR);

pub(crate) static TEXT_PRIMARY_VAR: ThemeVar<Color> =
    ThemeVar::alias("soft.text_primary", &kivact::style::color::TEXT_VAR);
pub const TEXT_PRIMARY: Color = Color::var(&TEXT_PRIMARY_VAR);

pub(crate) static TEXT_SECONDARY_VAR: ThemeVar<Color> =
    ThemeVar::alias("soft.text_secondary", &kivact::style::color::TEXT_MUTED_VAR);
pub const TEXT_SECONDARY: Color = Color::var(&TEXT_SECONDARY_VAR);

pub(crate) static TEXT_DISABLED_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.text_disabled", rgb(0x919191));
pub const TEXT_DISABLED: Color = Color::var(&TEXT_DISABLED_VAR);

pub(crate) static TEXT_ACCENT_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.text_accent", rgb(0x1b1b1b));
pub const TEXT_ACCENT: Color = Color::var(&TEXT_ACCENT_VAR);

pub(crate) static ICON_PRIMARY_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.icon_primary", rgb(0x000000));
pub const ICON_PRIMARY: Color = Color::var(&ICON_PRIMARY_VAR);

pub(crate) static ICON_SECONDARY_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.icon_secondary", rgb(0x6a6a6a));
pub const ICON_SECONDARY: Color = Color::var(&ICON_SECONDARY_VAR);

pub(crate) static ICON_DISABLED_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.icon_disabled", rgb(0x919191));
pub const ICON_DISABLED: Color = Color::var(&ICON_DISABLED_VAR);

pub(crate) static ICON_ACCENT_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.icon_accent", rgb(0x1b1b1b));
pub const ICON_ACCENT: Color = Color::var(&ICON_ACCENT_VAR);

pub(crate) static BORDER_VAR: ThemeVar<Color> =
    ThemeVar::alias("soft.border", &kivact::style::color::BORDER_VAR);
pub(crate) const BORDER: Color = Color::var(&BORDER_VAR);

pub(crate) static BORDER_EMPHASIZED_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.border_emphasized", rgb(0xd4d4d4));
pub const BORDER_EMPHASIZED: Color = Color::var(&BORDER_EMPHASIZED_VAR);

pub(crate) static SKELETON_VAR: ThemeVar<Color> = ThemeVar::new("soft.skeleton", rgb(0xf1f1f1));
pub const SKELETON: Color = Color::var(&SKELETON_VAR);

pub(crate) static TRACK_VAR: ThemeVar<Color> = ThemeVar::new("soft.track", rgb(0xccd3db));
pub const TRACK: Color = Color::var(&TRACK_VAR);

pub(crate) static SHADOW_VAR: ThemeVar<Color> =
    ThemeVar::alias("soft.shadow", &kivact::style::color::SHADOW_VAR);
pub const SHADOW: Color = Color::var(&SHADOW_VAR);

pub(crate) static TINT_HOVER_VAR: ThemeVar<Color> = ThemeVar::new("soft.tint_hover", rgb(0x000000));
pub const TINT_HOVER: Color = Color::var(&TINT_HOVER_VAR);

pub(crate) static OVERLAY_SCRIM_VAR: ThemeVar<Color> =
    ThemeVar::alias("soft.overlay_scrim", &kivact::style::color::OVERLAY_VAR);
pub const OVERLAY_SCRIM: Color = Color::var(&OVERLAY_SCRIM_VAR);

pub(crate) static OVERLAY_LIGHT_SCRIM_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.overlay_light_scrim", rgba(0xffffff, 0.6));
pub const OVERLAY_LIGHT_SCRIM: Color = Color::var(&OVERLAY_LIGHT_SCRIM_VAR);

pub(crate) static OVERLAY_HOVER_VAR: ThemeVar<Color> =
    ThemeVar::alias("soft.overlay_hover", &kivact::style::color::HOVER_VAR);
pub const OVERLAY_HOVER: Color = Color::var(&OVERLAY_HOVER_VAR);

pub(crate) static OVERLAY_PRESSED_VAR: ThemeVar<Color> =
    ThemeVar::alias("soft.overlay_pressed", &kivact::style::color::PRESSED_VAR);
pub const OVERLAY_PRESSED: Color = Color::var(&OVERLAY_PRESSED_VAR);

pub(crate) static FOCUS_RING_VAR: ThemeVar<Color> =
    ThemeVar::alias("soft.focus_ring", &kivact::style::color::FOCUS_VAR);
pub(crate) const FOCUS_RING: Color = Color::var(&FOCUS_RING_VAR);

pub(crate) static INFO_VAR: ThemeVar<Color> = ThemeVar::new("soft.info", rgb(0x0074e2));
pub const INFO: Color = Color::var(&INFO_VAR);

pub(crate) static ON_INFO_VAR: ThemeVar<Color> = ThemeVar::new("soft.on_info", rgb(0xffffff));
pub const ON_INFO: Color = Color::var(&ON_INFO_VAR);

pub(crate) static INFO_MUTED_VAR: ThemeVar<Color> =
    ThemeVar::alias("soft.info_muted", &kivact::style::color::SELECTION_VAR);
pub const INFO_MUTED: Color = Color::var(&INFO_MUTED_VAR);

pub(crate) static ON_INFO_MUTED_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.on_info_muted", rgb(0x003978));
pub const ON_INFO_MUTED: Color = Color::var(&ON_INFO_MUTED_VAR);

pub(crate) static TEXT_INFO_VAR: ThemeVar<Color> = ThemeVar::new("soft.text_info", rgb(0x0067ce));
pub const TEXT_INFO: Color = Color::var(&TEXT_INFO_VAR);

pub(crate) static SUCCESS_VAR: ThemeVar<Color> =
    ThemeVar::alias("soft.success", &kivact::style::color::SUCCESS_VAR);
pub const SUCCESS: Color = Color::var(&SUCCESS_VAR);

pub(crate) static ON_SUCCESS_VAR: ThemeVar<Color> = ThemeVar::new("soft.on_success", rgb(0xffffff));
pub const ON_SUCCESS: Color = Color::var(&ON_SUCCESS_VAR);

pub(crate) static SUCCESS_MUTED_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.success_muted", rgb(0xbce0bb));
pub const SUCCESS_MUTED: Color = Color::var(&SUCCESS_MUTED_VAR);

pub(crate) static ON_SUCCESS_MUTED_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.on_success_muted", rgb(0x00490b));
pub const ON_SUCCESS_MUTED: Color = Color::var(&ON_SUCCESS_MUTED_VAR);

pub(crate) static TEXT_SUCCESS_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.text_success", rgb(0x2f7d33));
pub const TEXT_SUCCESS: Color = Color::var(&TEXT_SUCCESS_VAR);

pub(crate) static WARNING_VAR: ThemeVar<Color> =
    ThemeVar::alias("soft.warning", &kivact::style::color::WARNING_VAR);
pub const WARNING: Color = Color::var(&WARNING_VAR);

pub(crate) static ON_WARNING_VAR: ThemeVar<Color> = ThemeVar::new("soft.on_warning", rgb(0x111111));
pub const ON_WARNING: Color = Color::var(&ON_WARNING_VAR);

pub(crate) static WARNING_MUTED_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.warning_muted", rgb(0xfae19e));
pub const WARNING_MUTED: Color = Color::var(&WARNING_MUTED_VAR);

pub(crate) static ON_WARNING_MUTED_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.on_warning_muted", rgb(0x4b3900));
pub const ON_WARNING_MUTED: Color = Color::var(&ON_WARNING_MUTED_VAR);

pub(crate) static TEXT_WARNING_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.text_warning", rgb(0x836700));
pub const TEXT_WARNING: Color = Color::var(&TEXT_WARNING_VAR);

pub(crate) static ERROR_VAR: ThemeVar<Color> =
    ThemeVar::alias("soft.error", &kivact::style::color::DANGER_VAR);
pub const ERROR: Color = Color::var(&ERROR_VAR);

pub(crate) static ON_ERROR_VAR: ThemeVar<Color> = ThemeVar::new("soft.on_error", rgb(0xffffff));
pub const ON_ERROR: Color = Color::var(&ON_ERROR_VAR);

pub(crate) static ERROR_MUTED_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.error_muted", rgb(0xffc4be));
pub const ERROR_MUTED: Color = Color::var(&ERROR_MUTED_VAR);

pub(crate) static ON_ERROR_MUTED_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.on_error_muted", rgb(0x76000c));
pub const ON_ERROR_MUTED: Color = Color::var(&ON_ERROR_MUTED_VAR);

pub(crate) static TEXT_ERROR_VAR: ThemeVar<Color> = ThemeVar::new("soft.text_error", rgb(0xc0262c));
pub const TEXT_ERROR: Color = Color::var(&TEXT_ERROR_VAR);

pub(crate) static HUE_RED_BG_VAR: ThemeVar<Color> = ThemeVar::new("soft.hue_red_bg", rgb(0xffc4be));
pub const HUE_RED_BG: Color = Color::var(&HUE_RED_BG_VAR);

pub(crate) static HUE_RED_BORDER_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.hue_red_border", rgb(0xffaea7));
pub const HUE_RED_BORDER: Color = Color::var(&HUE_RED_BORDER_VAR);

pub(crate) static HUE_RED_ICON_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.hue_red_icon", rgb(0x76000c));
pub const HUE_RED_ICON: Color = Color::var(&HUE_RED_ICON_VAR);

pub(crate) static HUE_RED_TEXT_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.hue_red_text", rgb(0x76000c));
pub const HUE_RED_TEXT: Color = Color::var(&HUE_RED_TEXT_VAR);

pub(crate) static HUE_ORANGE_BG_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.hue_orange_bg", rgb(0xffc7a1));
pub const HUE_ORANGE_BG: Color = Color::var(&HUE_ORANGE_BG_VAR);

pub(crate) static HUE_ORANGE_BORDER_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.hue_orange_border", rgb(0xffc7a1));
pub const HUE_ORANGE_BORDER: Color = Color::var(&HUE_ORANGE_BORDER_VAR);

pub(crate) static HUE_ORANGE_ICON_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.hue_orange_icon", rgb(0x622700));
pub const HUE_ORANGE_ICON: Color = Color::var(&HUE_ORANGE_ICON_VAR);

pub(crate) static HUE_ORANGE_TEXT_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.hue_orange_text", rgb(0x622700));
pub const HUE_ORANGE_TEXT: Color = Color::var(&HUE_ORANGE_TEXT_VAR);

pub(crate) static HUE_YELLOW_BG_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.hue_yellow_bg", rgb(0xfae19e));
pub const HUE_YELLOW_BG: Color = Color::var(&HUE_YELLOW_BG_VAR);

pub(crate) static HUE_YELLOW_BORDER_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.hue_yellow_border", rgb(0xe3c36c));
pub const HUE_YELLOW_BORDER: Color = Color::var(&HUE_YELLOW_BORDER_VAR);

pub(crate) static HUE_YELLOW_ICON_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.hue_yellow_icon", rgb(0x4b3900));
pub const HUE_YELLOW_ICON: Color = Color::var(&HUE_YELLOW_ICON_VAR);

pub(crate) static HUE_YELLOW_TEXT_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.hue_yellow_text", rgb(0x4b3900));
pub const HUE_YELLOW_TEXT: Color = Color::var(&HUE_YELLOW_TEXT_VAR);

pub(crate) static HUE_GREEN_BG_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.hue_green_bg", rgb(0xbce0bb));
pub const HUE_GREEN_BG: Color = Color::var(&HUE_GREEN_BG_VAR);

pub(crate) static HUE_GREEN_BORDER_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.hue_green_border", rgb(0xaad4a9));
pub const HUE_GREEN_BORDER: Color = Color::var(&HUE_GREEN_BORDER_VAR);

pub(crate) static HUE_GREEN_ICON_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.hue_green_icon", rgb(0x00490b));
pub const HUE_GREEN_ICON: Color = Color::var(&HUE_GREEN_ICON_VAR);

pub(crate) static HUE_GREEN_TEXT_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.hue_green_text", rgb(0x00490b));
pub const HUE_GREEN_TEXT: Color = Color::var(&HUE_GREEN_TEXT_VAR);

pub(crate) static HUE_TEAL_BG_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.hue_teal_bg", rgb(0xa9e2d6));
pub const HUE_TEAL_BG: Color = Color::var(&HUE_TEAL_BG_VAR);

pub(crate) static HUE_TEAL_BORDER_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.hue_teal_border", rgb(0x90d7c8));
pub const HUE_TEAL_BORDER: Color = Color::var(&HUE_TEAL_BORDER_VAR);

pub(crate) static HUE_TEAL_ICON_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.hue_teal_icon", rgb(0x00463d));
pub const HUE_TEAL_ICON: Color = Color::var(&HUE_TEAL_ICON_VAR);

pub(crate) static HUE_TEAL_TEXT_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.hue_teal_text", rgb(0x00463d));
pub const HUE_TEAL_TEXT: Color = Color::var(&HUE_TEAL_TEXT_VAR);

pub(crate) static HUE_CYAN_BG_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.hue_cyan_bg", rgb(0x9ae2f4));
pub const HUE_CYAN_BG: Color = Color::var(&HUE_CYAN_BG_VAR);

pub(crate) static HUE_CYAN_BORDER_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.hue_cyan_border", rgb(0x85d5e9));
pub const HUE_CYAN_BORDER: Color = Color::var(&HUE_CYAN_BORDER_VAR);

pub(crate) static HUE_CYAN_ICON_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.hue_cyan_icon", rgb(0x004351));
pub const HUE_CYAN_ICON: Color = Color::var(&HUE_CYAN_ICON_VAR);

pub(crate) static HUE_CYAN_TEXT_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.hue_cyan_text", rgb(0x004351));
pub const HUE_CYAN_TEXT: Color = Color::var(&HUE_CYAN_TEXT_VAR);

pub(crate) static HUE_BLUE_BG_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.hue_blue_bg", rgb(0xb9d7ff));
pub const HUE_BLUE_BG: Color = Color::var(&HUE_BLUE_BG_VAR);

pub(crate) static HUE_BLUE_BORDER_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.hue_blue_border", rgb(0xa1caff));
pub const HUE_BLUE_BORDER: Color = Color::var(&HUE_BLUE_BORDER_VAR);

pub(crate) static HUE_BLUE_ICON_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.hue_blue_icon", rgb(0x003978));
pub const HUE_BLUE_ICON: Color = Color::var(&HUE_BLUE_ICON_VAR);

pub(crate) static HUE_BLUE_TEXT_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.hue_blue_text", rgb(0x003978));
pub const HUE_BLUE_TEXT: Color = Color::var(&HUE_BLUE_TEXT_VAR);

pub(crate) static HUE_PURPLE_BG_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.hue_purple_bg", rgb(0xf7d5ff));
pub const HUE_PURPLE_BG: Color = Color::var(&HUE_PURPLE_BG_VAR);

pub(crate) static HUE_PURPLE_BORDER_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.hue_purple_border", rgb(0xf3bfff));
pub const HUE_PURPLE_BORDER: Color = Color::var(&HUE_PURPLE_BORDER_VAR);

pub(crate) static HUE_PURPLE_ICON_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.hue_purple_icon", rgb(0x5c0e6c));
pub const HUE_PURPLE_ICON: Color = Color::var(&HUE_PURPLE_ICON_VAR);

pub(crate) static HUE_PURPLE_TEXT_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.hue_purple_text", rgb(0x5c0e6c));
pub const HUE_PURPLE_TEXT: Color = Color::var(&HUE_PURPLE_TEXT_VAR);

pub(crate) static HUE_PINK_BG_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.hue_pink_bg", rgb(0xffc0d7));
pub const HUE_PINK_BG: Color = Color::var(&HUE_PINK_BG_VAR);

pub(crate) static HUE_PINK_BORDER_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.hue_pink_border", rgb(0xffc0d7));
pub const HUE_PINK_BORDER: Color = Color::var(&HUE_PINK_BORDER_VAR);

pub(crate) static HUE_PINK_ICON_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.hue_pink_icon", rgb(0x70003f));
pub const HUE_PINK_ICON: Color = Color::var(&HUE_PINK_ICON_VAR);

pub(crate) static HUE_PINK_TEXT_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.hue_pink_text", rgb(0x70003f));
pub const HUE_PINK_TEXT: Color = Color::var(&HUE_PINK_TEXT_VAR);

pub(crate) static HUE_GRAY_BG_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.hue_gray_bg", rgb(0xe2e2e2));
pub const HUE_GRAY_BG: Color = Color::var(&HUE_GRAY_BG_VAR);

pub(crate) static HUE_GRAY_BORDER_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.hue_gray_border", rgb(0xd4d4d4));
pub const HUE_GRAY_BORDER: Color = Color::var(&HUE_GRAY_BORDER_VAR);

pub(crate) static HUE_GRAY_ICON_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.hue_gray_icon", rgb(0x474747));
pub const HUE_GRAY_ICON: Color = Color::var(&HUE_GRAY_ICON_VAR);

pub(crate) static HUE_GRAY_TEXT_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.hue_gray_text", rgb(0x1b1b1b));
pub const HUE_GRAY_TEXT: Color = Color::var(&HUE_GRAY_TEXT_VAR);

pub(crate) static DATA_BLUE_VAR: ThemeVar<Color> = ThemeVar::new("soft.data_blue", rgb(0x0171e3));
pub const DATA_BLUE: Color = Color::var(&DATA_BLUE_VAR);

pub(crate) static DATA_CYAN_VAR: ThemeVar<Color> = ThemeVar::new("soft.data_cyan", rgb(0x0171a4));
pub const DATA_CYAN: Color = Color::var(&DATA_CYAN_VAR);

pub(crate) static DATA_ORANGE_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_orange", rgb(0xeb6e00));
pub const DATA_ORANGE: Color = Color::var(&DATA_ORANGE_VAR);

pub(crate) static DATA_RED_VAR: ThemeVar<Color> = ThemeVar::new("soft.data_red", rgb(0xf5394f));
pub const DATA_RED: Color = Color::var(&DATA_RED_VAR);

pub(crate) static DATA_PURPLE_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_purple", rgb(0x6b1efd));
pub const DATA_PURPLE: Color = Color::var(&DATA_PURPLE_VAR);

pub(crate) static DATA_TEAL_VAR: ThemeVar<Color> = ThemeVar::new("soft.data_teal", rgb(0x08a3a3));
pub const DATA_TEAL: Color = Color::var(&DATA_TEAL_VAR);

pub(crate) static DATA_GREEN_VAR: ThemeVar<Color> = ThemeVar::new("soft.data_green", rgb(0x0b991f));
pub const DATA_GREEN: Color = Color::var(&DATA_GREEN_VAR);

pub(crate) static DATA_BROWN_VAR: ThemeVar<Color> = ThemeVar::new("soft.data_brown", rgb(0x965e03));
pub const DATA_BROWN: Color = Color::var(&DATA_BROWN_VAR);

pub(crate) static DATA_PINK_VAR: ThemeVar<Color> = ThemeVar::new("soft.data_pink", rgb(0xf351c0));
pub const DATA_PINK: Color = Color::var(&DATA_PINK_VAR);

pub(crate) static DATA_INDIGO_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_indigo", rgb(0x6f8aff));
pub const DATA_INDIGO: Color = Color::var(&DATA_INDIGO_VAR);

pub(crate) static DATA_NEUTRAL_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_neutral", rgb(0x8494a3));
pub const DATA_NEUTRAL: Color = Color::var(&DATA_NEUTRAL_VAR);

pub(crate) static DATA_BLUE_RAMP_1_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_blue_ramp_1", rgb(0x02165e));
pub const DATA_BLUE_RAMP_1: Color = Color::var(&DATA_BLUE_RAMP_1_VAR);

pub(crate) static DATA_BLUE_RAMP_2_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_blue_ramp_2", rgb(0x004cbc));
pub const DATA_BLUE_RAMP_2: Color = Color::var(&DATA_BLUE_RAMP_2_VAR);

pub(crate) static DATA_BLUE_RAMP_3_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_blue_ramp_3", rgb(0x2694fe));
pub const DATA_BLUE_RAMP_3: Color = Color::var(&DATA_BLUE_RAMP_3_VAR);

pub(crate) static DATA_BLUE_RAMP_4_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_blue_ramp_4", rgb(0x78beff));
pub const DATA_BLUE_RAMP_4: Color = Color::var(&DATA_BLUE_RAMP_4_VAR);

pub(crate) static DATA_BLUE_RAMP_5_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_blue_ramp_5", rgb(0xdbecff));
pub const DATA_BLUE_RAMP_5: Color = Color::var(&DATA_BLUE_RAMP_5_VAR);

pub(crate) static DATA_SHAMROCK_RAMP_1_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_shamrock_ramp_1", rgb(0x0b603d));
pub const DATA_SHAMROCK_RAMP_1: Color = Color::var(&DATA_SHAMROCK_RAMP_1_VAR);

pub(crate) static DATA_SHAMROCK_RAMP_2_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_shamrock_ramp_2", rgb(0x138546));
pub const DATA_SHAMROCK_RAMP_2: Color = Color::var(&DATA_SHAMROCK_RAMP_2_VAR);

pub(crate) static DATA_SHAMROCK_RAMP_3_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_shamrock_ramp_3", rgb(0x24bb5e));
pub const DATA_SHAMROCK_RAMP_3: Color = Color::var(&DATA_SHAMROCK_RAMP_3_VAR);

pub(crate) static DATA_SHAMROCK_RAMP_4_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_shamrock_ramp_4", rgb(0x8ef7aa));
pub const DATA_SHAMROCK_RAMP_4: Color = Color::var(&DATA_SHAMROCK_RAMP_4_VAR);

pub(crate) static DATA_SHAMROCK_RAMP_5_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_shamrock_ramp_5", rgb(0xd6fee4));
pub const DATA_SHAMROCK_RAMP_5: Color = Color::var(&DATA_SHAMROCK_RAMP_5_VAR);

pub(crate) static DATA_ORANGE_RAMP_1_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_orange_ramp_1", rgb(0xa13f04));
pub const DATA_ORANGE_RAMP_1: Color = Color::var(&DATA_ORANGE_RAMP_1_VAR);

pub(crate) static DATA_ORANGE_RAMP_2_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_orange_ramp_2", rgb(0xd66100));
pub const DATA_ORANGE_RAMP_2: Color = Color::var(&DATA_ORANGE_RAMP_2_VAR);

pub(crate) static DATA_ORANGE_RAMP_3_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_orange_ramp_3", rgb(0xfd9537));
pub const DATA_ORANGE_RAMP_3: Color = Color::var(&DATA_ORANGE_RAMP_3_VAR);

pub(crate) static DATA_ORANGE_RAMP_4_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_orange_ramp_4", rgb(0xfdb876));
pub const DATA_ORANGE_RAMP_4: Color = Color::var(&DATA_ORANGE_RAMP_4_VAR);

pub(crate) static DATA_ORANGE_RAMP_5_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_orange_ramp_5", rgb(0xffe6cf));
pub const DATA_ORANGE_RAMP_5: Color = Color::var(&DATA_ORANGE_RAMP_5_VAR);

pub(crate) static DATA_PINK_RAMP_1_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_pink_ramp_1", rgb(0x8e1073));
pub const DATA_PINK_RAMP_1: Color = Color::var(&DATA_PINK_RAMP_1_VAR);

pub(crate) static DATA_PINK_RAMP_2_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_pink_ramp_2", rgb(0xd123a1));
pub const DATA_PINK_RAMP_2: Color = Color::var(&DATA_PINK_RAMP_2_VAR);

pub(crate) static DATA_PINK_RAMP_3_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_pink_ramp_3", rgb(0xf989d3));
pub const DATA_PINK_RAMP_3: Color = Color::var(&DATA_PINK_RAMP_3_VAR);

pub(crate) static DATA_PINK_RAMP_4_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_pink_ramp_4", rgb(0xfeade3));
pub const DATA_PINK_RAMP_4: Color = Color::var(&DATA_PINK_RAMP_4_VAR);

pub(crate) static DATA_PINK_RAMP_5_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_pink_ramp_5", rgb(0xfce3f4));
pub const DATA_PINK_RAMP_5: Color = Color::var(&DATA_PINK_RAMP_5_VAR);

pub(crate) static DATA_PURPLE_RAMP_1_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_purple_ramp_1", rgb(0x3e0697));
pub const DATA_PURPLE_RAMP_1: Color = Color::var(&DATA_PURPLE_RAMP_1_VAR);

pub(crate) static DATA_PURPLE_RAMP_2_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_purple_ramp_2", rgb(0x6b1efd));
pub const DATA_PURPLE_RAMP_2: Color = Color::var(&DATA_PURPLE_RAMP_2_VAR);

pub(crate) static DATA_PURPLE_RAMP_3_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_purple_ramp_3", rgb(0x9081ff));
pub const DATA_PURPLE_RAMP_3: Color = Color::var(&DATA_PURPLE_RAMP_3_VAR);

pub(crate) static DATA_PURPLE_RAMP_4_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_purple_ramp_4", rgb(0xb3b0fe));
pub const DATA_PURPLE_RAMP_4: Color = Color::var(&DATA_PURPLE_RAMP_4_VAR);

pub(crate) static DATA_PURPLE_RAMP_5_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_purple_ramp_5", rgb(0xe8e8fb));
pub const DATA_PURPLE_RAMP_5: Color = Color::var(&DATA_PURPLE_RAMP_5_VAR);

pub(crate) static DATA_RED_RAMP_1_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_red_ramp_1", rgb(0x9d0519));
pub const DATA_RED_RAMP_1: Color = Color::var(&DATA_RED_RAMP_1_VAR);

pub(crate) static DATA_RED_RAMP_2_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_red_ramp_2", rgb(0xd31130));
pub const DATA_RED_RAMP_2: Color = Color::var(&DATA_RED_RAMP_2_VAR);

pub(crate) static DATA_RED_RAMP_3_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_red_ramp_3", rgb(0xfb7d87));
pub const DATA_RED_RAMP_3: Color = Color::var(&DATA_RED_RAMP_3_VAR);

pub(crate) static DATA_RED_RAMP_4_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_red_ramp_4", rgb(0xffb2b8));
pub const DATA_RED_RAMP_4: Color = Color::var(&DATA_RED_RAMP_4_VAR);

pub(crate) static DATA_RED_RAMP_5_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_red_ramp_5", rgb(0xfee4e6));
pub const DATA_RED_RAMP_5: Color = Color::var(&DATA_RED_RAMP_5_VAR);

pub(crate) static DATA_TEAL_RAMP_1_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_teal_ramp_1", rgb(0x08767d));
pub const DATA_TEAL_RAMP_1: Color = Color::var(&DATA_TEAL_RAMP_1_VAR);

pub(crate) static DATA_TEAL_RAMP_2_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_teal_ramp_2", rgb(0x0c9293));
pub const DATA_TEAL_RAMP_2: Color = Color::var(&DATA_TEAL_RAMP_2_VAR);

pub(crate) static DATA_TEAL_RAMP_3_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_teal_ramp_3", rgb(0x0db7af));
pub const DATA_TEAL_RAMP_3: Color = Color::var(&DATA_TEAL_RAMP_3_VAR);

pub(crate) static DATA_TEAL_RAMP_4_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_teal_ramp_4", rgb(0x6ce6d8));
pub const DATA_TEAL_RAMP_4: Color = Color::var(&DATA_TEAL_RAMP_4_VAR);

pub(crate) static DATA_TEAL_RAMP_5_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_teal_ramp_5", rgb(0xd7fcf8));
pub const DATA_TEAL_RAMP_5: Color = Color::var(&DATA_TEAL_RAMP_5_VAR);

pub(crate) static DATA_YELLOW_RAMP_1_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_yellow_ramp_1", rgb(0x8a5001));
pub const DATA_YELLOW_RAMP_1: Color = Color::var(&DATA_YELLOW_RAMP_1_VAR);

pub(crate) static DATA_YELLOW_RAMP_2_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_yellow_ramp_2", rgb(0xd69804));
pub const DATA_YELLOW_RAMP_2: Color = Color::var(&DATA_YELLOW_RAMP_2_VAR);

pub(crate) static DATA_YELLOW_RAMP_3_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_yellow_ramp_3", rgb(0xfbce03));
pub const DATA_YELLOW_RAMP_3: Color = Color::var(&DATA_YELLOW_RAMP_3_VAR);

pub(crate) static DATA_YELLOW_RAMP_4_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_yellow_ramp_4", rgb(0xfcec85));
pub const DATA_YELLOW_RAMP_4: Color = Color::var(&DATA_YELLOW_RAMP_4_VAR);

pub(crate) static DATA_YELLOW_RAMP_5_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_yellow_ramp_5", rgb(0xfdf6ba));
pub const DATA_YELLOW_RAMP_5: Color = Color::var(&DATA_YELLOW_RAMP_5_VAR);

pub(crate) static DATA_GRAY_RAMP_1_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_gray_ramp_1", rgb(0x25363f));
pub const DATA_GRAY_RAMP_1: Color = Color::var(&DATA_GRAY_RAMP_1_VAR);

pub(crate) static DATA_GRAY_RAMP_2_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_gray_ramp_2", rgb(0x5d6c7b));
pub const DATA_GRAY_RAMP_2: Color = Color::var(&DATA_GRAY_RAMP_2_VAR);

pub(crate) static DATA_GRAY_RAMP_3_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_gray_ramp_3", rgb(0xafb9c4));
pub const DATA_GRAY_RAMP_3: Color = Color::var(&DATA_GRAY_RAMP_3_VAR);

pub(crate) static DATA_GRAY_RAMP_4_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_gray_ramp_4", rgb(0xccd3db));
pub const DATA_GRAY_RAMP_4: Color = Color::var(&DATA_GRAY_RAMP_4_VAR);

pub(crate) static DATA_GRAY_RAMP_5_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_gray_ramp_5", rgb(0xf1f4f7));
pub const DATA_GRAY_RAMP_5: Color = Color::var(&DATA_GRAY_RAMP_5_VAR);

pub(crate) static SYNTAX_BACKGROUND_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.syntax_background", rgb(0xffffff));
pub const SYNTAX_BACKGROUND: Color = Color::var(&SYNTAX_BACKGROUND_VAR);

pub(crate) static SYNTAX_ADDED_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.syntax_added", rgba(0xbce0bb, 64.0 / 255.0));
pub const SYNTAX_ADDED: Color = Color::var(&SYNTAX_ADDED_VAR);

pub(crate) static SYNTAX_REMOVED_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.syntax_removed", rgba(0xffc4be, 64.0 / 255.0));
pub const SYNTAX_REMOVED: Color = Color::var(&SYNTAX_REMOVED_VAR);

pub(crate) static SYNTAX_DIFF_HEADER_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.syntax_diff_header", Color::TRANSPARENT);
pub const SYNTAX_DIFF_HEADER: Color = Color::var(&SYNTAX_DIFF_HEADER_VAR);

pub(crate) static SYNTAX_ADDED_MARKER_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.syntax_added_marker", rgb(0x6a6a6a));
pub const SYNTAX_ADDED_MARKER: Color = Color::var(&SYNTAX_ADDED_MARKER_VAR);

pub(crate) static SYNTAX_REMOVED_MARKER_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.syntax_removed_marker", rgb(0x6a6a6a));
pub const SYNTAX_REMOVED_MARKER: Color = Color::var(&SYNTAX_REMOVED_MARKER_VAR);

/// Set every color variable of the kit to its value in `mode`.
pub(crate) fn set_colors(theme: Theme, mode: ThemeMode) -> Theme {
    let dark = mode == ThemeMode::Dark;
    let mut theme = theme;
    theme = set_colors_base_1(theme, dark);
    theme = set_colors_base_2(theme, dark);
    theme = set_colors_base_3(theme, dark);
    theme = set_colors_base_more_1(theme, dark);
    theme = set_colors_base_more_2(theme, dark);
    theme = set_colors_base_more_3(theme, dark);
    theme = set_colors_status_1(theme, dark);
    theme = set_colors_status_2(theme, dark);
    theme = set_colors_status_3(theme, dark);
    theme = set_colors_hue_1(theme, dark);
    theme = set_colors_hue_2(theme, dark);
    theme = set_colors_hue_3(theme, dark);
    theme = set_colors_hue_4(theme, dark);
    theme = set_colors_hue_5(theme, dark);
    theme = set_colors_data_1(theme);
    theme = set_colors_data_2(theme, dark);
    theme = set_colors_data_3(theme);
    theme = set_colors_data_4(theme);
    theme = set_colors_data_5(theme);
    theme = set_colors_data_6(theme);
    theme = set_colors_data_7(theme, dark);
    theme = set_colors_syntax(theme, dark);
    theme
}

fn set_colors_base_1(theme: Theme, dark: bool) -> Theme {
    theme
        .set(
            &BG_BODY_VAR,
            if dark { rgb(0x1b1b1b) } else { rgb(0xf1f1f1) },
        )
        .set(
            &BG_SURFACE_VAR,
            if dark { rgb(0x262626) } else { rgb(0xffffff) },
        )
        .set(
            &BG_CARD_VAR,
            if dark { rgb(0x1b1b1b) } else { rgb(0xffffff) },
        )
        .set(
            &BG_POPOVER_VAR,
            if dark { rgb(0x1b1b1b) } else { rgb(0xffffff) },
        )
        .set(
            &BG_MUTED_VAR,
            if dark { rgb(0x1b1b1b) } else { rgb(0xf1f1f1) },
        )
        .set(
            &BG_INVERTED_VAR,
            if dark { rgb(0xffffff) } else { rgb(0x0a1317) },
        )
        .set(
            &BG_ERROR_INVERTED_VAR,
            if dark { rgb(0xe3193b) } else { rgb(0xaa071e) },
        )
        .set(
            &NEUTRAL_VAR,
            if dark {
                rgba(0xffffff, 26.0 / 255.0)
            } else {
                rgba(0x000000, 15.0 / 255.0)
            },
        )
}

fn set_colors_base_2(theme: Theme, dark: bool) -> Theme {
    theme
        .set(
            &ACCENT_VAR,
            if dark { rgb(0xf1f1f1) } else { rgb(0x1b1b1b) },
        )
        .set(
            &ACCENT_HOVER_VAR,
            if dark { rgb(0xf3f3f3) } else { rgb(0x171717) },
        )
        .set(
            &CHOICE_BORDER_HOVER_VAR,
            if dark { rgb(0x757575) } else { rgb(0xaaaaaa) },
        )
        .set(
            &CHOICE_SURFACE_HOVER_VAR,
            if dark { rgb(0x313131) } else { rgb(0xf2f2f2) },
        )
        .set(
            &SWITCH_OFF_HOVER_VAR,
            if dark { rgb(0x5b5b5b) } else { rgb(0xc9c9c9) },
        )
        .set(
            &ACCENT_MUTED_VAR,
            if dark { rgb(0x262626) } else { rgb(0xf1f1f1) },
        )
        .set(
            &ON_ACCENT_VAR,
            if dark { rgb(0x111111) } else { rgb(0xffffff) },
        )
        .set(&ON_DARK_VAR, rgb(0xffffff))
        .set(&WHITE_VAR, rgb(0xffffff))
}

fn set_colors_base_3(theme: Theme, dark: bool) -> Theme {
    theme.set(&ON_LIGHT_VAR, rgb(0x111111)).set(
        &TEXT_PRIMARY_VAR,
        if dark { rgb(0xffffff) } else { rgb(0x000000) },
    )
}

fn set_colors_base_more_1(theme: Theme, dark: bool) -> Theme {
    theme
        .set(
            &TEXT_SECONDARY_VAR,
            if dark { rgb(0x9e9e9e) } else { rgb(0x474747) },
        )
        .set(
            &TEXT_DISABLED_VAR,
            if dark { rgb(0x525252) } else { rgb(0x919191) },
        )
        .set(
            &TEXT_ACCENT_VAR,
            if dark { rgb(0xf1f1f1) } else { rgb(0x1b1b1b) },
        )
        .set(
            &ICON_PRIMARY_VAR,
            if dark { rgb(0xffffff) } else { rgb(0x000000) },
        )
        .set(
            &ICON_SECONDARY_VAR,
            if dark { rgb(0x9e9e9e) } else { rgb(0x6a6a6a) },
        )
        .set(
            &ICON_DISABLED_VAR,
            if dark { rgb(0x525252) } else { rgb(0x919191) },
        )
        .set(
            &ICON_ACCENT_VAR,
            if dark { rgb(0xf1f1f1) } else { rgb(0x1b1b1b) },
        )
        .set(
            &BORDER_VAR,
            if dark {
                rgba(0xffffff, 26.0 / 255.0)
            } else {
                rgba(0x000000, 20.0 / 255.0)
            },
        )
}

fn set_colors_base_more_2(theme: Theme, dark: bool) -> Theme {
    theme
        .set(
            &BORDER_EMPHASIZED_VAR,
            if dark { rgb(0x525252) } else { rgb(0xd4d4d4) },
        )
        .set(
            &SKELETON_VAR,
            if dark { rgb(0x525252) } else { rgb(0xf1f1f1) },
        )
        .set(&TRACK_VAR, if dark { rgb(0x5a5e66) } else { rgb(0xccd3db) })
        .set(
            &SHADOW_VAR,
            if dark {
                rgba(0x000000, 77.0 / 255.0)
            } else {
                rgba(0x000000, 26.0 / 255.0)
            },
        )
        .set(
            &TINT_HOVER_VAR,
            if dark { rgb(0xffffff) } else { rgb(0x000000) },
        )
        .set(
            &OVERLAY_SCRIM_VAR,
            if dark {
                rgba(0x000000, 204.0 / 255.0)
            } else {
                rgba(0x000000, 128.0 / 255.0)
            },
        )
        .set(&OVERLAY_LIGHT_SCRIM_VAR, rgba(0xffffff, 0.6))
        .set(
            &OVERLAY_HOVER_VAR,
            if dark {
                rgba(0xffffff, 13.0 / 255.0)
            } else {
                rgba(0x000000, 13.0 / 255.0)
            },
        )
}

fn set_colors_base_more_3(theme: Theme, dark: bool) -> Theme {
    theme
        .set(
            &OVERLAY_PRESSED_VAR,
            if dark {
                rgba(0xffffff, 26.0 / 255.0)
            } else {
                rgba(0x000000, 26.0 / 255.0)
            },
        )
        .set(
            &FOCUS_RING_VAR,
            if dark { rgb(0xf1f1f1) } else { rgb(0x1b1b1b) },
        )
}

fn set_colors_status_1(theme: Theme, dark: bool) -> Theme {
    theme
        .set(&INFO_VAR, if dark { rgb(0x6d9cfe) } else { rgb(0x0074e2) })
        .set(
            &ON_INFO_VAR,
            if dark { rgb(0x111111) } else { rgb(0xffffff) },
        )
        .set(
            &INFO_MUTED_VAR,
            if dark {
                rgba(0x88bcff, 61.0 / 255.0)
            } else {
                rgb(0xb9d7ff)
            },
        )
        .set(
            &ON_INFO_MUTED_VAR,
            if dark { rgb(0xa1caff) } else { rgb(0x003978) },
        )
        .set(
            &TEXT_INFO_VAR,
            if dark { rgb(0xa1caff) } else { rgb(0x0067ce) },
        )
        .set(
            &SUCCESS_VAR,
            if dark { rgb(0x64af4c) } else { rgb(0x198100) },
        )
        .set(
            &ON_SUCCESS_VAR,
            if dark { rgb(0x111111) } else { rgb(0xffffff) },
        )
        .set(
            &SUCCESS_MUTED_VAR,
            if dark {
                rgba(0x90ca90, 61.0 / 255.0)
            } else {
                rgb(0xbce0bb)
            },
        )
}

fn set_colors_status_2(theme: Theme, dark: bool) -> Theme {
    theme
        .set(
            &ON_SUCCESS_MUTED_VAR,
            if dark { rgb(0x90ca90) } else { rgb(0x00490b) },
        )
        .set(
            &TEXT_SUCCESS_VAR,
            if dark { rgb(0x90ca90) } else { rgb(0x2f7d33) },
        )
        .set(&WARNING_VAR, rgb(0xffce2f))
        .set(&ON_WARNING_VAR, rgb(0x111111))
        .set(
            &WARNING_MUTED_VAR,
            if dark {
                rgba(0xe2b623, 61.0 / 255.0)
            } else {
                rgb(0xfae19e)
            },
        )
        .set(
            &ON_WARNING_MUTED_VAR,
            if dark { rgb(0xeec448) } else { rgb(0x4b3900) },
        )
        .set(
            &TEXT_WARNING_VAR,
            if dark { rgb(0xeec448) } else { rgb(0x836700) },
        )
        .set(&ERROR_VAR, if dark { rgb(0xff705d) } else { rgb(0xc9303a) })
}

fn set_colors_status_3(theme: Theme, dark: bool) -> Theme {
    theme
        .set(
            &ON_ERROR_VAR,
            if dark { rgb(0x111111) } else { rgb(0xffffff) },
        )
        .set(
            &ERROR_MUTED_VAR,
            if dark {
                rgba(0xff9890, 61.0 / 255.0)
            } else {
                rgb(0xffc4be)
            },
        )
        .set(
            &ON_ERROR_MUTED_VAR,
            if dark { rgb(0xffaea7) } else { rgb(0x76000c) },
        )
        .set(
            &TEXT_ERROR_VAR,
            if dark { rgb(0xffaea7) } else { rgb(0xc0262c) },
        )
}

fn set_colors_hue_1(theme: Theme, dark: bool) -> Theme {
    theme
        .set(
            &HUE_RED_BG_VAR,
            if dark { rgb(0x5b2b28) } else { rgb(0xffc4be) },
        )
        .set(
            &HUE_RED_BORDER_VAR,
            if dark { rgb(0xfa6762) } else { rgb(0xffaea7) },
        )
        .set(
            &HUE_RED_ICON_VAR,
            if dark { rgb(0xff9890) } else { rgb(0x76000c) },
        )
        .set(
            &HUE_RED_TEXT_VAR,
            if dark { rgb(0xffaea7) } else { rgb(0x76000c) },
        )
        .set(
            &HUE_ORANGE_BG_VAR,
            if dark { rgb(0x503424) } else { rgb(0xffc7a1) },
        )
        .set(
            &HUE_ORANGE_BORDER_VAR,
            if dark { rgb(0xdf843f) } else { rgb(0xffc7a1) },
        )
        .set(
            &HUE_ORANGE_ICON_VAR,
            if dark { rgb(0xff9e55) } else { rgb(0x622700) },
        )
        .set(
            &HUE_ORANGE_TEXT_VAR,
            if dark { rgb(0xffb37e) } else { rgb(0x622700) },
        )
}

fn set_colors_hue_2(theme: Theme, dark: bool) -> Theme {
    theme
        .set(
            &HUE_YELLOW_BG_VAR,
            if dark { rgb(0x453a1c) } else { rgb(0xfae19e) },
        )
        .set(
            &HUE_YELLOW_BORDER_VAR,
            if dark { rgb(0xc29900) } else { rgb(0xe3c36c) },
        )
        .set(
            &HUE_YELLOW_ICON_VAR,
            if dark { rgb(0xe2b623) } else { rgb(0x4b3900) },
        )
        .set(
            &HUE_YELLOW_TEXT_VAR,
            if dark { rgb(0xeec448) } else { rgb(0x4b3900) },
        )
        .set(
            &HUE_GREEN_BG_VAR,
            if dark { rgb(0x2b422b) } else { rgb(0xbce0bb) },
        )
        .set(
            &HUE_GREEN_BORDER_VAR,
            if dark { rgb(0x6ab26b) } else { rgb(0xaad4a9) },
        )
        .set(
            &HUE_GREEN_ICON_VAR,
            if dark { rgb(0x90ca90) } else { rgb(0x00490b) },
        )
        .set(
            &HUE_GREEN_TEXT_VAR,
            if dark { rgb(0x90ca90) } else { rgb(0x00490b) },
        )
}

fn set_colors_hue_3(theme: Theme, dark: bool) -> Theme {
    theme
        .set(
            &HUE_TEAL_BG_VAR,
            if dark { rgb(0x28413c) } else { rgb(0xa9e2d6) },
        )
        .set(
            &HUE_TEAL_BORDER_VAR,
            if dark { rgb(0x4fb1a0) } else { rgb(0x90d7c8) },
        )
        .set(
            &HUE_TEAL_ICON_VAR,
            if dark { rgb(0x81c9bb) } else { rgb(0x00463d) },
        )
        .set(
            &HUE_TEAL_TEXT_VAR,
            if dark { rgb(0x90d7c8) } else { rgb(0x00463d) },
        )
        .set(
            &HUE_CYAN_BG_VAR,
            if dark { rgb(0x274046) } else { rgb(0x9ae2f4) },
        )
        .set(
            &HUE_CYAN_BORDER_VAR,
            if dark { rgb(0x49adc4) } else { rgb(0x85d5e9) },
        )
        .set(
            &HUE_CYAN_ICON_VAR,
            if dark { rgb(0x71c7dd) } else { rgb(0x004351) },
        )
        .set(
            &HUE_CYAN_TEXT_VAR,
            if dark { rgb(0x85d5e9) } else { rgb(0x004351) },
        )
}

fn set_colors_hue_4(theme: Theme, dark: bool) -> Theme {
    theme
        .set(
            &HUE_BLUE_BG_VAR,
            if dark { rgb(0x253c5a) } else { rgb(0xb9d7ff) },
        )
        .set(
            &HUE_BLUE_BORDER_VAR,
            if dark { rgb(0x5aa0f8) } else { rgb(0xa1caff) },
        )
        .set(
            &HUE_BLUE_ICON_VAR,
            if dark { rgb(0x88bcff) } else { rgb(0x003978) },
        )
        .set(
            &HUE_BLUE_TEXT_VAR,
            if dark { rgb(0xa1caff) } else { rgb(0x003978) },
        )
        .set(
            &HUE_PURPLE_BG_VAR,
            if dark { rgb(0x4a2f51) } else { rgb(0xf7d5ff) },
        )
        .set(
            &HUE_PURPLE_BORDER_VAR,
            if dark { rgb(0xd885eb) } else { rgb(0xf3bfff) },
        )
        .set(
            &HUE_PURPLE_ICON_VAR,
            if dark { rgb(0xe496f6) } else { rgb(0x5c0e6c) },
        )
        .set(
            &HUE_PURPLE_TEXT_VAR,
            if dark { rgb(0xeaacf8) } else { rgb(0x5c0e6c) },
        )
}

fn set_colors_hue_5(theme: Theme, dark: bool) -> Theme {
    theme
        .set(
            &HUE_PINK_BG_VAR,
            if dark { rgb(0x572b3d) } else { rgb(0xffc0d7) },
        )
        .set(
            &HUE_PINK_BORDER_VAR,
            if dark { rgb(0xfc78b1) } else { rgb(0xffc0d7) },
        )
        .set(
            &HUE_PINK_ICON_VAR,
            if dark { rgb(0xfd92bd) } else { rgb(0x70003f) },
        )
        .set(
            &HUE_PINK_TEXT_VAR,
            if dark { rgb(0xffa9ca) } else { rgb(0x70003f) },
        )
        .set(
            &HUE_GRAY_BG_VAR,
            if dark { rgb(0x303030) } else { rgb(0xe2e2e2) },
        )
        .set(
            &HUE_GRAY_BORDER_VAR,
            if dark { rgb(0x262626) } else { rgb(0xd4d4d4) },
        )
        .set(
            &HUE_GRAY_ICON_VAR,
            if dark { rgb(0x9e9e9e) } else { rgb(0x474747) },
        )
        .set(
            &HUE_GRAY_TEXT_VAR,
            if dark { rgb(0xd4d4d4) } else { rgb(0x1b1b1b) },
        )
}

fn set_colors_data_1(theme: Theme) -> Theme {
    theme
        .set(&DATA_BLUE_VAR, rgb(0x0171e3))
        .set(&DATA_CYAN_VAR, rgb(0x0171a4))
        .set(&DATA_ORANGE_VAR, rgb(0xeb6e00))
        .set(&DATA_RED_VAR, rgb(0xf5394f))
        .set(&DATA_PURPLE_VAR, rgb(0x6b1efd))
        .set(&DATA_TEAL_VAR, rgb(0x08a3a3))
        .set(&DATA_GREEN_VAR, rgb(0x0b991f))
        .set(&DATA_BROWN_VAR, rgb(0x965e03))
}

fn set_colors_data_2(theme: Theme, dark: bool) -> Theme {
    theme
        .set(&DATA_PINK_VAR, rgb(0xf351c0))
        .set(&DATA_INDIGO_VAR, rgb(0x6f8aff))
        .set(
            &DATA_NEUTRAL_VAR,
            if dark { rgb(0x8c939b) } else { rgb(0x8494a3) },
        )
        .set(&DATA_BLUE_RAMP_1_VAR, rgb(0x02165e))
        .set(&DATA_BLUE_RAMP_2_VAR, rgb(0x004cbc))
        .set(&DATA_BLUE_RAMP_3_VAR, rgb(0x2694fe))
        .set(&DATA_BLUE_RAMP_4_VAR, rgb(0x78beff))
        .set(&DATA_BLUE_RAMP_5_VAR, rgb(0xdbecff))
}

fn set_colors_data_3(theme: Theme) -> Theme {
    theme
        .set(&DATA_SHAMROCK_RAMP_1_VAR, rgb(0x0b603d))
        .set(&DATA_SHAMROCK_RAMP_2_VAR, rgb(0x138546))
        .set(&DATA_SHAMROCK_RAMP_3_VAR, rgb(0x24bb5e))
        .set(&DATA_SHAMROCK_RAMP_4_VAR, rgb(0x8ef7aa))
        .set(&DATA_SHAMROCK_RAMP_5_VAR, rgb(0xd6fee4))
        .set(&DATA_ORANGE_RAMP_1_VAR, rgb(0xa13f04))
        .set(&DATA_ORANGE_RAMP_2_VAR, rgb(0xd66100))
        .set(&DATA_ORANGE_RAMP_3_VAR, rgb(0xfd9537))
}

fn set_colors_data_4(theme: Theme) -> Theme {
    theme
        .set(&DATA_ORANGE_RAMP_4_VAR, rgb(0xfdb876))
        .set(&DATA_ORANGE_RAMP_5_VAR, rgb(0xffe6cf))
        .set(&DATA_PINK_RAMP_1_VAR, rgb(0x8e1073))
        .set(&DATA_PINK_RAMP_2_VAR, rgb(0xd123a1))
        .set(&DATA_PINK_RAMP_3_VAR, rgb(0xf989d3))
        .set(&DATA_PINK_RAMP_4_VAR, rgb(0xfeade3))
        .set(&DATA_PINK_RAMP_5_VAR, rgb(0xfce3f4))
        .set(&DATA_PURPLE_RAMP_1_VAR, rgb(0x3e0697))
}

fn set_colors_data_5(theme: Theme) -> Theme {
    theme
        .set(&DATA_PURPLE_RAMP_2_VAR, rgb(0x6b1efd))
        .set(&DATA_PURPLE_RAMP_3_VAR, rgb(0x9081ff))
        .set(&DATA_PURPLE_RAMP_4_VAR, rgb(0xb3b0fe))
        .set(&DATA_PURPLE_RAMP_5_VAR, rgb(0xe8e8fb))
        .set(&DATA_RED_RAMP_1_VAR, rgb(0x9d0519))
        .set(&DATA_RED_RAMP_2_VAR, rgb(0xd31130))
        .set(&DATA_RED_RAMP_3_VAR, rgb(0xfb7d87))
        .set(&DATA_RED_RAMP_4_VAR, rgb(0xffb2b8))
}

fn set_colors_data_6(theme: Theme) -> Theme {
    theme
        .set(&DATA_RED_RAMP_5_VAR, rgb(0xfee4e6))
        .set(&DATA_TEAL_RAMP_1_VAR, rgb(0x08767d))
        .set(&DATA_TEAL_RAMP_2_VAR, rgb(0x0c9293))
        .set(&DATA_TEAL_RAMP_3_VAR, rgb(0x0db7af))
        .set(&DATA_TEAL_RAMP_4_VAR, rgb(0x6ce6d8))
        .set(&DATA_TEAL_RAMP_5_VAR, rgb(0xd7fcf8))
        .set(&DATA_YELLOW_RAMP_1_VAR, rgb(0x8a5001))
        .set(&DATA_YELLOW_RAMP_2_VAR, rgb(0xd69804))
}

fn set_colors_data_7(theme: Theme, dark: bool) -> Theme {
    theme
        .set(&DATA_YELLOW_RAMP_3_VAR, rgb(0xfbce03))
        .set(&DATA_YELLOW_RAMP_4_VAR, rgb(0xfcec85))
        .set(&DATA_YELLOW_RAMP_5_VAR, rgb(0xfdf6ba))
        .set(
            &DATA_GRAY_RAMP_1_VAR,
            if dark { rgb(0x333338) } else { rgb(0x25363f) },
        )
        .set(
            &DATA_GRAY_RAMP_2_VAR,
            if dark { rgb(0x666a72) } else { rgb(0x5d6c7b) },
        )
        .set(
            &DATA_GRAY_RAMP_3_VAR,
            if dark { rgb(0xb2b8be) } else { rgb(0xafb9c4) },
        )
        .set(
            &DATA_GRAY_RAMP_4_VAR,
            if dark { rgb(0xd0d3d6) } else { rgb(0xccd3db) },
        )
        .set(
            &DATA_GRAY_RAMP_5_VAR,
            if dark { rgb(0xf2f4f6) } else { rgb(0xf1f4f7) },
        )
}

fn set_colors_syntax(theme: Theme, dark: bool) -> Theme {
    theme
        .set(
            &SYNTAX_BACKGROUND_VAR,
            if dark { rgb(0x111111) } else { rgb(0xffffff) },
        )
        .set(
            &SYNTAX_ADDED_VAR,
            if dark {
                rgba(0x90ca90, 15.0 / 255.0)
            } else {
                rgba(0xbce0bb, 64.0 / 255.0)
            },
        )
        .set(
            &SYNTAX_REMOVED_VAR,
            if dark {
                rgba(0xff9890, 15.0 / 255.0)
            } else {
                rgba(0xffc4be, 64.0 / 255.0)
            },
        )
        .set(&SYNTAX_DIFF_HEADER_VAR, Color::TRANSPARENT)
        .set(
            &SYNTAX_ADDED_MARKER_VAR,
            if dark { rgb(0x9e9e9e) } else { rgb(0x6a6a6a) },
        )
        .set(
            &SYNTAX_REMOVED_MARKER_VAR,
            if dark { rgb(0x9e9e9e) } else { rgb(0x6a6a6a) },
        )
}

/// Black at `alpha`: the color of the outer elevation shadows.
pub(crate) fn shadow_black(alpha: f64) -> Color {
    rgba(0x000000, alpha)
}

/// White at `alpha`: the inner edge of a dark elevation shadow.
pub(crate) fn shadow_white(alpha: f64) -> Color {
    rgba(0xffffff, alpha)
}

/// The error red at `alpha`: the error edge shadow.
pub(crate) fn shadow_error(alpha: f64) -> Color {
    rgba(0xde4745, alpha)
}

/// The outline of an input under the pointer.
pub(crate) fn input_hover_color(dark: bool) -> Color {
    rgba(if dark { 0x525252 } else { 0xd4d4d4 }, 0.3)
}

/// The outline of a focused input.
pub(crate) fn input_focus_color(dark: bool) -> Color {
    rgb(if dark { 0x262626 } else { 0xf1f1f1 })
}
