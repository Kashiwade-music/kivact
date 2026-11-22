//! Role: The Astryx Neutral color value of each variable for each appearance.
//! `set_colors` sets them; the shadow and input colors are built from them.
use crate::color::{
    ACCENT_HOVER_VAR, ACCENT_MUTED_VAR, ACCENT_VAR, BG_BODY_VAR, BG_CARD_VAR,
    BG_ERROR_INVERTED_VAR, BG_INVERTED_VAR, BG_MUTED_VAR, BG_POPOVER_VAR, BG_SURFACE_VAR,
    BORDER_EMPHASIZED_VAR, BORDER_VAR, CHOICE_BORDER_HOVER_VAR, CHOICE_SURFACE_HOVER_VAR,
    DATA_BLUE_RAMP_1_VAR, DATA_BLUE_RAMP_2_VAR, DATA_BLUE_RAMP_3_VAR, DATA_BLUE_RAMP_4_VAR,
    DATA_BLUE_RAMP_5_VAR, DATA_BLUE_VAR, DATA_BROWN_VAR, DATA_CYAN_VAR, DATA_GRAY_RAMP_1_VAR,
    DATA_GRAY_RAMP_2_VAR, DATA_GRAY_RAMP_3_VAR, DATA_GRAY_RAMP_4_VAR, DATA_GRAY_RAMP_5_VAR,
    DATA_GREEN_VAR, DATA_INDIGO_VAR, DATA_NEUTRAL_VAR, DATA_ORANGE_RAMP_1_VAR,
    DATA_ORANGE_RAMP_2_VAR, DATA_ORANGE_RAMP_3_VAR, DATA_ORANGE_RAMP_4_VAR, DATA_ORANGE_RAMP_5_VAR,
    DATA_ORANGE_VAR, DATA_PINK_RAMP_1_VAR, DATA_PINK_RAMP_2_VAR, DATA_PINK_RAMP_3_VAR,
    DATA_PINK_RAMP_4_VAR, DATA_PINK_RAMP_5_VAR, DATA_PINK_VAR, DATA_PURPLE_RAMP_1_VAR,
    DATA_PURPLE_RAMP_2_VAR, DATA_PURPLE_RAMP_3_VAR, DATA_PURPLE_RAMP_4_VAR, DATA_PURPLE_RAMP_5_VAR,
    DATA_PURPLE_VAR, DATA_RED_RAMP_1_VAR, DATA_RED_RAMP_2_VAR, DATA_RED_RAMP_3_VAR,
    DATA_RED_RAMP_4_VAR, DATA_RED_RAMP_5_VAR, DATA_RED_VAR, DATA_SHAMROCK_RAMP_1_VAR,
    DATA_SHAMROCK_RAMP_2_VAR, DATA_SHAMROCK_RAMP_3_VAR, DATA_SHAMROCK_RAMP_4_VAR,
    DATA_SHAMROCK_RAMP_5_VAR, DATA_TEAL_RAMP_1_VAR, DATA_TEAL_RAMP_2_VAR, DATA_TEAL_RAMP_3_VAR,
    DATA_TEAL_RAMP_4_VAR, DATA_TEAL_RAMP_5_VAR, DATA_TEAL_VAR, DATA_YELLOW_RAMP_1_VAR,
    DATA_YELLOW_RAMP_2_VAR, DATA_YELLOW_RAMP_3_VAR, DATA_YELLOW_RAMP_4_VAR, DATA_YELLOW_RAMP_5_VAR,
    ERROR_MUTED_VAR, ERROR_VAR, FOCUS_RING_VAR, HUE_BLUE_BG_VAR, HUE_BLUE_BORDER_VAR,
    HUE_BLUE_ICON_VAR, HUE_BLUE_TEXT_VAR, HUE_CYAN_BG_VAR, HUE_CYAN_BORDER_VAR, HUE_CYAN_ICON_VAR,
    HUE_CYAN_TEXT_VAR, HUE_GRAY_BG_VAR, HUE_GRAY_BORDER_VAR, HUE_GRAY_ICON_VAR, HUE_GRAY_TEXT_VAR,
    HUE_GREEN_BG_VAR, HUE_GREEN_BORDER_VAR, HUE_GREEN_ICON_VAR, HUE_GREEN_TEXT_VAR,
    HUE_ORANGE_BG_VAR, HUE_ORANGE_BORDER_VAR, HUE_ORANGE_ICON_VAR, HUE_ORANGE_TEXT_VAR,
    HUE_PINK_BG_VAR, HUE_PINK_BORDER_VAR, HUE_PINK_ICON_VAR, HUE_PINK_TEXT_VAR, HUE_PURPLE_BG_VAR,
    HUE_PURPLE_BORDER_VAR, HUE_PURPLE_ICON_VAR, HUE_PURPLE_TEXT_VAR, HUE_RED_BG_VAR,
    HUE_RED_BORDER_VAR, HUE_RED_ICON_VAR, HUE_RED_TEXT_VAR, HUE_TEAL_BG_VAR, HUE_TEAL_BORDER_VAR,
    HUE_TEAL_ICON_VAR, HUE_TEAL_TEXT_VAR, HUE_YELLOW_BG_VAR, HUE_YELLOW_BORDER_VAR,
    HUE_YELLOW_ICON_VAR, HUE_YELLOW_TEXT_VAR, ICON_ACCENT_VAR, ICON_DISABLED_VAR, ICON_PRIMARY_VAR,
    ICON_SECONDARY_VAR, INFO_MUTED_VAR, INFO_VAR, NEUTRAL_VAR, ON_ACCENT_VAR, ON_DARK_VAR,
    ON_ERROR_MUTED_VAR, ON_ERROR_VAR, ON_INFO_MUTED_VAR, ON_INFO_VAR, ON_LIGHT_VAR,
    ON_SUCCESS_MUTED_VAR, ON_SUCCESS_VAR, ON_WARNING_MUTED_VAR, ON_WARNING_VAR, OVERLAY_HOVER_VAR,
    OVERLAY_LIGHT_SCRIM_VAR, OVERLAY_PRESSED_VAR, OVERLAY_SCRIM_VAR, SHADOW_VAR, SKELETON_VAR,
    SUCCESS_MUTED_VAR, SUCCESS_VAR, SWITCH_OFF_HOVER_VAR, SYNTAX_ADDED_MARKER_VAR,
    SYNTAX_ADDED_VAR, SYNTAX_BACKGROUND_VAR, SYNTAX_DIFF_HEADER_VAR, SYNTAX_REMOVED_MARKER_VAR,
    SYNTAX_REMOVED_VAR, TEXT_ACCENT_VAR, TEXT_DISABLED_VAR, TEXT_ERROR_VAR, TEXT_INFO_VAR,
    TEXT_PRIMARY_VAR, TEXT_SECONDARY_VAR, TEXT_SUCCESS_VAR, TEXT_WARNING_VAR, TINT_HOVER_VAR,
    TRACK_VAR, WARNING_MUTED_VAR, WARNING_VAR, WHITE_VAR,
};
use kivact::style::{Color, Theme, ThemeMode, rgb, rgba};

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
