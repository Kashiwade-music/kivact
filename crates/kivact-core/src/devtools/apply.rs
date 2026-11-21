//! Role: Apply DevTools overrides and emulation to styles, themes and texts.

use std::panic::Location;

use kivact_devtools_protocol as p;
use kivact_runtime::Owner;
use kivact_runtime::driver::Runtime;
use kivact_style::ThemeVar;
use kivact_style::internal::style_property;
use kivact_style::{Color, LineHeight, Style, Theme};

use kivact_types::NodeId;

/// The longhands a shorthand override sets, each with the value it was given.
const SHORTHANDS: [(&str, &[&str]); 6] = [
    (
        "padding",
        &[
            "padding_top",
            "padding_right",
            "padding_bottom",
            "padding_left",
        ],
    ),
    (
        "margin",
        &["margin_top", "margin_right", "margin_bottom", "margin_left"],
    ),
    ("gap", &["row_gap", "column_gap"]),
    (
        "border_width",
        &[
            "border_top_width",
            "border_right_width",
            "border_bottom_width",
            "border_left_width",
        ],
    ),
    (
        "border_color",
        &[
            "border_top_color",
            "border_right_color",
            "border_bottom_color",
            "border_left_color",
        ],
    ),
    (
        "border_radius",
        &[
            "border_top_left_radius",
            "border_top_right_radius",
            "border_bottom_right_radius",
            "border_bottom_left_radius",
        ],
    ),
];

/// Read one override into a style, or name the property it does not know.
/// Each value is read by the property's entry in kivact-style's table.
pub(crate) fn override_style(property: &str, value: &str) -> Result<Style, String> {
    match SHORTHANDS.iter().find(|(name, _)| *name == property) {
        Some((_, longhands)) => read_all(property, longhands, value),
        None => read_all(property, &[property], value),
    }
}

/// Read `value` as each of `names`, the longhands of `property`, into one style.
fn read_all(property: &str, names: &[&str], value: &str) -> Result<Style, String> {
    names
        .iter()
        .try_fold(Style::default(), |style, name| -> Result<Style, String> {
            let definition =
                style_property(name).ok_or_else(|| format!("unknown property: {property}"))?;
            let set =
                (definition.parse)(value).map_err(|message| format!("{property}: {message}"))?;
            Ok(style.merge(&set))
        })
}

fn number(value: &str) -> Result<f64, String> {
    value
        .trim()
        .parse::<f64>()
        .map_err(|_| format!("expected a number, got {value}"))
}

fn color(value: &str) -> Result<Color, String> {
    if let Some(var) = color_var(value.trim()) {
        return Ok(Color::var(var));
    }
    Color::css(value).map_err(|_| format!("expected a color, got {value}"))
}

/// The overrides of `id`: call-site overrides first, then node overrides, so
/// the node wins.
pub(crate) fn node_overrides(
    all: &[p::StyleOverride],
    id: NodeId,
    sites: &[&'static Location<'static>],
) -> Vec<(String, String)> {
    let mut by_site = Vec::new();
    let mut by_node = Vec::new();
    for entry in all {
        match &entry.scope {
            p::OverrideScope::CallSite { location } => {
                if sites.iter().any(|site| same_location(site, location)) {
                    by_site.push((entry.property.clone(), entry.value.clone()));
                }
            }
            p::OverrideScope::Node { node } if *node == id.0 => {
                by_node.push((entry.property.clone(), entry.value.clone()));
            }
            _ => {}
        }
    }
    by_site.extend(by_node);
    by_site
}

fn same_location(site: &'static Location<'static>, location: &p::SourceLocation) -> bool {
    site.file() == location.file && site.line() == location.line && site.column() == location.column
}

/// `at`, then the creation locations of `owner` and its ancestors, with a
/// repeated location taken once.
pub(crate) fn call_sites(
    rt: &Runtime,
    owner: Owner,
    at: Option<&'static Location<'static>>,
) -> Vec<&'static Location<'static>> {
    let mut sites: Vec<&'static Location<'static>> = Vec::new();
    if let Some(at) = at {
        push_site(&mut sites, at);
    }
    let mut current = Some(owner.id());
    while let Some(id) = current {
        let Some(info) = rt.owner_info(id) else {
            break;
        };
        push_site(&mut sites, info.created_at);
        current = info.parent;
    }
    sites
}

fn push_site(sites: &mut Vec<&'static Location<'static>>, at: &'static Location<'static>) {
    if sites.last().is_none_or(|previous| {
        previous.file() != at.file()
            || previous.line() != at.line()
            || previous.column() != at.column()
    }) {
        sites.push(at);
    }
}

/// Apply the theme token overrides of `all` to `theme`.
pub(crate) fn apply_token_overrides(theme: Theme, all: &[p::StyleOverride]) -> Theme {
    let mut theme = theme;
    for entry in all {
        let p::OverrideScope::Token { token } = &entry.scope else {
            continue;
        };
        theme = apply_token(theme, token, &entry.value);
    }
    theme
}

fn apply_token(theme: Theme, token: &str, value: &str) -> Theme {
    if let Some(var) = color_var(token) {
        let Ok(color) = color(value) else {
            return theme;
        };
        return theme.set(var, color);
    }
    if let Some(var) = length_var(token) {
        let Ok(number) = number(value) else {
            return theme;
        };
        return theme.set(var, number);
    }
    if let Some(var) = weight_var(token) {
        let Ok(number) = number(value) else {
            return theme;
        };
        return theme.set(var, number as u16);
    }
    if token == "text.line_height" {
        return match line_height(value) {
            Some(line_height) => theme.set(&kivact_style::font::LINE_HEIGHT_VAR, line_height),
            None => theme,
        };
    }
    apply_scalar(theme, token, value)
}

fn line_height(value: &str) -> Option<LineHeight> {
    if let Some(px) = value.trim().strip_suffix("px") {
        px.trim().parse::<f64>().ok().map(LineHeight::Px)
    } else {
        number(value).ok().map(LineHeight::Scale)
    }
}

fn apply_scalar(theme: Theme, token: &str, value: &str) -> Theme {
    let Ok(value) = number(value) else {
        return theme;
    };
    match token {
        "focus_ring" => theme.with_focus_ring(value),
        "focus_ring_offset" => theme.with_focus_ring_offset(value),
        _ => theme,
    }
}

/// The color theme variables a token name stands for.
const COLOR_TOKENS: [(&str, &ThemeVar<Color>); 17] = [
    ("surface", &kivact_style::color::SURFACE_VAR),
    ("surface_alt", &kivact_style::color::SURFACE_ALT_VAR),
    ("text", &kivact_style::color::TEXT_VAR),
    ("text_muted", &kivact_style::color::TEXT_MUTED_VAR),
    ("accent", &kivact_style::color::ACCENT_VAR),
    ("accent_text", &kivact_style::color::ACCENT_TEXT_VAR),
    ("border", &kivact_style::color::BORDER_VAR),
    ("focus", &kivact_style::color::FOCUS_VAR),
    ("danger", &kivact_style::color::DANGER_VAR),
    ("success", &kivact_style::color::SUCCESS_VAR),
    ("warning", &kivact_style::color::WARNING_VAR),
    ("overlay", &kivact_style::color::OVERLAY_VAR),
    ("shadow", &kivact_style::color::SHADOW_VAR),
    ("selection", &kivact_style::color::SELECTION_VAR),
    ("hover", &kivact_style::color::HOVER_VAR),
    ("pressed", &kivact_style::color::PRESSED_VAR),
    ("raised", &kivact_style::color::RAISED_VAR),
];

/// The length theme variables (px) a token name stands for.
const LENGTH_TOKENS: [(&str, &ThemeVar<f64>); 14] = [
    (
        "disabled_opacity",
        &kivact_style::control::DISABLED_OPACITY_VAR,
    ),
    ("control_height", &kivact_style::control::HEIGHT_VAR),
    ("overlay_margin", &kivact_style::control::OVERLAY_MARGIN_VAR),
    ("space.xs", &kivact_style::space::XS_VAR),
    ("space.sm", &kivact_style::space::SM_VAR),
    ("space.md", &kivact_style::space::MD_VAR),
    ("space.lg", &kivact_style::space::LG_VAR),
    ("space.xl", &kivact_style::space::XL_VAR),
    ("radius.sm", &kivact_style::radius::SM_VAR),
    ("radius.md", &kivact_style::radius::MD_VAR),
    ("radius.lg", &kivact_style::radius::LG_VAR),
    ("text.size", &kivact_style::font::SIZE_VAR),
    ("text.size_small", &kivact_style::font::SIZE_SMALL_VAR),
    ("text.size_large", &kivact_style::font::SIZE_LARGE_VAR),
];

/// The weight theme variables a token name stands for.
const WEIGHT_TOKENS: [(&str, &ThemeVar<u16>); 2] = [
    ("text.weight", &kivact_style::font::WEIGHT_VAR),
    ("text.weight_strong", &kivact_style::font::WEIGHT_STRONG_VAR),
];

fn strip_colors(name: &str) -> &str {
    name.strip_prefix("colors.").unwrap_or(name)
}

fn color_var(name: &str) -> Option<&'static ThemeVar<Color>> {
    let name = strip_colors(name);
    COLOR_TOKENS
        .iter()
        .find(|(token, _)| *token == name)
        .map(|(_, var)| *var)
}

fn length_var(name: &str) -> Option<&'static ThemeVar<f64>> {
    LENGTH_TOKENS
        .iter()
        .find(|(token, _)| *token == name)
        .map(|(_, var)| *var)
}

fn weight_var(name: &str) -> Option<&'static ThemeVar<u16>> {
    WEIGHT_TOKENS
        .iter()
        .find(|(token, _)| *token == name)
        .map(|(_, var)| *var)
}

/// The theme tokens of `theme`, with their values.
pub(crate) fn token_list(theme: &Theme) -> Vec<p::ThemeToken> {
    let mut tokens = Vec::new();
    for (name, var) in COLOR_TOKENS {
        tokens.push(p::ThemeToken {
            name: format!("colors.{name}"),
            kind: p::TokenKind::Color,
            value: hex(theme.get(var).components(theme)),
        });
    }
    for (name, value) in [
        ("space.xs", theme.get(&kivact_style::space::XS_VAR)),
        ("space.sm", theme.get(&kivact_style::space::SM_VAR)),
        ("space.md", theme.get(&kivact_style::space::MD_VAR)),
        ("space.lg", theme.get(&kivact_style::space::LG_VAR)),
        ("space.xl", theme.get(&kivact_style::space::XL_VAR)),
        ("radius.sm", theme.get(&kivact_style::radius::SM_VAR)),
        ("radius.md", theme.get(&kivact_style::radius::MD_VAR)),
        ("radius.lg", theme.get(&kivact_style::radius::LG_VAR)),
        ("text.size", theme.get(&kivact_style::font::SIZE_VAR)),
        (
            "text.size_small",
            theme.get(&kivact_style::font::SIZE_SMALL_VAR),
        ),
        (
            "text.size_large",
            theme.get(&kivact_style::font::SIZE_LARGE_VAR),
        ),
        (
            "text.weight",
            f64::from(theme.get(&kivact_style::font::WEIGHT_VAR)),
        ),
        (
            "text.weight_strong",
            f64::from(theme.get(&kivact_style::font::WEIGHT_STRONG_VAR)),
        ),
        ("focus_ring", theme.focus_ring()),
        ("focus_ring_offset", theme.focus_ring_offset()),
        (
            "disabled_opacity",
            theme.get(&kivact_style::control::DISABLED_OPACITY_VAR),
        ),
        (
            "control_height",
            theme.get(&kivact_style::control::HEIGHT_VAR),
        ),
        (
            "overlay_margin",
            theme.get(&kivact_style::control::OVERLAY_MARGIN_VAR),
        ),
    ] {
        tokens.push(p::ThemeToken {
            name: name.to_owned(),
            kind: p::TokenKind::Length,
            value: format!("{value}"),
        });
    }
    tokens.push(p::ThemeToken {
        name: "text.line_height".to_owned(),
        kind: p::TokenKind::Text,
        value: match theme.get(&kivact_style::font::LINE_HEIGHT_VAR) {
            LineHeight::Normal => "normal".to_owned(),
            LineHeight::Scale(value) => format!("{value}"),
            LineHeight::Px(value) => format!("{value}px"),
        },
    });
    tokens
}

fn hex([r, g, b, a]: [f64; 4]) -> String {
    let channel = |value: f64| (value.clamp(0.0, 1.0) * 255.0).round() as u8;
    if a < 1.0 {
        format!(
            "#{:02x}{:02x}{:02x}{:02x}",
            channel(r),
            channel(g),
            channel(b),
            channel(a)
        )
    } else {
        format!("#{:02x}{:02x}{:02x}", channel(r), channel(g), channel(b))
    }
}

/// The DevTools pseudo-locale transformation; `{name}` placeholders
/// keep their positions.
pub(crate) fn pseudo_locale(mode: p::PseudoLocale, text: &str) -> String {
    let accented = match mode {
        p::PseudoLocale::Off => return text.to_owned(),
        p::PseudoLocale::Rtl => return format!("\u{202e}{text}\u{202c}"),
        p::PseudoLocale::Accented | p::PseudoLocale::Long => accented(text),
    };
    let mut out = format!("［{accented}");
    if matches!(mode, p::PseudoLocale::Long) {
        let padding = (text.chars().count() as f64 * 0.4).ceil() as usize;
        out.push_str(&"~".repeat(padding));
    }
    out.push('］');
    out
}

fn accented(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut in_name = false;
    for c in text.chars() {
        match c {
            '{' => {
                in_name = true;
                out.push(c);
            }
            '}' => {
                in_name = false;
                out.push(c);
            }
            _ if in_name => out.push(c),
            _ => out.push(accent_char(c)),
        }
    }
    out
}

fn accent_char(c: char) -> char {
    match c {
        'a' => 'á',
        'e' => 'é',
        'i' => 'í',
        'o' => 'ó',
        'u' => 'ú',
        'c' => 'ç',
        'n' => 'ñ',
        'y' => 'ý',
        'A' => 'Á',
        'E' => 'É',
        'I' => 'Í',
        'O' => 'Ó',
        'U' => 'Ú',
        'C' => 'Ç',
        'N' => 'Ñ',
        'Y' => 'Ý',
        other => other,
    }
}
