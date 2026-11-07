//! Role: Parsing CSS color text (`Color::css`): names, hex and the functions
//! `rgb()`, `hsl()`, `hwb()`, `lab()`, `lch()`, `oklab()`, `oklch()` and
//! `color()`.

use super::Color;
use kivact_types::ColorSpace;

/// The CSS named colors (CSS Color 4 §6.1), as `0xRRGGBB`.
const NAMED: &[(&str, u32)] = &[
    ("aliceblue", 0xf0f8ff),
    ("antiquewhite", 0xfaebd7),
    ("aqua", 0x00ffff),
    ("aquamarine", 0x7fffd4),
    ("azure", 0xf0ffff),
    ("beige", 0xf5f5dc),
    ("bisque", 0xffe4c4),
    ("black", 0x000000),
    ("blanchedalmond", 0xffebcd),
    ("blue", 0x0000ff),
    ("blueviolet", 0x8a2be2),
    ("brown", 0xa52a2a),
    ("burlywood", 0xdeb887),
    ("cadetblue", 0x5f9ea0),
    ("chartreuse", 0x7fff00),
    ("chocolate", 0xd2691e),
    ("coral", 0xff7f50),
    ("cornflowerblue", 0x6495ed),
    ("cornsilk", 0xfff8dc),
    ("crimson", 0xdc143c),
    ("cyan", 0x00ffff),
    ("darkblue", 0x00008b),
    ("darkcyan", 0x008b8b),
    ("darkgoldenrod", 0xb8860b),
    ("darkgray", 0xa9a9a9),
    ("darkgreen", 0x006400),
    ("darkgrey", 0xa9a9a9),
    ("darkkhaki", 0xbdb76b),
    ("darkmagenta", 0x8b008b),
    ("darkolivegreen", 0x556b2f),
    ("darkorange", 0xff8c00),
    ("darkorchid", 0x9932cc),
    ("darkred", 0x8b0000),
    ("darksalmon", 0xe9967a),
    ("darkseagreen", 0x8fbc8f),
    ("darkslateblue", 0x483d8b),
    ("darkslategray", 0x2f4f4f),
    ("darkslategrey", 0x2f4f4f),
    ("darkturquoise", 0x00ced1),
    ("darkviolet", 0x9400d3),
    ("deeppink", 0xff1493),
    ("deepskyblue", 0x00bfff),
    ("dimgray", 0x696969),
    ("dimgrey", 0x696969),
    ("dodgerblue", 0x1e90ff),
    ("firebrick", 0xb22222),
    ("floralwhite", 0xfffaf0),
    ("forestgreen", 0x228b22),
    ("fuchsia", 0xff00ff),
    ("gainsboro", 0xdcdcdc),
    ("ghostwhite", 0xf8f8ff),
    ("gold", 0xffd700),
    ("goldenrod", 0xdaa520),
    ("gray", 0x808080),
    ("green", 0x008000),
    ("greenyellow", 0xadff2f),
    ("grey", 0x808080),
    ("honeydew", 0xf0fff0),
    ("hotpink", 0xff69b4),
    ("indianred", 0xcd5c5c),
    ("indigo", 0x4b0082),
    ("ivory", 0xfffff0),
    ("khaki", 0xf0e68c),
    ("lavender", 0xe6e6fa),
    ("lavenderblush", 0xfff0f5),
    ("lawngreen", 0x7cfc00),
    ("lemonchiffon", 0xfffacd),
    ("lightblue", 0xadd8e6),
    ("lightcoral", 0xf08080),
    ("lightcyan", 0xe0ffff),
    ("lightgoldenrodyellow", 0xfafad2),
    ("lightgray", 0xd3d3d3),
    ("lightgreen", 0x90ee90),
    ("lightgrey", 0xd3d3d3),
    ("lightpink", 0xffb6c1),
    ("lightsalmon", 0xffa07a),
    ("lightseagreen", 0x20b2aa),
    ("lightskyblue", 0x87cefa),
    ("lightslategray", 0x778899),
    ("lightslategrey", 0x778899),
    ("lightsteelblue", 0xb0c4de),
    ("lightyellow", 0xffffe0),
    ("lime", 0x00ff00),
    ("limegreen", 0x32cd32),
    ("linen", 0xfaf0e6),
    ("magenta", 0xff00ff),
    ("maroon", 0x800000),
    ("mediumaquamarine", 0x66cdaa),
    ("mediumblue", 0x0000cd),
    ("mediumorchid", 0xba55d3),
    ("mediumpurple", 0x9370db),
    ("mediumseagreen", 0x3cb371),
    ("mediumslateblue", 0x7b68ee),
    ("mediumspringgreen", 0x00fa9a),
    ("mediumturquoise", 0x48d1cc),
    ("mediumvioletred", 0xc71585),
    ("midnightblue", 0x191970),
    ("mintcream", 0xf5fffa),
    ("mistyrose", 0xffe4e1),
    ("moccasin", 0xffe4b5),
    ("navajowhite", 0xffdead),
    ("navy", 0x000080),
    ("oldlace", 0xfdf5e6),
    ("olive", 0x808000),
    ("olivedrab", 0x6b8e23),
    ("orange", 0xffa500),
    ("orangered", 0xff4500),
    ("orchid", 0xda70d6),
    ("palegoldenrod", 0xeee8aa),
    ("palegreen", 0x98fb98),
    ("paleturquoise", 0xafeeee),
    ("palevioletred", 0xdb7093),
    ("papayawhip", 0xffefd5),
    ("peachpuff", 0xffdab9),
    ("peru", 0xcd853f),
    ("pink", 0xffc0cb),
    ("plum", 0xdda0dd),
    ("powderblue", 0xb0e0e6),
    ("purple", 0x800080),
    ("rebeccapurple", 0x663399),
    ("red", 0xff0000),
    ("rosybrown", 0xbc8f8f),
    ("royalblue", 0x4169e1),
    ("saddlebrown", 0x8b4513),
    ("salmon", 0xfa8072),
    ("sandybrown", 0xf4a460),
    ("seagreen", 0x2e8b57),
    ("seashell", 0xfff5ee),
    ("sienna", 0xa0522d),
    ("silver", 0xc0c0c0),
    ("skyblue", 0x87ceeb),
    ("slateblue", 0x6a5acd),
    ("slategray", 0x708090),
    ("slategrey", 0x708090),
    ("snow", 0xfffafa),
    ("springgreen", 0x00ff7f),
    ("steelblue", 0x4682b4),
    ("tan", 0xd2b48c),
    ("teal", 0x008080),
    ("thistle", 0xd8bfd8),
    ("tomato", 0xff6347),
    ("turquoise", 0x40e0d0),
    ("violet", 0xee82ee),
    ("wheat", 0xf5deb3),
    ("white", 0xffffff),
    ("whitesmoke", 0xf5f5f5),
    ("yellow", 0xffff00),
    ("yellowgreen", 0x9acd32),
];

/// Parse a CSS color, or say what is wrong with it.
pub(super) fn parse(text: &str) -> Result<Color, String> {
    let t = text.trim().to_ascii_lowercase();
    if t.is_empty() {
        return Err("empty color".to_string());
    }
    if let Some(hex) = t.strip_prefix('#') {
        return hex_color(hex, &t);
    }
    if t == "transparent" {
        return Ok(Color::TRANSPARENT);
    }
    if let Some((_, v)) = NAMED.iter().find(|(n, _)| *n == t) {
        return Ok(rgb_hex(*v, 1.0));
    }
    let open = t
        .find('(')
        .ok_or_else(|| format!("unknown color `{text}`"))?;
    if !t.ends_with(')') {
        return Err(format!("color `{text}` is missing `)`"));
    }
    let name = t[..open].trim();
    let inner = &t[open + 1..t.len() - 1];
    if inner.contains("calc(") || inner.contains("none") || name.contains("current") {
        return Err(format!("color `{text}` uses an unsupported form"));
    }
    let (values, alpha) = split_alpha(inner, text)?;
    if name != "color" && values.len() != 3 {
        return Err(format!("color `{text}` needs three components"));
    }
    let color = color_function(name, &values, text)?;
    Ok(match alpha {
        Some(a) => color.with_alpha(a),
        None => color,
    })
}

/// The component `i` of a color function, or an error naming `text`.
fn component(values: &[String], i: usize, text: &str) -> Result<Value, String> {
    values
        .get(i)
        .ok_or_else(|| format!("color `{text}` needs more components"))
        .and_then(|s| Value::parse(s, text))
}

/// The color of the color function `name` (without alpha) over `values`.
fn color_function(name: &str, values: &[String], text: &str) -> Result<Color, String> {
    let v = |i: usize| component(values, i, text);
    match name {
        "rgb" | "rgba" => rgb_color(values, text),
        "hsl" | "hsla" => Ok(Color::from_space(
            ColorSpace::Hsl,
            v(0)?.angle_deg()?,
            v(1)?.fraction(100.0)?,
            v(2)?.fraction(100.0)?,
        )),
        "hwb" => Ok(Color::from_space(
            ColorSpace::Hwb,
            v(0)?.angle_deg()?,
            v(1)?.fraction(100.0)?,
            v(2)?.fraction(100.0)?,
        )),
        "lab" => Ok(Color::from_space(
            ColorSpace::Lab,
            v(0)?.scaled_pct(100.0)?,
            v(1)?.scaled_pct(125.0)?,
            v(2)?.scaled_pct(125.0)?,
        )),
        "lch" => Ok(Color::from_space(
            ColorSpace::Lch,
            v(0)?.scaled_pct(100.0)?,
            v(1)?.scaled_pct(150.0)?,
            v(2)?.angle_deg()?,
        )),
        "oklab" => Ok(Color::from_space(
            ColorSpace::Oklab,
            v(0)?.scaled_pct(1.0)?,
            v(1)?.scaled_pct(0.4)?,
            v(2)?.scaled_pct(0.4)?,
        )),
        "oklch" => Ok(Color::from_space(
            ColorSpace::Oklch,
            v(0)?.scaled_pct(1.0)?,
            v(1)?.scaled_pct(0.4)?,
            v(2)?.angle_deg()?,
        )),
        "color" => color_space_color(values, text),
        other => Err(format!("unknown color function `{other}()`")),
    }
}

/// The `rgb()` and `rgba()` color: percentages are of full scale, numbers
/// are 0..=255.
fn rgb_color(values: &[String], text: &str) -> Result<Color, String> {
    let ch = |i: usize| -> Result<f64, String> {
        let x = component(values, i, text)?;
        Ok(match x.unit {
            Unit::Percent => x.value / 100.0,
            _ => x.value / 255.0,
        })
    };
    Ok(Color::from_rgba_f64(ch(0)?, ch(1)?, ch(2)?, 1.0))
}

/// The `color(space c1 c2 c3)` color.
fn color_space_color(values: &[String], text: &str) -> Result<Color, String> {
    let space = values
        .first()
        .ok_or_else(|| format!("color `{text}` needs a color space"))?;
    let space = match space.as_str() {
        "srgb" => ColorSpace::Srgb,
        "srgb-linear" => ColorSpace::SrgbLinear,
        "display-p3" => ColorSpace::DisplayP3,
        "a98-rgb" => ColorSpace::A98Rgb,
        "prophoto-rgb" => ColorSpace::ProphotoRgb,
        "rec2020" => ColorSpace::Rec2020,
        "xyz" | "xyz-d65" => ColorSpace::XyzD65,
        "xyz-d50" => ColorSpace::XyzD50,
        other => return Err(format!("unknown color space `{other}`")),
    };
    if values.len() != 4 {
        return Err(format!("color `{text}` needs three components"));
    }
    let c = |s: &str| Value::parse(s, text).and_then(|x| x.fraction(1.0));
    Ok(Color::from_space(
        space,
        c(&values[1])?,
        c(&values[2])?,
        c(&values[3])?,
    ))
}

/// Split `inner` into its three components and an optional alpha. The
/// legacy comma form and the modern space form with `/ alpha` both parse.
fn split_alpha(inner: &str, text: &str) -> Result<(Vec<String>, Option<f64>), String> {
    if inner.contains(',') {
        let parts: Vec<&str> = inner.split(',').map(str::trim).collect();
        return match parts.len() {
            3 => Ok((parts.iter().map(|s| s.to_string()).collect(), None)),
            4 => {
                let a = Value::parse(parts[3], text)?.fraction(1.0)?;
                Ok((parts[..3].iter().map(|s| s.to_string()).collect(), Some(a)))
            }
            _ => Err(format!("color `{text}` has the wrong number of components")),
        };
    }
    let (main, alpha) = match inner.split_once('/') {
        Some((m, a)) => (m, Some(Value::parse(a.trim(), text)?.fraction(1.0)?)),
        None => (inner, None),
    };
    let comps: Vec<String> = main.split_whitespace().map(str::to_string).collect();
    Ok((comps, alpha))
}

/// A `#rgb`, `#rgba`, `#rrggbb` or `#rrggbbaa` color.
fn hex_color(hex: &str, text: &str) -> Result<Color, String> {
    let digit = |c: char| {
        c.to_digit(16)
            .ok_or_else(|| format!("bad hex color `{text}`"))
    };
    let digits: Vec<u32> = hex.chars().map(digit).collect::<Result<_, _>>()?;
    let expand = |d: &[u32]| -> Vec<u32> { d.iter().flat_map(|v| [v * 17]).collect() };
    let (rgba_digits, byte): (Vec<u32>, bool) = match digits.len() {
        3 | 4 => (expand(&digits), false),
        6 | 8 => (digits.clone(), true),
        _ => return Err(format!("bad hex color `{text}`")),
    };
    let pair = |i: usize| -> u32 {
        if byte {
            rgba_digits[2 * i] * 16 + rgba_digits[2 * i + 1]
        } else {
            rgba_digits[i]
        }
    };
    let (r, g, b) = (pair(0), pair(1), pair(2));
    let alpha = if rgba_digits.len() == 4 || rgba_digits.len() == 8 {
        pair(3) as f64 / 255.0
    } else {
        1.0
    };
    let hex = (r << 16) | (g << 8) | b;
    Ok(rgb_hex(hex, alpha))
}

/// A color of `0xRRGGBB` and alpha.
fn rgb_hex(hex: u32, alpha: f64) -> Color {
    super::rgba(hex, alpha)
}

/// The unit of a component.
#[derive(Clone, Copy, PartialEq)]
enum Unit {
    Number,
    Percent,
    Deg,
    Rad,
    Grad,
    Turn,
}

/// One parsed component: a number with its unit.
struct Value {
    value: f64,
    unit: Unit,
}

impl Value {
    fn parse(s: &str, text: &str) -> Result<Value, String> {
        let bad = || format!("bad component `{s}` in color `{text}`");
        let (num, unit) = if let Some(n) = s.strip_suffix('%') {
            (n, Unit::Percent)
        } else if let Some(n) = s.strip_suffix("deg") {
            (n, Unit::Deg)
        } else if let Some(n) = s.strip_suffix("grad") {
            (n, Unit::Grad)
        } else if let Some(n) = s.strip_suffix("rad") {
            (n, Unit::Rad)
        } else if let Some(n) = s.strip_suffix("turn") {
            (n, Unit::Turn)
        } else {
            (s, Unit::Number)
        };
        let value: f64 = num.parse().map_err(|_| bad())?;
        if !value.is_finite() {
            return Err(bad());
        }
        Ok(Value { value, unit })
    }

    /// A hue in degrees (a number is degrees).
    fn angle_deg(&self) -> Result<f64, String> {
        Ok(match self.unit {
            Unit::Number | Unit::Deg => self.value,
            Unit::Rad => self.value.to_degrees(),
            Unit::Grad => self.value * 0.9,
            Unit::Turn => self.value * 360.0,
            Unit::Percent => return Err("a hue cannot be a percentage".to_string()),
        })
    }

    /// A fraction of `full` for a percentage, and the number itself as a
    /// fraction of `full`: a percentage of 100 is 1, a number is its value
    /// over `full` as well.
    fn fraction(&self, full: f64) -> Result<f64, String> {
        match self.unit {
            Unit::Percent => Ok(self.value / 100.0),
            Unit::Number => Ok(self.value / full),
            _ => Err("this component takes a number or a percentage".to_string()),
        }
    }

    /// A number as it is, a percentage of `full`.
    fn scaled_pct(&self, full: f64) -> Result<f64, String> {
        match self.unit {
            Unit::Percent => Ok(self.value * full / 100.0),
            Unit::Number => Ok(self.value),
            _ => Err("this component takes a number or a percentage".to_string()),
        }
    }
}
