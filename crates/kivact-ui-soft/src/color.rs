//! Role: The Astryx Neutral colors: a theme variable for each color of the kit,
//! with the value each appearance sets it to, in `tokens`.
use kivact::style::{Color, ThemeVar, rgb, rgba};

pub static BG_BODY_VAR: ThemeVar<Color> = ThemeVar::new("soft.bg_body", rgb(0xf1f1f1));
pub const BG_BODY: Color = Color::var(&BG_BODY_VAR);

pub static BG_SURFACE_VAR: ThemeVar<Color> =
    ThemeVar::alias("soft.bg_surface", &kivact::style::color::SURFACE_VAR);
pub const BG_SURFACE: Color = Color::var(&BG_SURFACE_VAR);

pub static BG_CARD_VAR: ThemeVar<Color> = ThemeVar::new("soft.bg_card", rgb(0xffffff));
pub const BG_CARD: Color = Color::var(&BG_CARD_VAR);

pub static BG_POPOVER_VAR: ThemeVar<Color> =
    ThemeVar::alias("soft.bg_popover", &kivact::style::color::RAISED_VAR);
pub const BG_POPOVER: Color = Color::var(&BG_POPOVER_VAR);

pub static BG_MUTED_VAR: ThemeVar<Color> =
    ThemeVar::alias("soft.bg_muted", &kivact::style::color::SURFACE_ALT_VAR);
pub const BG_MUTED: Color = Color::var(&BG_MUTED_VAR);

pub static BG_INVERTED_VAR: ThemeVar<Color> = ThemeVar::new("soft.bg_inverted", rgb(0x0a1317));
pub const BG_INVERTED: Color = Color::var(&BG_INVERTED_VAR);

pub static BG_ERROR_INVERTED_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.bg_error_inverted", rgb(0xaa071e));
pub const BG_ERROR_INVERTED: Color = Color::var(&BG_ERROR_INVERTED_VAR);

pub static NEUTRAL_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.neutral", rgba(0x000000, 15.0 / 255.0));
pub const NEUTRAL: Color = Color::var(&NEUTRAL_VAR);

pub static ACCENT_VAR: ThemeVar<Color> =
    ThemeVar::alias("soft.accent", &kivact::style::color::ACCENT_VAR);
pub const ACCENT: Color = Color::var(&ACCENT_VAR);

pub static ACCENT_HOVER_VAR: ThemeVar<Color> = ThemeVar::new("soft.accent_hover", rgb(0x171717));
pub const ACCENT_HOVER: Color = Color::var(&ACCENT_HOVER_VAR);

pub static CHOICE_BORDER_HOVER_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.choice_border_hover", rgb(0xaaaaaa));
pub const CHOICE_BORDER_HOVER: Color = Color::var(&CHOICE_BORDER_HOVER_VAR);

pub static CHOICE_SURFACE_HOVER_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.choice_surface_hover", rgb(0xf2f2f2));
pub const CHOICE_SURFACE_HOVER: Color = Color::var(&CHOICE_SURFACE_HOVER_VAR);

pub static SWITCH_OFF_HOVER_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.switch_off_hover", rgb(0xc9c9c9));
pub const SWITCH_OFF_HOVER: Color = Color::var(&SWITCH_OFF_HOVER_VAR);

pub static ACCENT_MUTED_VAR: ThemeVar<Color> = ThemeVar::new("soft.accent_muted", rgb(0xf1f1f1));
pub const ACCENT_MUTED: Color = Color::var(&ACCENT_MUTED_VAR);

pub static ON_ACCENT_VAR: ThemeVar<Color> =
    ThemeVar::alias("soft.on_accent", &kivact::style::color::ACCENT_TEXT_VAR);
pub const ON_ACCENT: Color = Color::var(&ON_ACCENT_VAR);

pub static ON_DARK_VAR: ThemeVar<Color> = ThemeVar::new("soft.on_dark", rgb(0xffffff));
pub const ON_DARK: Color = Color::var(&ON_DARK_VAR);

pub static WHITE_VAR: ThemeVar<Color> = ThemeVar::new("soft.white", rgb(0xffffff));

pub static ON_LIGHT_VAR: ThemeVar<Color> = ThemeVar::new("soft.on_light", rgb(0x111111));
pub const ON_LIGHT: Color = Color::var(&ON_LIGHT_VAR);

pub static TEXT_PRIMARY_VAR: ThemeVar<Color> =
    ThemeVar::alias("soft.text_primary", &kivact::style::color::TEXT_VAR);
pub const TEXT_PRIMARY: Color = Color::var(&TEXT_PRIMARY_VAR);

pub static TEXT_SECONDARY_VAR: ThemeVar<Color> =
    ThemeVar::alias("soft.text_secondary", &kivact::style::color::TEXT_MUTED_VAR);
pub const TEXT_SECONDARY: Color = Color::var(&TEXT_SECONDARY_VAR);

pub static TEXT_DISABLED_VAR: ThemeVar<Color> = ThemeVar::new("soft.text_disabled", rgb(0x919191));
pub const TEXT_DISABLED: Color = Color::var(&TEXT_DISABLED_VAR);

pub static TEXT_ACCENT_VAR: ThemeVar<Color> = ThemeVar::new("soft.text_accent", rgb(0x1b1b1b));
pub const TEXT_ACCENT: Color = Color::var(&TEXT_ACCENT_VAR);

pub static ICON_PRIMARY_VAR: ThemeVar<Color> = ThemeVar::new("soft.icon_primary", rgb(0x000000));
pub const ICON_PRIMARY: Color = Color::var(&ICON_PRIMARY_VAR);

pub static ICON_SECONDARY_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.icon_secondary", rgb(0x6a6a6a));
pub const ICON_SECONDARY: Color = Color::var(&ICON_SECONDARY_VAR);

pub static ICON_DISABLED_VAR: ThemeVar<Color> = ThemeVar::new("soft.icon_disabled", rgb(0x919191));
pub const ICON_DISABLED: Color = Color::var(&ICON_DISABLED_VAR);

pub static ICON_ACCENT_VAR: ThemeVar<Color> = ThemeVar::new("soft.icon_accent", rgb(0x1b1b1b));
pub const ICON_ACCENT: Color = Color::var(&ICON_ACCENT_VAR);

pub static BORDER_VAR: ThemeVar<Color> =
    ThemeVar::alias("soft.border", &kivact::style::color::BORDER_VAR);
pub(crate) const BORDER: Color = Color::var(&BORDER_VAR);

pub static BORDER_EMPHASIZED_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.border_emphasized", rgb(0xd4d4d4));
pub const BORDER_EMPHASIZED: Color = Color::var(&BORDER_EMPHASIZED_VAR);

pub static SKELETON_VAR: ThemeVar<Color> = ThemeVar::new("soft.skeleton", rgb(0xf1f1f1));
pub const SKELETON: Color = Color::var(&SKELETON_VAR);

pub static TRACK_VAR: ThemeVar<Color> = ThemeVar::new("soft.track", rgb(0xccd3db));
pub const TRACK: Color = Color::var(&TRACK_VAR);

pub static SHADOW_VAR: ThemeVar<Color> =
    ThemeVar::alias("soft.shadow", &kivact::style::color::SHADOW_VAR);
pub const SHADOW: Color = Color::var(&SHADOW_VAR);

pub static TINT_HOVER_VAR: ThemeVar<Color> = ThemeVar::new("soft.tint_hover", rgb(0x000000));
pub const TINT_HOVER: Color = Color::var(&TINT_HOVER_VAR);

pub static OVERLAY_SCRIM_VAR: ThemeVar<Color> =
    ThemeVar::alias("soft.overlay_scrim", &kivact::style::color::OVERLAY_VAR);
pub const OVERLAY_SCRIM: Color = Color::var(&OVERLAY_SCRIM_VAR);

pub static OVERLAY_LIGHT_SCRIM_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.overlay_light_scrim", rgba(0xffffff, 0.6));
pub const OVERLAY_LIGHT_SCRIM: Color = Color::var(&OVERLAY_LIGHT_SCRIM_VAR);

pub static OVERLAY_HOVER_VAR: ThemeVar<Color> =
    ThemeVar::alias("soft.overlay_hover", &kivact::style::color::HOVER_VAR);
pub const OVERLAY_HOVER: Color = Color::var(&OVERLAY_HOVER_VAR);

pub static OVERLAY_PRESSED_VAR: ThemeVar<Color> =
    ThemeVar::alias("soft.overlay_pressed", &kivact::style::color::PRESSED_VAR);
pub const OVERLAY_PRESSED: Color = Color::var(&OVERLAY_PRESSED_VAR);

pub static FOCUS_RING_VAR: ThemeVar<Color> =
    ThemeVar::alias("soft.focus_ring", &kivact::style::color::FOCUS_VAR);
pub(crate) const FOCUS_RING: Color = Color::var(&FOCUS_RING_VAR);

pub static INFO_VAR: ThemeVar<Color> = ThemeVar::new("soft.info", rgb(0x0074e2));
pub const INFO: Color = Color::var(&INFO_VAR);

pub static ON_INFO_VAR: ThemeVar<Color> = ThemeVar::new("soft.on_info", rgb(0xffffff));
pub const ON_INFO: Color = Color::var(&ON_INFO_VAR);

pub static INFO_MUTED_VAR: ThemeVar<Color> =
    ThemeVar::alias("soft.info_muted", &kivact::style::color::SELECTION_VAR);
pub const INFO_MUTED: Color = Color::var(&INFO_MUTED_VAR);

pub static ON_INFO_MUTED_VAR: ThemeVar<Color> = ThemeVar::new("soft.on_info_muted", rgb(0x003978));
pub const ON_INFO_MUTED: Color = Color::var(&ON_INFO_MUTED_VAR);

pub static TEXT_INFO_VAR: ThemeVar<Color> = ThemeVar::new("soft.text_info", rgb(0x0067ce));
pub const TEXT_INFO: Color = Color::var(&TEXT_INFO_VAR);

pub static SUCCESS_VAR: ThemeVar<Color> =
    ThemeVar::alias("soft.success", &kivact::style::color::SUCCESS_VAR);
pub const SUCCESS: Color = Color::var(&SUCCESS_VAR);

pub static ON_SUCCESS_VAR: ThemeVar<Color> = ThemeVar::new("soft.on_success", rgb(0xffffff));
pub const ON_SUCCESS: Color = Color::var(&ON_SUCCESS_VAR);

pub static SUCCESS_MUTED_VAR: ThemeVar<Color> = ThemeVar::new("soft.success_muted", rgb(0xbce0bb));
pub const SUCCESS_MUTED: Color = Color::var(&SUCCESS_MUTED_VAR);

pub static ON_SUCCESS_MUTED_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.on_success_muted", rgb(0x00490b));
pub const ON_SUCCESS_MUTED: Color = Color::var(&ON_SUCCESS_MUTED_VAR);

pub static TEXT_SUCCESS_VAR: ThemeVar<Color> = ThemeVar::new("soft.text_success", rgb(0x2f7d33));
pub const TEXT_SUCCESS: Color = Color::var(&TEXT_SUCCESS_VAR);

pub static WARNING_VAR: ThemeVar<Color> =
    ThemeVar::alias("soft.warning", &kivact::style::color::WARNING_VAR);
pub const WARNING: Color = Color::var(&WARNING_VAR);

pub static ON_WARNING_VAR: ThemeVar<Color> = ThemeVar::new("soft.on_warning", rgb(0x111111));
pub const ON_WARNING: Color = Color::var(&ON_WARNING_VAR);

pub static WARNING_MUTED_VAR: ThemeVar<Color> = ThemeVar::new("soft.warning_muted", rgb(0xfae19e));
pub const WARNING_MUTED: Color = Color::var(&WARNING_MUTED_VAR);

pub static ON_WARNING_MUTED_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.on_warning_muted", rgb(0x4b3900));
pub const ON_WARNING_MUTED: Color = Color::var(&ON_WARNING_MUTED_VAR);

pub static TEXT_WARNING_VAR: ThemeVar<Color> = ThemeVar::new("soft.text_warning", rgb(0x836700));
pub const TEXT_WARNING: Color = Color::var(&TEXT_WARNING_VAR);

pub static ERROR_VAR: ThemeVar<Color> =
    ThemeVar::alias("soft.error", &kivact::style::color::DANGER_VAR);
pub const ERROR: Color = Color::var(&ERROR_VAR);

pub static ON_ERROR_VAR: ThemeVar<Color> = ThemeVar::new("soft.on_error", rgb(0xffffff));
pub const ON_ERROR: Color = Color::var(&ON_ERROR_VAR);

pub static ERROR_MUTED_VAR: ThemeVar<Color> = ThemeVar::new("soft.error_muted", rgb(0xffc4be));
pub const ERROR_MUTED: Color = Color::var(&ERROR_MUTED_VAR);

pub static ON_ERROR_MUTED_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.on_error_muted", rgb(0x76000c));
pub const ON_ERROR_MUTED: Color = Color::var(&ON_ERROR_MUTED_VAR);

pub static TEXT_ERROR_VAR: ThemeVar<Color> = ThemeVar::new("soft.text_error", rgb(0xc0262c));
pub const TEXT_ERROR: Color = Color::var(&TEXT_ERROR_VAR);

pub static HUE_RED_BG_VAR: ThemeVar<Color> = ThemeVar::new("soft.hue_red_bg", rgb(0xffc4be));
pub const HUE_RED_BG: Color = Color::var(&HUE_RED_BG_VAR);

pub static HUE_RED_BORDER_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.hue_red_border", rgb(0xffaea7));
pub const HUE_RED_BORDER: Color = Color::var(&HUE_RED_BORDER_VAR);

pub static HUE_RED_ICON_VAR: ThemeVar<Color> = ThemeVar::new("soft.hue_red_icon", rgb(0x76000c));
pub const HUE_RED_ICON: Color = Color::var(&HUE_RED_ICON_VAR);

pub static HUE_RED_TEXT_VAR: ThemeVar<Color> = ThemeVar::new("soft.hue_red_text", rgb(0x76000c));
pub const HUE_RED_TEXT: Color = Color::var(&HUE_RED_TEXT_VAR);

pub static HUE_ORANGE_BG_VAR: ThemeVar<Color> = ThemeVar::new("soft.hue_orange_bg", rgb(0xffc7a1));
pub const HUE_ORANGE_BG: Color = Color::var(&HUE_ORANGE_BG_VAR);

pub static HUE_ORANGE_BORDER_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.hue_orange_border", rgb(0xffc7a1));
pub const HUE_ORANGE_BORDER: Color = Color::var(&HUE_ORANGE_BORDER_VAR);

pub static HUE_ORANGE_ICON_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.hue_orange_icon", rgb(0x622700));
pub const HUE_ORANGE_ICON: Color = Color::var(&HUE_ORANGE_ICON_VAR);

pub static HUE_ORANGE_TEXT_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.hue_orange_text", rgb(0x622700));
pub const HUE_ORANGE_TEXT: Color = Color::var(&HUE_ORANGE_TEXT_VAR);

pub static HUE_YELLOW_BG_VAR: ThemeVar<Color> = ThemeVar::new("soft.hue_yellow_bg", rgb(0xfae19e));
pub const HUE_YELLOW_BG: Color = Color::var(&HUE_YELLOW_BG_VAR);

pub static HUE_YELLOW_BORDER_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.hue_yellow_border", rgb(0xe3c36c));
pub const HUE_YELLOW_BORDER: Color = Color::var(&HUE_YELLOW_BORDER_VAR);

pub static HUE_YELLOW_ICON_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.hue_yellow_icon", rgb(0x4b3900));
pub const HUE_YELLOW_ICON: Color = Color::var(&HUE_YELLOW_ICON_VAR);

pub static HUE_YELLOW_TEXT_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.hue_yellow_text", rgb(0x4b3900));
pub const HUE_YELLOW_TEXT: Color = Color::var(&HUE_YELLOW_TEXT_VAR);

pub static HUE_GREEN_BG_VAR: ThemeVar<Color> = ThemeVar::new("soft.hue_green_bg", rgb(0xbce0bb));
pub const HUE_GREEN_BG: Color = Color::var(&HUE_GREEN_BG_VAR);

pub static HUE_GREEN_BORDER_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.hue_green_border", rgb(0xaad4a9));
pub const HUE_GREEN_BORDER: Color = Color::var(&HUE_GREEN_BORDER_VAR);

pub static HUE_GREEN_ICON_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.hue_green_icon", rgb(0x00490b));
pub const HUE_GREEN_ICON: Color = Color::var(&HUE_GREEN_ICON_VAR);

pub static HUE_GREEN_TEXT_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.hue_green_text", rgb(0x00490b));
pub const HUE_GREEN_TEXT: Color = Color::var(&HUE_GREEN_TEXT_VAR);

pub static HUE_TEAL_BG_VAR: ThemeVar<Color> = ThemeVar::new("soft.hue_teal_bg", rgb(0xa9e2d6));
pub const HUE_TEAL_BG: Color = Color::var(&HUE_TEAL_BG_VAR);

pub static HUE_TEAL_BORDER_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.hue_teal_border", rgb(0x90d7c8));
pub const HUE_TEAL_BORDER: Color = Color::var(&HUE_TEAL_BORDER_VAR);

pub static HUE_TEAL_ICON_VAR: ThemeVar<Color> = ThemeVar::new("soft.hue_teal_icon", rgb(0x00463d));
pub const HUE_TEAL_ICON: Color = Color::var(&HUE_TEAL_ICON_VAR);

pub static HUE_TEAL_TEXT_VAR: ThemeVar<Color> = ThemeVar::new("soft.hue_teal_text", rgb(0x00463d));
pub const HUE_TEAL_TEXT: Color = Color::var(&HUE_TEAL_TEXT_VAR);

pub static HUE_CYAN_BG_VAR: ThemeVar<Color> = ThemeVar::new("soft.hue_cyan_bg", rgb(0x9ae2f4));
pub const HUE_CYAN_BG: Color = Color::var(&HUE_CYAN_BG_VAR);

pub static HUE_CYAN_BORDER_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.hue_cyan_border", rgb(0x85d5e9));
pub const HUE_CYAN_BORDER: Color = Color::var(&HUE_CYAN_BORDER_VAR);

pub static HUE_CYAN_ICON_VAR: ThemeVar<Color> = ThemeVar::new("soft.hue_cyan_icon", rgb(0x004351));
pub const HUE_CYAN_ICON: Color = Color::var(&HUE_CYAN_ICON_VAR);

pub static HUE_CYAN_TEXT_VAR: ThemeVar<Color> = ThemeVar::new("soft.hue_cyan_text", rgb(0x004351));
pub const HUE_CYAN_TEXT: Color = Color::var(&HUE_CYAN_TEXT_VAR);

pub static HUE_BLUE_BG_VAR: ThemeVar<Color> = ThemeVar::new("soft.hue_blue_bg", rgb(0xb9d7ff));
pub const HUE_BLUE_BG: Color = Color::var(&HUE_BLUE_BG_VAR);

pub static HUE_BLUE_BORDER_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.hue_blue_border", rgb(0xa1caff));
pub const HUE_BLUE_BORDER: Color = Color::var(&HUE_BLUE_BORDER_VAR);

pub static HUE_BLUE_ICON_VAR: ThemeVar<Color> = ThemeVar::new("soft.hue_blue_icon", rgb(0x003978));
pub const HUE_BLUE_ICON: Color = Color::var(&HUE_BLUE_ICON_VAR);

pub static HUE_BLUE_TEXT_VAR: ThemeVar<Color> = ThemeVar::new("soft.hue_blue_text", rgb(0x003978));
pub const HUE_BLUE_TEXT: Color = Color::var(&HUE_BLUE_TEXT_VAR);

pub static HUE_PURPLE_BG_VAR: ThemeVar<Color> = ThemeVar::new("soft.hue_purple_bg", rgb(0xf7d5ff));
pub const HUE_PURPLE_BG: Color = Color::var(&HUE_PURPLE_BG_VAR);

pub static HUE_PURPLE_BORDER_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.hue_purple_border", rgb(0xf3bfff));
pub const HUE_PURPLE_BORDER: Color = Color::var(&HUE_PURPLE_BORDER_VAR);

pub static HUE_PURPLE_ICON_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.hue_purple_icon", rgb(0x5c0e6c));
pub const HUE_PURPLE_ICON: Color = Color::var(&HUE_PURPLE_ICON_VAR);

pub static HUE_PURPLE_TEXT_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.hue_purple_text", rgb(0x5c0e6c));
pub const HUE_PURPLE_TEXT: Color = Color::var(&HUE_PURPLE_TEXT_VAR);

pub static HUE_PINK_BG_VAR: ThemeVar<Color> = ThemeVar::new("soft.hue_pink_bg", rgb(0xffc0d7));
pub const HUE_PINK_BG: Color = Color::var(&HUE_PINK_BG_VAR);

pub static HUE_PINK_BORDER_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.hue_pink_border", rgb(0xffc0d7));
pub const HUE_PINK_BORDER: Color = Color::var(&HUE_PINK_BORDER_VAR);

pub static HUE_PINK_ICON_VAR: ThemeVar<Color> = ThemeVar::new("soft.hue_pink_icon", rgb(0x70003f));
pub const HUE_PINK_ICON: Color = Color::var(&HUE_PINK_ICON_VAR);

pub static HUE_PINK_TEXT_VAR: ThemeVar<Color> = ThemeVar::new("soft.hue_pink_text", rgb(0x70003f));
pub const HUE_PINK_TEXT: Color = Color::var(&HUE_PINK_TEXT_VAR);

pub static HUE_GRAY_BG_VAR: ThemeVar<Color> = ThemeVar::new("soft.hue_gray_bg", rgb(0xe2e2e2));
pub const HUE_GRAY_BG: Color = Color::var(&HUE_GRAY_BG_VAR);

pub static HUE_GRAY_BORDER_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.hue_gray_border", rgb(0xd4d4d4));
pub const HUE_GRAY_BORDER: Color = Color::var(&HUE_GRAY_BORDER_VAR);

pub static HUE_GRAY_ICON_VAR: ThemeVar<Color> = ThemeVar::new("soft.hue_gray_icon", rgb(0x474747));
pub const HUE_GRAY_ICON: Color = Color::var(&HUE_GRAY_ICON_VAR);

pub static HUE_GRAY_TEXT_VAR: ThemeVar<Color> = ThemeVar::new("soft.hue_gray_text", rgb(0x1b1b1b));
pub const HUE_GRAY_TEXT: Color = Color::var(&HUE_GRAY_TEXT_VAR);

pub static DATA_BLUE_VAR: ThemeVar<Color> = ThemeVar::new("soft.data_blue", rgb(0x0171e3));
pub const DATA_BLUE: Color = Color::var(&DATA_BLUE_VAR);

pub static DATA_CYAN_VAR: ThemeVar<Color> = ThemeVar::new("soft.data_cyan", rgb(0x0171a4));
pub const DATA_CYAN: Color = Color::var(&DATA_CYAN_VAR);

pub static DATA_ORANGE_VAR: ThemeVar<Color> = ThemeVar::new("soft.data_orange", rgb(0xeb6e00));
pub const DATA_ORANGE: Color = Color::var(&DATA_ORANGE_VAR);

pub static DATA_RED_VAR: ThemeVar<Color> = ThemeVar::new("soft.data_red", rgb(0xf5394f));
pub const DATA_RED: Color = Color::var(&DATA_RED_VAR);

pub static DATA_PURPLE_VAR: ThemeVar<Color> = ThemeVar::new("soft.data_purple", rgb(0x6b1efd));
pub const DATA_PURPLE: Color = Color::var(&DATA_PURPLE_VAR);

pub static DATA_TEAL_VAR: ThemeVar<Color> = ThemeVar::new("soft.data_teal", rgb(0x08a3a3));
pub const DATA_TEAL: Color = Color::var(&DATA_TEAL_VAR);

pub static DATA_GREEN_VAR: ThemeVar<Color> = ThemeVar::new("soft.data_green", rgb(0x0b991f));
pub const DATA_GREEN: Color = Color::var(&DATA_GREEN_VAR);

pub static DATA_BROWN_VAR: ThemeVar<Color> = ThemeVar::new("soft.data_brown", rgb(0x965e03));
pub const DATA_BROWN: Color = Color::var(&DATA_BROWN_VAR);

pub static DATA_PINK_VAR: ThemeVar<Color> = ThemeVar::new("soft.data_pink", rgb(0xf351c0));
pub const DATA_PINK: Color = Color::var(&DATA_PINK_VAR);

pub static DATA_INDIGO_VAR: ThemeVar<Color> = ThemeVar::new("soft.data_indigo", rgb(0x6f8aff));
pub const DATA_INDIGO: Color = Color::var(&DATA_INDIGO_VAR);

pub static DATA_NEUTRAL_VAR: ThemeVar<Color> = ThemeVar::new("soft.data_neutral", rgb(0x8494a3));
pub const DATA_NEUTRAL: Color = Color::var(&DATA_NEUTRAL_VAR);

pub static DATA_BLUE_RAMP_1_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_blue_ramp_1", rgb(0x02165e));
pub const DATA_BLUE_RAMP_1: Color = Color::var(&DATA_BLUE_RAMP_1_VAR);

pub static DATA_BLUE_RAMP_2_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_blue_ramp_2", rgb(0x004cbc));
pub const DATA_BLUE_RAMP_2: Color = Color::var(&DATA_BLUE_RAMP_2_VAR);

pub static DATA_BLUE_RAMP_3_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_blue_ramp_3", rgb(0x2694fe));
pub const DATA_BLUE_RAMP_3: Color = Color::var(&DATA_BLUE_RAMP_3_VAR);

pub static DATA_BLUE_RAMP_4_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_blue_ramp_4", rgb(0x78beff));
pub const DATA_BLUE_RAMP_4: Color = Color::var(&DATA_BLUE_RAMP_4_VAR);

pub static DATA_BLUE_RAMP_5_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_blue_ramp_5", rgb(0xdbecff));
pub const DATA_BLUE_RAMP_5: Color = Color::var(&DATA_BLUE_RAMP_5_VAR);

pub static DATA_SHAMROCK_RAMP_1_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_shamrock_ramp_1", rgb(0x0b603d));
pub const DATA_SHAMROCK_RAMP_1: Color = Color::var(&DATA_SHAMROCK_RAMP_1_VAR);

pub static DATA_SHAMROCK_RAMP_2_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_shamrock_ramp_2", rgb(0x138546));
pub const DATA_SHAMROCK_RAMP_2: Color = Color::var(&DATA_SHAMROCK_RAMP_2_VAR);

pub static DATA_SHAMROCK_RAMP_3_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_shamrock_ramp_3", rgb(0x24bb5e));
pub const DATA_SHAMROCK_RAMP_3: Color = Color::var(&DATA_SHAMROCK_RAMP_3_VAR);

pub static DATA_SHAMROCK_RAMP_4_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_shamrock_ramp_4", rgb(0x8ef7aa));
pub const DATA_SHAMROCK_RAMP_4: Color = Color::var(&DATA_SHAMROCK_RAMP_4_VAR);

pub static DATA_SHAMROCK_RAMP_5_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_shamrock_ramp_5", rgb(0xd6fee4));
pub const DATA_SHAMROCK_RAMP_5: Color = Color::var(&DATA_SHAMROCK_RAMP_5_VAR);

pub static DATA_ORANGE_RAMP_1_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_orange_ramp_1", rgb(0xa13f04));
pub const DATA_ORANGE_RAMP_1: Color = Color::var(&DATA_ORANGE_RAMP_1_VAR);

pub static DATA_ORANGE_RAMP_2_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_orange_ramp_2", rgb(0xd66100));
pub const DATA_ORANGE_RAMP_2: Color = Color::var(&DATA_ORANGE_RAMP_2_VAR);

pub static DATA_ORANGE_RAMP_3_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_orange_ramp_3", rgb(0xfd9537));
pub const DATA_ORANGE_RAMP_3: Color = Color::var(&DATA_ORANGE_RAMP_3_VAR);

pub static DATA_ORANGE_RAMP_4_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_orange_ramp_4", rgb(0xfdb876));
pub const DATA_ORANGE_RAMP_4: Color = Color::var(&DATA_ORANGE_RAMP_4_VAR);

pub static DATA_ORANGE_RAMP_5_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_orange_ramp_5", rgb(0xffe6cf));
pub const DATA_ORANGE_RAMP_5: Color = Color::var(&DATA_ORANGE_RAMP_5_VAR);

pub static DATA_PINK_RAMP_1_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_pink_ramp_1", rgb(0x8e1073));
pub const DATA_PINK_RAMP_1: Color = Color::var(&DATA_PINK_RAMP_1_VAR);

pub static DATA_PINK_RAMP_2_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_pink_ramp_2", rgb(0xd123a1));
pub const DATA_PINK_RAMP_2: Color = Color::var(&DATA_PINK_RAMP_2_VAR);

pub static DATA_PINK_RAMP_3_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_pink_ramp_3", rgb(0xf989d3));
pub const DATA_PINK_RAMP_3: Color = Color::var(&DATA_PINK_RAMP_3_VAR);

pub static DATA_PINK_RAMP_4_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_pink_ramp_4", rgb(0xfeade3));
pub const DATA_PINK_RAMP_4: Color = Color::var(&DATA_PINK_RAMP_4_VAR);

pub static DATA_PINK_RAMP_5_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_pink_ramp_5", rgb(0xfce3f4));
pub const DATA_PINK_RAMP_5: Color = Color::var(&DATA_PINK_RAMP_5_VAR);

pub static DATA_PURPLE_RAMP_1_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_purple_ramp_1", rgb(0x3e0697));
pub const DATA_PURPLE_RAMP_1: Color = Color::var(&DATA_PURPLE_RAMP_1_VAR);

pub static DATA_PURPLE_RAMP_2_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_purple_ramp_2", rgb(0x6b1efd));
pub const DATA_PURPLE_RAMP_2: Color = Color::var(&DATA_PURPLE_RAMP_2_VAR);

pub static DATA_PURPLE_RAMP_3_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_purple_ramp_3", rgb(0x9081ff));
pub const DATA_PURPLE_RAMP_3: Color = Color::var(&DATA_PURPLE_RAMP_3_VAR);

pub static DATA_PURPLE_RAMP_4_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_purple_ramp_4", rgb(0xb3b0fe));
pub const DATA_PURPLE_RAMP_4: Color = Color::var(&DATA_PURPLE_RAMP_4_VAR);

pub static DATA_PURPLE_RAMP_5_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_purple_ramp_5", rgb(0xe8e8fb));
pub const DATA_PURPLE_RAMP_5: Color = Color::var(&DATA_PURPLE_RAMP_5_VAR);

pub static DATA_RED_RAMP_1_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_red_ramp_1", rgb(0x9d0519));
pub const DATA_RED_RAMP_1: Color = Color::var(&DATA_RED_RAMP_1_VAR);

pub static DATA_RED_RAMP_2_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_red_ramp_2", rgb(0xd31130));
pub const DATA_RED_RAMP_2: Color = Color::var(&DATA_RED_RAMP_2_VAR);

pub static DATA_RED_RAMP_3_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_red_ramp_3", rgb(0xfb7d87));
pub const DATA_RED_RAMP_3: Color = Color::var(&DATA_RED_RAMP_3_VAR);

pub static DATA_RED_RAMP_4_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_red_ramp_4", rgb(0xffb2b8));
pub const DATA_RED_RAMP_4: Color = Color::var(&DATA_RED_RAMP_4_VAR);

pub static DATA_RED_RAMP_5_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_red_ramp_5", rgb(0xfee4e6));
pub const DATA_RED_RAMP_5: Color = Color::var(&DATA_RED_RAMP_5_VAR);

pub static DATA_TEAL_RAMP_1_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_teal_ramp_1", rgb(0x08767d));
pub const DATA_TEAL_RAMP_1: Color = Color::var(&DATA_TEAL_RAMP_1_VAR);

pub static DATA_TEAL_RAMP_2_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_teal_ramp_2", rgb(0x0c9293));
pub const DATA_TEAL_RAMP_2: Color = Color::var(&DATA_TEAL_RAMP_2_VAR);

pub static DATA_TEAL_RAMP_3_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_teal_ramp_3", rgb(0x0db7af));
pub const DATA_TEAL_RAMP_3: Color = Color::var(&DATA_TEAL_RAMP_3_VAR);

pub static DATA_TEAL_RAMP_4_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_teal_ramp_4", rgb(0x6ce6d8));
pub const DATA_TEAL_RAMP_4: Color = Color::var(&DATA_TEAL_RAMP_4_VAR);

pub static DATA_TEAL_RAMP_5_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_teal_ramp_5", rgb(0xd7fcf8));
pub const DATA_TEAL_RAMP_5: Color = Color::var(&DATA_TEAL_RAMP_5_VAR);

pub static DATA_YELLOW_RAMP_1_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_yellow_ramp_1", rgb(0x8a5001));
pub const DATA_YELLOW_RAMP_1: Color = Color::var(&DATA_YELLOW_RAMP_1_VAR);

pub static DATA_YELLOW_RAMP_2_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_yellow_ramp_2", rgb(0xd69804));
pub const DATA_YELLOW_RAMP_2: Color = Color::var(&DATA_YELLOW_RAMP_2_VAR);

pub static DATA_YELLOW_RAMP_3_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_yellow_ramp_3", rgb(0xfbce03));
pub const DATA_YELLOW_RAMP_3: Color = Color::var(&DATA_YELLOW_RAMP_3_VAR);

pub static DATA_YELLOW_RAMP_4_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_yellow_ramp_4", rgb(0xfcec85));
pub const DATA_YELLOW_RAMP_4: Color = Color::var(&DATA_YELLOW_RAMP_4_VAR);

pub static DATA_YELLOW_RAMP_5_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_yellow_ramp_5", rgb(0xfdf6ba));
pub const DATA_YELLOW_RAMP_5: Color = Color::var(&DATA_YELLOW_RAMP_5_VAR);

pub static DATA_GRAY_RAMP_1_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_gray_ramp_1", rgb(0x25363f));
pub const DATA_GRAY_RAMP_1: Color = Color::var(&DATA_GRAY_RAMP_1_VAR);

pub static DATA_GRAY_RAMP_2_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_gray_ramp_2", rgb(0x5d6c7b));
pub const DATA_GRAY_RAMP_2: Color = Color::var(&DATA_GRAY_RAMP_2_VAR);

pub static DATA_GRAY_RAMP_3_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_gray_ramp_3", rgb(0xafb9c4));
pub const DATA_GRAY_RAMP_3: Color = Color::var(&DATA_GRAY_RAMP_3_VAR);

pub static DATA_GRAY_RAMP_4_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_gray_ramp_4", rgb(0xccd3db));
pub const DATA_GRAY_RAMP_4: Color = Color::var(&DATA_GRAY_RAMP_4_VAR);

pub static DATA_GRAY_RAMP_5_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.data_gray_ramp_5", rgb(0xf1f4f7));
pub const DATA_GRAY_RAMP_5: Color = Color::var(&DATA_GRAY_RAMP_5_VAR);

pub static SYNTAX_BACKGROUND_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.syntax_background", rgb(0xffffff));
pub const SYNTAX_BACKGROUND: Color = Color::var(&SYNTAX_BACKGROUND_VAR);

pub static SYNTAX_ADDED_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.syntax_added", rgba(0xbce0bb, 64.0 / 255.0));
pub const SYNTAX_ADDED: Color = Color::var(&SYNTAX_ADDED_VAR);

pub static SYNTAX_REMOVED_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.syntax_removed", rgba(0xffc4be, 64.0 / 255.0));
pub const SYNTAX_REMOVED: Color = Color::var(&SYNTAX_REMOVED_VAR);

pub static SYNTAX_DIFF_HEADER_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.syntax_diff_header", Color::TRANSPARENT);
pub const SYNTAX_DIFF_HEADER: Color = Color::var(&SYNTAX_DIFF_HEADER_VAR);

pub static SYNTAX_ADDED_MARKER_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.syntax_added_marker", rgb(0x6a6a6a));
pub const SYNTAX_ADDED_MARKER: Color = Color::var(&SYNTAX_ADDED_MARKER_VAR);

pub static SYNTAX_REMOVED_MARKER_VAR: ThemeVar<Color> =
    ThemeVar::new("soft.syntax_removed_marker", rgb(0x6a6a6a));
pub const SYNTAX_REMOVED_MARKER: Color = Color::var(&SYNTAX_REMOVED_MARKER_VAR);
