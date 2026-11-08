//! Role: Apply DevTools overrides and emulation to styles, themes and texts.

use std::panic::Location;

use kivact_devtools_protocol as p;
use kivact_runtime::{Owner, Runtime};
use kivact_style::theme::{Palette, Radii, Spacing};
use kivact_style::{Color, ColorToken, Length, LineHeight, Style, TextAlign, Theme};

use kivact_types::{Display, NodeId};

/// The property table of design 4.4.5: one layout property.
fn override_layout(property: &str, value: &str) -> Option<Result<Style, String>> {
    let result = match property {
        "width" => length(value).map(|v| Style::default().width(v)),
        "height" => length(value).map(|v| Style::default().height(v)),
        "min_width" => length(value).map(|v| Style::default().min_width(v)),
        "min_height" => length(value).map(|v| Style::default().min_height(v)),
        "max_width" => length(value).map(|v| Style::default().max_width(v)),
        "max_height" => length(value).map(|v| Style::default().max_height(v)),
        "padding" => length(value).map(|v| Style::default().padding(v)),
        "padding_top" => length(value).map(|v| Style::default().padding_top(v)),
        "padding_right" => length(value).map(|v| Style::default().padding_right(v)),
        "padding_bottom" => length(value).map(|v| Style::default().padding_bottom(v)),
        "padding_left" => length(value).map(|v| Style::default().padding_left(v)),
        "margin" => length(value).map(|v| Style::default().margin(v)),
        "margin_top" => length(value).map(|v| Style::default().margin_top(v)),
        "margin_right" => length(value).map(|v| Style::default().margin_right(v)),
        "margin_bottom" => length(value).map(|v| Style::default().margin_bottom(v)),
        "margin_left" => length(value).map(|v| Style::default().margin_left(v)),
        "gap" => length(value).map(|v| Style::default().gap(v)),
        "row_gap" => length(value).map(|v| Style::default().row_gap(v)),
        "column_gap" => length(value).map(|v| Style::default().column_gap(v)),
        "grow" => number(value).map(|v| Style::default().grow(v)),
        "shrink" => number(value).map(|v| Style::default().shrink(v)),
        "display" => display(value).map(|v| Style::default().display(v)),
        _ => return None,
    };
    Some(with_property(property, result))
}

/// The property table of design 4.4.5: one paint property.
fn override_paint(property: &str, value: &str) -> Option<Result<Style, String>> {
    let result = match property {
        "background" => color(value).map(|v| Style::default().background(v)),
        "color" => color(value).map(|v| Style::default().color(v)),
        "border_color" => color(value).map(|v| Style::default().border_color(v)),
        "border_width" => number(value).map(|v| Style::default().border_width(v)),
        "radius" => number(value).map(|v| Style::default().radius(v)),
        "opacity" => number(value).map(|v| Style::default().opacity(v)),
        _ => return None,
    };
    Some(with_property(property, result))
}

/// The property table of design 4.4.5: one text property.
fn override_text(property: &str, value: &str) -> Option<Result<Style, String>> {
    let result = match property {
        "font_size" => number(value).map(|v| Style::default().font_size(v)),
        "line_height" => number(value).map(|v| Style::default().line_height(v)),
        "weight" => number(value).map(|v| Style::default().weight(v as u16)),
        "text_align" => text_align(value).map(|v| Style::default().text_align(v)),
        _ => return None,
    };
    Some(with_property(property, result))
}

/// Read one override into a style, or name the property it does not know.
pub(crate) fn override_style(property: &str, value: &str) -> Result<Style, String> {
    if let Some(result) = override_layout(property, value) {
        return result;
    }
    if let Some(result) = override_paint(property, value) {
        return result;
    }
    if let Some(result) = override_text(property, value) {
        return result;
    }
    Err(format!("unknown property: {property}"))
}

fn with_property<T>(property: &str, result: Result<T, String>) -> Result<T, String> {
    result.map_err(|message| format!("{property}: {message}"))
}

fn length(value: &str) -> Result<Length, String> {
    let value = value.trim();
    if value == "auto" {
        return Ok(Length::Auto);
    }
    if let Some(percent) = value.strip_suffix('%') {
        return percent
            .trim()
            .parse::<f64>()
            .map(Length::Percent)
            .map_err(|_| {
                format!("expected a length (a number, a percentage, or auto), got {value}")
            });
    }
    value
        .parse::<f64>()
        .map(Length::Px)
        .map_err(|_| format!("expected a length (a number, a percentage, or auto), got {value}"))
}

fn number(value: &str) -> Result<f64, String> {
    value
        .trim()
        .parse::<f64>()
        .map_err(|_| format!("expected a number, got {value}"))
}

fn color(value: &str) -> Result<Color, String> {
    if let Some(token) = color_token(value.trim()) {
        return Ok(Color::token(token));
    }
    Color::css(value).map_err(|_| format!("expected a color, got {value}"))
}

fn display(value: &str) -> Result<Display, String> {
    match value.trim() {
        "flex" => Ok(Display::Flex),
        "grid" => Ok(Display::Grid),
        "contents" => Ok(Display::Contents),
        _ => Err(format!("expected flex, grid or contents, got {value}")),
    }
}

fn text_align(value: &str) -> Result<TextAlign, String> {
    match value.trim() {
        "start" => Ok(TextAlign::Start),
        "center" => Ok(TextAlign::Center),
        "end" => Ok(TextAlign::End),
        _ => Err(format!("expected start, center or end, got {value}")),
    }
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
    if let Some(token) = color_token(token) {
        let Ok(color) = color(value) else {
            return theme;
        };
        let mut palette: Palette = theme.colors().clone();
        set_palette(&mut palette, token, color);
        return theme.with_colors(palette);
    }
    if let Some(theme) = apply_space(&theme, token, value) {
        return theme;
    }
    if let Some(theme) = apply_radius(&theme, token, value) {
        return theme;
    }
    if let Some(theme) = apply_text(&theme, token, value) {
        return theme;
    }
    apply_scalar(&theme, token, value)
}

fn apply_space(theme: &Theme, token: &str, value: &str) -> Option<Theme> {
    if !matches!(
        token,
        "space.xs" | "space.sm" | "space.md" | "space.lg" | "space.xl"
    ) {
        return None;
    }
    let Ok(value) = number(value) else {
        return Some(theme.clone());
    };
    let mut space: Spacing = *theme.space();
    match token {
        "space.xs" => space.xs = value,
        "space.sm" => space.sm = value,
        "space.md" => space.md = value,
        "space.lg" => space.lg = value,
        "space.xl" => space.xl = value,
        _ => return None,
    }
    Some(theme.clone().with_space(space))
}

fn apply_radius(theme: &Theme, token: &str, value: &str) -> Option<Theme> {
    if !matches!(token, "radius.sm" | "radius.md" | "radius.lg") {
        return None;
    }
    let Ok(value) = number(value) else {
        return Some(theme.clone());
    };
    let mut radius: Radii = *theme.radius();
    match token {
        "radius.sm" => radius.sm = value,
        "radius.md" => radius.md = value,
        "radius.lg" => radius.lg = value,
        _ => return None,
    }
    Some(theme.clone().with_radius(radius))
}

fn apply_text(theme: &Theme, token: &str, value: &str) -> Option<Theme> {
    let line_height = if token == "text.line_height" {
        if let Some(px) = value.trim().strip_suffix("px") {
            px.trim().parse::<f64>().ok().map(LineHeight::Px)
        } else {
            number(value).ok().map(LineHeight::Scale)
        }
    } else {
        None
    };
    if token == "text.line_height" {
        return Some(match line_height {
            Some(value) => {
                let mut text = theme.text().clone();
                text.line_height = value;
                theme.clone().with_text(text)
            }
            None => theme.clone(),
        });
    }
    if !matches!(
        token,
        "text.size" | "text.size_small" | "text.size_large" | "text.weight" | "text.weight_strong"
    ) {
        return None;
    }
    let Ok(value) = number(value) else {
        return Some(theme.clone());
    };
    let mut text = theme.text().clone();
    match token {
        "text.size" => text.size = value,
        "text.size_small" => text.size_small = value,
        "text.size_large" => text.size_large = value,
        "text.weight" => text.weight = value as u16,
        "text.weight_strong" => text.weight_strong = value as u16,
        _ => return None,
    }
    Some(theme.clone().with_text(text))
}

fn apply_scalar(theme: &Theme, token: &str, value: &str) -> Theme {
    let Ok(value) = number(value) else {
        return theme.clone();
    };
    match token {
        "focus_ring" => theme.clone().with_focus_ring(value),
        "focus_ring_offset" => theme.clone().with_focus_ring_offset(value),
        "disabled_opacity" => theme.clone().with_disabled_opacity(value),
        "control_height" => theme.clone().with_control_height(value),
        "overlay_margin" => theme.clone().with_overlay_margin(value),
        _ => theme.clone(),
    }
}

fn set_palette(palette: &mut Palette, token: ColorToken, color: Color) {
    match token {
        ColorToken::Surface => palette.surface = color,
        ColorToken::SurfaceAlt => palette.surface_alt = color,
        ColorToken::Text => palette.text = color,
        ColorToken::TextMuted => palette.text_muted = color,
        ColorToken::Accent => palette.accent = color,
        ColorToken::AccentText => palette.accent_text = color,
        ColorToken::Border => palette.border = color,
        ColorToken::Focus => palette.focus = color,
        ColorToken::Danger => palette.danger = color,
        ColorToken::Success => palette.success = color,
        ColorToken::Warning => palette.warning = color,
        ColorToken::Overlay => palette.overlay = color,
        ColorToken::Shadow => palette.shadow = color,
        ColorToken::Selection => palette.selection = color,
        ColorToken::Hover => palette.hover = color,
        ColorToken::Pressed => palette.pressed = color,
        ColorToken::Raised => palette.raised = color,
    }
}

const COLOR_TOKENS: [(&str, ColorToken); 17] = [
    ("surface", ColorToken::Surface),
    ("surface_alt", ColorToken::SurfaceAlt),
    ("text", ColorToken::Text),
    ("text_muted", ColorToken::TextMuted),
    ("accent", ColorToken::Accent),
    ("accent_text", ColorToken::AccentText),
    ("border", ColorToken::Border),
    ("focus", ColorToken::Focus),
    ("danger", ColorToken::Danger),
    ("success", ColorToken::Success),
    ("warning", ColorToken::Warning),
    ("overlay", ColorToken::Overlay),
    ("shadow", ColorToken::Shadow),
    ("selection", ColorToken::Selection),
    ("hover", ColorToken::Hover),
    ("pressed", ColorToken::Pressed),
    ("raised", ColorToken::Raised),
];

fn color_token(name: &str) -> Option<ColorToken> {
    let name = name.strip_prefix("colors.").unwrap_or(name);
    COLOR_TOKENS
        .iter()
        .find(|(token, _)| *token == name)
        .map(|(_, token)| *token)
}

/// The theme tokens of `theme`, with their values.
pub(crate) fn token_list(theme: &Theme) -> Vec<p::ThemeToken> {
    let mut tokens = Vec::new();
    for (name, token) in COLOR_TOKENS {
        tokens.push(p::ThemeToken {
            name: format!("colors.{name}"),
            kind: p::TokenKind::Color,
            value: hex(theme.colors().get(token).components_in(theme)),
        });
    }
    for (name, value) in [
        ("space.xs", theme.space().xs),
        ("space.sm", theme.space().sm),
        ("space.md", theme.space().md),
        ("space.lg", theme.space().lg),
        ("space.xl", theme.space().xl),
        ("radius.sm", theme.radius().sm),
        ("radius.md", theme.radius().md),
        ("radius.lg", theme.radius().lg),
        ("text.size", theme.text().size),
        ("text.size_small", theme.text().size_small),
        ("text.size_large", theme.text().size_large),
        ("text.weight", f64::from(theme.text().weight)),
        ("text.weight_strong", f64::from(theme.text().weight_strong)),
        ("focus_ring", theme.focus_ring()),
        ("focus_ring_offset", theme.focus_ring_offset()),
        ("disabled_opacity", theme.disabled_opacity()),
        ("control_height", theme.control_height()),
        ("overlay_margin", theme.overlay_margin()),
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
        value: match theme.text().line_height {
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

/// The pseudo-locale transformation of design 4.2.4; `{name}` placeholders
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
