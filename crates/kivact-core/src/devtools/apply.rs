//! Role: Apply DevTools overrides and emulation to styles, themes and texts.

use std::panic::Location;

use kivact_devtools_protocol as p;
use kivact_runtime::Owner;
use kivact_runtime::driver::Runtime;
use kivact_style::ThemeVar;
use kivact_style::{Color, Length, LineHeight, Style, TextAlign, Theme};

use kivact_types::{Display, NodeId};

/// The DevTools override properties: one layout property.
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

/// The DevTools override properties: one paint property.
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

/// The DevTools override properties: one text property.
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
    if let Some(var) = color_var(value.trim()) {
        return Ok(Color::var(var));
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

/// The override syntax (`12`, `50%`, `auto`, `#2563eb`, a token name, ...) of
/// one property value, given as
/// `describe_style` wrote it: the `Debug` text of the value, wrapped in
/// `Some(...)` for an optional field. `None` when `property` is not in that
/// table or the value has no such syntax. The edges and corners that
/// `describe_style` names one by one (`border_top_width`, `radius_top_left`, ...)
/// are not in the table, so they have no editable form.
pub(crate) fn editable_value(property: &str, described: &str) -> Option<String> {
    let text = described
        .strip_prefix("Some(")
        .and_then(|rest| rest.strip_suffix(')'))
        .unwrap_or(described);
    match property {
        "width" | "height" | "min_width" | "min_height" | "max_width" | "max_height"
        | "padding" | "padding_top" | "padding_right" | "padding_bottom" | "padding_left"
        | "margin" | "margin_top" | "margin_right" | "margin_bottom" | "margin_left" | "gap"
        | "row_gap" | "column_gap" => length_text(text),
        "grow" | "shrink" | "opacity" | "font_size" | "weight" => {
            text.parse::<f64>().ok().map(number_text)
        }
        "line_height" => one_value(text, "Scale").map(number_text),
        "background" => background_text(text),
        "color" => color_text(text),
        "display" => display_text(text),
        "text_align" => text_align_text(text),
        _ => None,
    }
}

fn number_text(value: f64) -> String {
    format!("{value}")
}

/// The text inside `name(...)` when `text` is exactly that, and the text
/// after the closing parenthesis.
fn split_group<'a>(text: &'a str, name: &str) -> Option<(&'a str, &'a str)> {
    let inner = text.strip_prefix(name)?.strip_prefix('(')?;
    let mut depth = 1_usize;
    for (index, c) in inner.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Some((&inner[..index], &inner[index + 1..]));
                }
            }
            _ => {}
        }
    }
    None
}

/// The number inside `name(...)`, when `text` is exactly that.
fn one_value(text: &str, name: &str) -> Option<f64> {
    let (inner, rest) = split_group(text, name)?;
    if !rest.is_empty() {
        return None;
    }
    inner.trim().parse::<f64>().ok()
}

/// `Auto`, `Px(n)` or `Percent(n)` as `auto`, `n` or `n%`.
fn length_text(text: &str) -> Option<String> {
    if text == "Auto" {
        return Some("auto".to_owned());
    }
    if let Some(px) = one_value(text, "Px") {
        return Some(number_text(px));
    }
    one_value(text, "Percent").map(|percent| format!("{}%", number_text(percent)))
}

fn display_text(text: &str) -> Option<String> {
    Some(
        match text {
            "Flex" => "flex",
            "Grid" => "grid",
            "Contents" => "contents",
            _ => return None,
        }
        .to_owned(),
    )
}

fn text_align_text(text: &str) -> Option<String> {
    Some(
        match text {
            "Start" => "start",
            "Center" => "center",
            "End" => "end",
            _ => return None,
        }
        .to_owned(),
    )
}

/// `Color(Rgba(r, g, b, a))` as `#rrggbb` or `#rrggbbaa`, and
/// `Color(Token(Name, 1.0))` as the token's name.
fn color_text(text: &str) -> Option<String> {
    let (repr, rest) = split_group(text, "Color")?;
    if !rest.is_empty() {
        return None;
    }
    if let Some((components, after)) = split_group(repr, "Rgba") {
        if !after.is_empty() {
            return None;
        }
        let values = components
            .split(", ")
            .map(|part| part.trim().parse::<f64>())
            .collect::<Result<Vec<f64>, _>>()
            .ok()?;
        let rgba: [f64; 4] = values.try_into().ok()?;
        return Some(hex(rgba));
    }
    let (token, after) = split_group(repr, "Token")?;
    if !after.is_empty() {
        return None;
    }
    let (name, alpha) = token.split_once(", ")?;
    if alpha.trim().parse::<f64>().ok()? != 1.0 {
        return None;
    }
    token_name(name)
}

/// The theme token name of a `Color` `Debug` name: `SurfaceAlt` is
/// `surface_alt`.
fn token_name(debug: &str) -> Option<String> {
    let mut name = String::with_capacity(debug.len() + 2);
    for (index, c) in debug.chars().enumerate() {
        if c.is_ascii_uppercase() {
            if index > 0 {
                name.push('_');
            }
            name.push(c.to_ascii_lowercase());
        } else {
            name.push(c);
        }
    }
    color_var(&name).map(|_| name)
}

/// The color of a background that is one solid color covering the box, from
/// the `Background` `Debug` text.
fn background_text(text: &str) -> Option<String> {
    let (layers, rest) = split_group(text, "Background")?;
    if !rest.is_empty() {
        return None;
    }
    let paint = layers.strip_prefix("[Paint { image: ")?;
    let (image, tail) = split_group(paint, "Color")?;
    let solid = tail.starts_with(", size: Auto, ")
        && tail
            .ends_with(", repeat: (true, true), clip: BorderBox, origin: None, blend: Normal }]")
        && !tail.contains("Paint {");
    if !solid {
        return None;
    }
    color_text(image)
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
            value: hex(theme.get(var).components_in(theme)),
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

#[cfg(test)]
mod tests {
    //! Role: Tests for the override syntax and its round trip.

    use super::{editable_value, override_style};
    use kivact_style::internal::describe_style;

    /// One sample value for each property of the table that `describe_style`
    /// names as the property itself. The border and radius properties are
    /// left out: `describe_style` names their edges and corners one by one.
    const SAMPLES: [(&str, &str); 26] = [
        ("width", "120"),
        ("height", "50%"),
        ("min_width", "auto"),
        ("min_height", "20"),
        ("max_width", "300"),
        ("max_height", "80%"),
        ("padding", "12"),
        ("padding_top", "4"),
        ("margin", "8"),
        ("margin_left", "2"),
        ("gap", "8"),
        ("row_gap", "4"),
        ("column_gap", "6"),
        ("grow", "1"),
        ("shrink", "0.5"),
        ("display", "grid"),
        ("background", "#2563eb"),
        ("color", "accent"),
        ("opacity", "0.5"),
        ("font_size", "14"),
        ("line_height", "1.5"),
        ("weight", "700"),
        ("text_align", "center"),
        ("text_align", "end"),
        ("display", "contents"),
        ("background", "#2563eb80"),
    ];

    #[test]
    fn every_described_value_overrides_back_to_the_same_text() {
        for (property, sample) in SAMPLES {
            let style = override_style(property, sample)
                .unwrap_or_else(|error| panic!("{property} {sample}: {error}"));
            for (name, described) in describe_style(&style) {
                let value = editable_value(name, &described)
                    .unwrap_or_else(|| panic!("{name} has no editable form: {described}"));
                let again = override_style(name, &value)
                    .unwrap_or_else(|error| panic!("{name} {value}: {error}"));
                let text = describe_style(&again)
                    .into_iter()
                    .find(|(same, _)| *same == name)
                    .map(|(_, text)| text);
                assert_eq!(text.as_deref(), Some(described.as_str()), "{name} {value}");
            }
        }
    }

    #[test]
    fn the_editable_form_is_the_override_syntax() {
        assert_eq!(
            editable_value("width", "Some(Px(12.0))").as_deref(),
            Some("12")
        );
        assert_eq!(
            editable_value("height", "Some(Percent(50.0))").as_deref(),
            Some("50%")
        );
        assert_eq!(
            editable_value("min_width", "Some(Auto)").as_deref(),
            Some("auto")
        );
        assert_eq!(
            editable_value("padding_left", "Px(0.5)").as_deref(),
            Some("0.5")
        );
        assert_eq!(
            editable_value("weight", "Some(700)").as_deref(),
            Some("700")
        );
        assert_eq!(
            editable_value("color", "Some(Color(Token(Accent, 1.0)))").as_deref(),
            Some("accent")
        );
        assert_eq!(
            editable_value("text_align", "Some(Center)").as_deref(),
            Some("center")
        );
        assert_eq!(editable_value("width", "Some(MinContent)"), None);
        assert_eq!(editable_value("border_top_width", "Some(1.0)"), None);
    }
}
