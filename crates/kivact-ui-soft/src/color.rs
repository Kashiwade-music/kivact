//! Role: Define the Astryx Neutral palette for light and dark appearances.
use kivact::style::Color;
use kivact::style::rgb;
use kivact::style::rgba;

/// Neutral surface, semantic, categorical, data and syntax colors.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Colors {
    /// Application canvas behind content surfaces.
    pub bg_body: Color,
    /// Default content surface background.
    pub bg_surface: Color,
    /// Card surface background.
    pub bg_card: Color,
    /// Floating popover surface background.
    pub bg_popover: Color,
    /// Subdued background for secondary regions.
    pub bg_muted: Color,
    /// High-contrast inverted surface background.
    pub bg_inverted: Color,
    /// Inverted error surface background.
    pub bg_error_inverted: Color,
    /// Translucent neutral fill for secondary controls and placeholders.
    pub neutral: Color,
    /// Primary action and selection fill.
    pub accent: Color,
    /// Primary action fill on hover.
    pub accent_hover: Color,
    /// Hover border for checkboxes and radio indicators.
    pub choice_border_hover: Color,
    /// Hover surface for checkboxes and radio indicators.
    pub choice_surface_hover: Color,
    /// Hover track fill for an unchecked switch.
    pub switch_off_hover: Color,
    /// Subdued accent background.
    pub accent_muted: Color,
    /// Foreground for content over the accent fill.
    pub on_accent: Color,
    /// Foreground for content over dark media or surfaces.
    pub on_dark: Color,
    /// Foreground for content over light media or surfaces.
    pub on_light: Color,
    /// Text color for primary content.
    pub text_primary: Color,
    /// Text color for secondary content.
    pub text_secondary: Color,
    /// Text color for disabled content.
    pub text_disabled: Color,
    /// Text color for accented content.
    pub text_accent: Color,
    /// Icon color for primary content.
    pub icon_primary: Color,
    /// Icon color for secondary content.
    pub icon_secondary: Color,
    /// Icon color for disabled content.
    pub icon_disabled: Color,
    /// Icon color for accented content.
    pub icon_accent: Color,
    /// Default low-emphasis separator and outline color.
    pub border: Color,
    /// Stronger outline for emphasized boundaries.
    pub border_emphasized: Color,
    /// Placeholder fill for loading skeletons.
    pub skeleton: Color,
    /// Unfilled slider track color.
    pub track: Color,
    /// Base color for surface shadow effects.
    pub shadow: Color,
    /// Tint applied to hovered surfaces.
    pub tint_hover: Color,
    /// Scrim behind modal or overlaid content.
    pub overlay_scrim: Color,
    /// Light scrim for media overlays in either appearance.
    pub overlay_light_scrim: Color,
    /// Translucent hover layer over an existing surface.
    pub overlay_hover: Color,
    /// Translucent pressed layer over an existing surface.
    pub overlay_pressed: Color,
    /// Outline color identifying keyboard focus.
    pub focus_ring: Color,
    /// Solid fill for informational status indicators.
    pub info: Color,
    /// Foreground on the solid info fill.
    pub on_info: Color,
    /// Subdued background for informational messages.
    pub info_muted: Color,
    /// Foreground on the subdued info background.
    pub on_info_muted: Color,
    /// Text color for informational status messages.
    pub text_info: Color,
    /// Solid fill for successful status indicators.
    pub success: Color,
    /// Foreground on the solid success fill.
    pub on_success: Color,
    /// Subdued background for successful messages.
    pub success_muted: Color,
    /// Foreground on the subdued success background.
    pub on_success_muted: Color,
    /// Text color for successful status messages.
    pub text_success: Color,
    /// Solid fill for warning status indicators.
    pub warning: Color,
    /// Foreground on the solid warning fill.
    pub on_warning: Color,
    /// Subdued background for warning messages.
    pub warning_muted: Color,
    /// Foreground on the subdued warning background.
    pub on_warning_muted: Color,
    /// Text color for warning status messages.
    pub text_warning: Color,
    /// Solid fill for error status indicators.
    pub error: Color,
    /// Foreground on the solid error fill.
    pub on_error: Color,
    /// Subdued background for error messages.
    pub error_muted: Color,
    /// Foreground on the subdued error background.
    pub on_error_muted: Color,
    /// Text color for error status messages.
    pub text_error: Color,
    /// Coordinated categorical background, border, icon and text palettes.
    pub hue: Hues,
    /// Chart categories and ordered sequential color ramps.
    pub data: DataColors,
    /// Code-token and diff-highlighting palette.
    pub syntax: SyntaxColors,
}

impl Colors {
    pub(super) fn light() -> Self {
        Self {
            bg_body: rgb(0xf1f1f1),
            bg_surface: rgb(0xffffff),
            bg_card: rgb(0xffffff),
            bg_popover: rgb(0xffffff),
            bg_muted: rgb(0xf1f1f1),
            bg_inverted: rgb(0x0a1317),
            bg_error_inverted: rgb(0xaa071e),
            neutral: rgba(0x000000, 15.0 / 255.0),
            accent: rgb(0x1b1b1b),
            accent_hover: rgb(0x171717),
            choice_border_hover: rgb(0xaaaaaa),
            choice_surface_hover: rgb(0xf2f2f2),
            switch_off_hover: rgb(0xc9c9c9),
            accent_muted: rgb(0xf1f1f1),
            on_accent: rgb(0xffffff),
            on_dark: rgb(0xffffff),
            on_light: rgb(0x111111),
            text_primary: rgb(0x000000),
            text_secondary: rgb(0x474747),
            text_disabled: rgb(0x919191),
            text_accent: rgb(0x1b1b1b),
            icon_primary: rgb(0x000000),
            icon_secondary: rgb(0x6a6a6a),
            icon_disabled: rgb(0x919191),
            icon_accent: rgb(0x1b1b1b),
            border: rgba(0x000000, 20.0 / 255.0),
            border_emphasized: rgb(0xd4d4d4),
            skeleton: rgb(0xf1f1f1),
            track: rgb(0xccd3db),
            shadow: rgba(0x000000, 26.0 / 255.0),
            tint_hover: rgb(0x000000),
            overlay_light_scrim: rgba(0xffffff, 0.6),
            overlay_scrim: rgba(0x000000, 128.0 / 255.0),
            overlay_hover: rgba(0x000000, 13.0 / 255.0),
            overlay_pressed: rgba(0x000000, 26.0 / 255.0),
            focus_ring: rgb(0x1b1b1b),
            info: rgb(0x0074e2),
            on_info: rgb(0xffffff),
            info_muted: rgb(0xb9d7ff),
            on_info_muted: rgb(0x003978),
            text_info: rgb(0x0067ce),
            success: rgb(0x198100),
            on_success: rgb(0xffffff),
            success_muted: rgb(0xbce0bb),
            on_success_muted: rgb(0x00490b),
            text_success: rgb(0x2f7d33),
            warning: rgb(0xffce2f),
            on_warning: rgb(0x111111),
            warning_muted: rgb(0xfae19e),
            on_warning_muted: rgb(0x4b3900),
            text_warning: rgb(0x836700),
            error: rgb(0xc9303a),
            on_error: rgb(0xffffff),
            error_muted: rgb(0xffc4be),
            on_error_muted: rgb(0x76000c),
            text_error: rgb(0xc0262c),
            hue: Hues::light(),
            data: DataColors::new(false),
            syntax: SyntaxColors::light(),
        }
    }

    pub(super) fn dark() -> Self {
        Self {
            bg_body: rgb(0x1b1b1b),
            bg_surface: rgb(0x262626),
            bg_card: rgb(0x1b1b1b),
            bg_popover: rgb(0x1b1b1b),
            bg_muted: rgb(0x1b1b1b),
            bg_inverted: rgb(0xffffff),
            bg_error_inverted: rgb(0xe3193b),
            neutral: rgba(0xffffff, 26.0 / 255.0),
            accent: rgb(0xf1f1f1),
            accent_hover: rgb(0xf3f3f3),
            choice_border_hover: rgb(0x757575),
            choice_surface_hover: rgb(0x313131),
            switch_off_hover: rgb(0x5b5b5b),
            accent_muted: rgb(0x262626),
            on_accent: rgb(0x111111),
            on_dark: rgb(0xffffff),
            on_light: rgb(0x111111),
            text_primary: rgb(0xffffff),
            text_secondary: rgb(0x9e9e9e),
            text_disabled: rgb(0x525252),
            text_accent: rgb(0xf1f1f1),
            icon_primary: rgb(0xffffff),
            icon_secondary: rgb(0x9e9e9e),
            icon_disabled: rgb(0x525252),
            icon_accent: rgb(0xf1f1f1),
            border: rgba(0xffffff, 26.0 / 255.0),
            border_emphasized: rgb(0x525252),
            skeleton: rgb(0x525252),
            track: rgb(0x5a5e66),
            shadow: rgba(0x000000, 77.0 / 255.0),
            tint_hover: rgb(0xffffff),
            overlay_light_scrim: rgba(0xffffff, 0.6),
            overlay_scrim: rgba(0x000000, 204.0 / 255.0),
            overlay_hover: rgba(0xffffff, 13.0 / 255.0),
            overlay_pressed: rgba(0xffffff, 26.0 / 255.0),
            focus_ring: rgb(0xf1f1f1),
            info: rgb(0x6d9cfe),
            on_info: rgb(0x111111),
            info_muted: rgba(0x88bcff, 61.0 / 255.0),
            on_info_muted: rgb(0xa1caff),
            text_info: rgb(0xa1caff),
            success: rgb(0x64af4c),
            on_success: rgb(0x111111),
            success_muted: rgba(0x90ca90, 61.0 / 255.0),
            on_success_muted: rgb(0x90ca90),
            text_success: rgb(0x90ca90),
            warning: rgb(0xffce2f),
            on_warning: rgb(0x111111),
            warning_muted: rgba(0xe2b623, 61.0 / 255.0),
            on_warning_muted: rgb(0xeec448),
            text_warning: rgb(0xeec448),
            error: rgb(0xff705d),
            on_error: rgb(0x111111),
            error_muted: rgba(0xff9890, 61.0 / 255.0),
            on_error_muted: rgb(0xffaea7),
            text_error: rgb(0xffaea7),
            hue: Hues::dark(),
            data: DataColors::new(true),
            syntax: SyntaxColors::dark(),
        }
    }
}

/// Colors associated with one categorical hue.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HueColors {
    /// Subdued background for this categorical hue.
    pub bg: Color,
    /// Outline color paired with this hue background.
    pub border: Color,
    /// Icon foreground for this categorical hue.
    pub icon: Color,
    /// Text foreground for this categorical hue.
    pub text: Color,
}

/// Neutral categorical hue palettes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hues {
    /// Red categorical palette with coordinated surface and foreground colors.
    pub red: HueColors,
    /// Orange categorical palette with coordinated surface and foreground colors.
    pub orange: HueColors,
    /// Yellow categorical palette with coordinated surface and foreground colors.
    pub yellow: HueColors,
    /// Green categorical palette with coordinated surface and foreground colors.
    pub green: HueColors,
    /// Teal categorical palette with coordinated surface and foreground colors.
    pub teal: HueColors,
    /// Cyan categorical palette with coordinated surface and foreground colors.
    pub cyan: HueColors,
    /// Blue categorical palette with coordinated surface and foreground colors.
    pub blue: HueColors,
    /// Purple categorical palette with coordinated surface and foreground colors.
    pub purple: HueColors,
    /// Pink categorical palette with coordinated surface and foreground colors.
    pub pink: HueColors,
    /// Gray categorical palette with coordinated surface and foreground colors.
    pub gray: HueColors,
}

impl Hues {
    fn light() -> Self {
        Self {
            red: HueColors {
                bg: rgb(0xffc4be),
                border: rgb(0xffaea7),
                icon: rgb(0x76000c),
                text: rgb(0x76000c),
            },
            orange: HueColors {
                bg: rgb(0xffc7a1),
                border: rgb(0xffc7a1),
                icon: rgb(0x622700),
                text: rgb(0x622700),
            },
            yellow: HueColors {
                bg: rgb(0xfae19e),
                border: rgb(0xe3c36c),
                icon: rgb(0x4b3900),
                text: rgb(0x4b3900),
            },
            green: HueColors {
                bg: rgb(0xbce0bb),
                border: rgb(0xaad4a9),
                icon: rgb(0x00490b),
                text: rgb(0x00490b),
            },
            teal: HueColors {
                bg: rgb(0xa9e2d6),
                border: rgb(0x90d7c8),
                icon: rgb(0x00463d),
                text: rgb(0x00463d),
            },
            cyan: HueColors {
                bg: rgb(0x9ae2f4),
                border: rgb(0x85d5e9),
                icon: rgb(0x004351),
                text: rgb(0x004351),
            },
            blue: HueColors {
                bg: rgb(0xb9d7ff),
                border: rgb(0xa1caff),
                icon: rgb(0x003978),
                text: rgb(0x003978),
            },
            purple: HueColors {
                bg: rgb(0xf7d5ff),
                border: rgb(0xf3bfff),
                icon: rgb(0x5c0e6c),
                text: rgb(0x5c0e6c),
            },
            pink: HueColors {
                bg: rgb(0xffc0d7),
                border: rgb(0xffc0d7),
                icon: rgb(0x70003f),
                text: rgb(0x70003f),
            },
            gray: HueColors {
                bg: rgb(0xe2e2e2),
                border: rgb(0xd4d4d4),
                icon: rgb(0x474747),
                text: rgb(0x1b1b1b),
            },
        }
    }

    fn dark() -> Self {
        Self {
            red: HueColors {
                bg: rgb(0x5b2b28),
                border: rgb(0xfa6762),
                icon: rgb(0xff9890),
                text: rgb(0xffaea7),
            },
            orange: HueColors {
                bg: rgb(0x503424),
                border: rgb(0xdf843f),
                icon: rgb(0xff9e55),
                text: rgb(0xffb37e),
            },
            yellow: HueColors {
                bg: rgb(0x453a1c),
                border: rgb(0xc29900),
                icon: rgb(0xe2b623),
                text: rgb(0xeec448),
            },
            green: HueColors {
                bg: rgb(0x2b422b),
                border: rgb(0x6ab26b),
                icon: rgb(0x90ca90),
                text: rgb(0x90ca90),
            },
            teal: HueColors {
                bg: rgb(0x28413c),
                border: rgb(0x4fb1a0),
                icon: rgb(0x81c9bb),
                text: rgb(0x90d7c8),
            },
            cyan: HueColors {
                bg: rgb(0x274046),
                border: rgb(0x49adc4),
                icon: rgb(0x71c7dd),
                text: rgb(0x85d5e9),
            },
            blue: HueColors {
                bg: rgb(0x253c5a),
                border: rgb(0x5aa0f8),
                icon: rgb(0x88bcff),
                text: rgb(0xa1caff),
            },
            purple: HueColors {
                bg: rgb(0x4a2f51),
                border: rgb(0xd885eb),
                icon: rgb(0xe496f6),
                text: rgb(0xeaacf8),
            },
            pink: HueColors {
                bg: rgb(0x572b3d),
                border: rgb(0xfc78b1),
                icon: rgb(0xfd92bd),
                text: rgb(0xffa9ca),
            },
            gray: HueColors {
                bg: rgb(0x303030),
                border: rgb(0x262626),
                icon: rgb(0x9e9e9e),
                text: rgb(0xd4d4d4),
            },
        }
    }
}

/// Neutral syntax and diff colors.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SyntaxColors {
    /// Language keywords and control-flow words.
    pub keyword: Color,
    /// Type names in highlighted source code.
    pub r#type: Color,
    /// Quoted strings and string literals.
    pub string: Color,
    /// Source-code comments.
    pub comment: Color,
    /// Arithmetic, logical and other operator tokens.
    pub operator: Color,
    /// Source delimiters and punctuation.
    pub punctuation: Color,
    /// Numeric literals in source code.
    pub number: Color,
    /// Named constants in source code.
    pub constant: Color,
    /// Function names in highlighted source code.
    pub function: Color,
    /// Variable names in highlighted source code.
    pub variable: Color,
    /// Markup tag names.
    pub tag: Color,
    /// Markup attribute names.
    pub attribute: Color,
    /// Object property and member names.
    pub property: Color,
    /// Background behind highlighted source code.
    pub background: Color,
    /// Background highlighting for added diff lines.
    pub added: Color,
    /// Background highlighting for removed diff lines.
    pub removed: Color,
    /// Diff hunk-header highlighting.
    pub diff_header: Color,
    /// Foreground for the added-line marker in a diff.
    pub added_marker: Color,
    /// Foreground for the removed-line marker in a diff.
    pub removed_marker: Color,
}

impl SyntaxColors {
    fn light() -> Self {
        Self {
            keyword: rgb(0x6b187c),
            r#type: rgb(0x6b187c),
            string: rgb(0x0b5615),
            comment: rgb(0x6a6a6a),
            operator: rgb(0x6a6a6a),
            punctuation: rgb(0x6a6a6a),
            number: rgb(0x733100),
            constant: rgb(0x733100),
            function: rgb(0x00458c),
            variable: rgb(0x111111),
            tag: rgb(0x8a0011),
            attribute: rgb(0x584400),
            property: rgb(0x005348),
            background: rgb(0xffffff),
            added: rgba(0xbce0bb, 64.0 / 255.0),
            removed: rgba(0xffc4be, 64.0 / 255.0),
            diff_header: Color::TRANSPARENT,
            added_marker: rgb(0x6a6a6a),
            removed_marker: rgb(0x6a6a6a),
        }
    }

    fn dark() -> Self {
        Self {
            keyword: rgb(0xefa8ff),
            r#type: rgb(0xefa8ff),
            string: rgb(0xa4d6a3),
            comment: rgb(0x9e9e9e),
            operator: rgb(0x9e9e9e),
            punctuation: rgb(0x9e9e9e),
            number: rgb(0xffb37e),
            constant: rgb(0xffb37e),
            function: rgb(0xa1caff),
            variable: rgb(0xe2e2e2),
            tag: rgb(0xffaea7),
            attribute: rgb(0xeec448),
            property: rgb(0x90d7c8),
            background: rgb(0x111111),
            added: rgba(0x90ca90, 15.0 / 255.0),
            removed: rgba(0xff9890, 15.0 / 255.0),
            diff_header: Color::TRANSPARENT,
            added_marker: rgb(0x9e9e9e),
            removed_marker: rgb(0x9e9e9e),
        }
    }
}

/// Categorical data colors and five-stop ramps, darkest stop first.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DataColors {
    /// Blue category color for data series.
    pub blue: Color,
    /// Cyan category color for data series.
    pub cyan: Color,
    /// Orange category color for data series.
    pub orange: Color,
    /// Red category color for data series.
    pub red: Color,
    /// Purple category color for data series.
    pub purple: Color,
    /// Teal category color for data series.
    pub teal: Color,
    /// Green category color for data series.
    pub green: Color,
    /// Brown category color for data series.
    pub brown: Color,
    /// Pink category color for data series.
    pub pink: Color,
    /// Indigo category color for data series.
    pub indigo: Color,
    /// Neutral category color for data series.
    pub neutral: Color,
    /// Five-stop blue data ramp, ordered from darkest to lightest.
    pub blue_ramp: [Color; 5],
    /// Five-stop shamrock data ramp, ordered from darkest to lightest.
    pub shamrock_ramp: [Color; 5],
    /// Five-stop orange data ramp, ordered from darkest to lightest.
    pub orange_ramp: [Color; 5],
    /// Five-stop pink data ramp, ordered from darkest to lightest.
    pub pink_ramp: [Color; 5],
    /// Five-stop purple data ramp, ordered from darkest to lightest.
    pub purple_ramp: [Color; 5],
    /// Five-stop red data ramp, ordered from darkest to lightest.
    pub red_ramp: [Color; 5],
    /// Five-stop teal data ramp, ordered from darkest to lightest.
    pub teal_ramp: [Color; 5],
    /// Five-stop yellow data ramp, ordered from darkest to lightest.
    pub yellow_ramp: [Color; 5],
    /// Five-stop gray data ramp, ordered from darkest to lightest.
    pub gray_ramp: [Color; 5],
}

impl DataColors {
    fn new(dark: bool) -> Self {
        Self {
            blue: rgb(0x0171e3),
            cyan: rgb(0x0171a4),
            orange: rgb(0xeb6e00),
            red: rgb(0xf5394f),
            purple: rgb(0x6b1efd),
            teal: rgb(0x08a3a3),
            green: rgb(0x0b991f),
            brown: rgb(0x965e03),
            pink: rgb(0xf351c0),
            indigo: rgb(0x6f8aff),
            neutral: if dark { rgb(0x8c939b) } else { rgb(0x8494a3) },
            blue_ramp: [
                rgb(0x02165e),
                rgb(0x004cbc),
                rgb(0x2694fe),
                rgb(0x78beff),
                rgb(0xdbecff),
            ],
            shamrock_ramp: [
                rgb(0x0b603d),
                rgb(0x138546),
                rgb(0x24bb5e),
                rgb(0x8ef7aa),
                rgb(0xd6fee4),
            ],
            orange_ramp: [
                rgb(0xa13f04),
                rgb(0xd66100),
                rgb(0xfd9537),
                rgb(0xfdb876),
                rgb(0xffe6cf),
            ],
            pink_ramp: [
                rgb(0x8e1073),
                rgb(0xd123a1),
                rgb(0xf989d3),
                rgb(0xfeade3),
                rgb(0xfce3f4),
            ],
            purple_ramp: [
                rgb(0x3e0697),
                rgb(0x6b1efd),
                rgb(0x9081ff),
                rgb(0xb3b0fe),
                rgb(0xe8e8fb),
            ],
            red_ramp: [
                rgb(0x9d0519),
                rgb(0xd31130),
                rgb(0xfb7d87),
                rgb(0xffb2b8),
                rgb(0xfee4e6),
            ],
            teal_ramp: [
                rgb(0x08767d),
                rgb(0x0c9293),
                rgb(0x0db7af),
                rgb(0x6ce6d8),
                rgb(0xd7fcf8),
            ],
            yellow_ramp: [
                rgb(0x8a5001),
                rgb(0xd69804),
                rgb(0xfbce03),
                rgb(0xfcec85),
                rgb(0xfdf6ba),
            ],
            gray_ramp: gray_data_ramp(dark),
        }
    }
}

fn gray_data_ramp(dark: bool) -> [Color; 5] {
    if dark {
        [
            rgb(0x333338),
            rgb(0x666a72),
            rgb(0xb2b8be),
            rgb(0xd0d3d6),
            rgb(0xf2f4f6),
        ]
    } else {
        [
            rgb(0x25363f),
            rgb(0x5d6c7b),
            rgb(0xafb9c4),
            rgb(0xccd3db),
            rgb(0xf1f4f7),
        ]
    }
}
