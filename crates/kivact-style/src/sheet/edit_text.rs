//! Role: The DevTools text form of each value type: shown and parsed.

use kivact_types::{Align, Display, FontStyle, Justify, Length, TextAlign, ThemeVar};

use crate::color_value::Color;
use crate::geometry::CornerRadius;
use crate::paint::Background;
use crate::paint::Paint;
use crate::text::LineHeight;

/// A value DevTools writes and reads as text.
pub(crate) trait EditText: Sized {
    fn to_edit_text(&self) -> String;
    fn from_edit_text(text: &str) -> Result<Self, String>;
}

const COLOR_VARS: [&ThemeVar<Color>; 17] = [
    &crate::color::SURFACE_VAR,
    &crate::color::SURFACE_ALT_VAR,
    &crate::color::TEXT_VAR,
    &crate::color::TEXT_MUTED_VAR,
    &crate::color::ACCENT_VAR,
    &crate::color::ACCENT_TEXT_VAR,
    &crate::color::BORDER_VAR,
    &crate::color::FOCUS_VAR,
    &crate::color::DANGER_VAR,
    &crate::color::SUCCESS_VAR,
    &crate::color::WARNING_VAR,
    &crate::color::OVERLAY_VAR,
    &crate::color::SHADOW_VAR,
    &crate::color::SELECTION_VAR,
    &crate::color::HOVER_VAR,
    &crate::color::PRESSED_VAR,
    &crate::color::RAISED_VAR,
];

const LENGTH_VARS: [&ThemeVar<f64>; 11] = [
    &crate::space::XS_VAR,
    &crate::space::SM_VAR,
    &crate::space::MD_VAR,
    &crate::space::LG_VAR,
    &crate::space::XL_VAR,
    &crate::radius::SM_VAR,
    &crate::radius::MD_VAR,
    &crate::radius::LG_VAR,
    &crate::font::SIZE_VAR,
    &crate::font::SIZE_SMALL_VAR,
    &crate::font::SIZE_LARGE_VAR,
];

impl EditText for Length {
    fn to_edit_text(&self) -> String {
        match self {
            Length::Px(v) => format!("{v}"),
            Length::Percent(v) => format!("{v}%"),
            Length::Auto => "auto".to_owned(),
            Length::MinContent => "min-content".to_owned(),
            Length::MaxContent => "max-content".to_owned(),
            Length::FitContent => "fit-content".to_owned(),
            Length::Em(v) => format!("{v}em"),
            Length::Calc {
                percent,
                px,
                em,
                var,
            } => {
                let var = var.map_or(String::new(), |(v, k)| format!(" + {k} * {}", v.name()));
                format!("calc({percent}% + {px}px + {em}em{var})")
            }
            Length::Var(v) => v.name().to_owned(),
            Length::Invalid => "invalid".to_owned(),
        }
    }

    fn from_edit_text(text: &str) -> Result<Self, String> {
        let text = text.trim();
        match text {
            "auto" => return Ok(Length::Auto),
            "min-content" => return Ok(Length::MinContent),
            "max-content" => return Ok(Length::MaxContent),
            "fit-content" => return Ok(Length::FitContent),
            _ => {}
        }
        if let Some(var) = LENGTH_VARS.into_iter().find(|v| v.name() == text) {
            return Ok(Length::Var(var));
        }
        if let Some(em) = text.strip_suffix("em") {
            return em
                .trim()
                .parse::<f64>()
                .map(Length::Em)
                .map_err(|_| expected("a length", text));
        }
        if let Some(percent) = text.strip_suffix('%') {
            return percent
                .trim()
                .parse::<f64>()
                .map(Length::Percent)
                .map_err(|_| expected("a length", text));
        }
        text.trim_end_matches("px")
            .parse::<f64>()
            .map(Length::Px)
            .map_err(|_| expected("a length", text))
    }
}

impl EditText for Color {
    fn to_edit_text(&self) -> String {
        match self.var_target() {
            Some((var, 1.0)) => var.name().to_owned(),
            _ => {
                let [r, g, b, a] = self.components(&crate::Theme::light());
                let c = |v: f64| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
                if a < 1.0 {
                    format!("#{:02x}{:02x}{:02x}{:02x}", c(r), c(g), c(b), c(a))
                } else {
                    format!("#{:02x}{:02x}{:02x}", c(r), c(g), c(b))
                }
            }
        }
    }

    fn from_edit_text(text: &str) -> Result<Self, String> {
        let text = text.trim();
        if let Some(var) = COLOR_VARS.into_iter().find(|v| v.name() == text) {
            return Ok(Color::var(var));
        }
        Color::css(text).map_err(|_| expected("a color", text))
    }
}

impl EditText for Background {
    fn to_edit_text(&self) -> String {
        match self.solid() {
            Some(c) => c.to_edit_text(),
            None => format!("{self:?}"),
        }
    }

    fn from_edit_text(text: &str) -> Result<Self, String> {
        Color::from_edit_text(text).map(|c| Background::from(Paint::from(c)))
    }
}

impl EditText for LineHeight {
    fn to_edit_text(&self) -> String {
        match self {
            LineHeight::Normal => "normal".to_owned(),
            LineHeight::Scale(v) => format!("{v}"),
            LineHeight::Px(v) => format!("{v}px"),
        }
    }

    fn from_edit_text(text: &str) -> Result<Self, String> {
        let text = text.trim();
        if text == "normal" {
            return Ok(LineHeight::Normal);
        }
        if let Some(px) = text.strip_suffix("px") {
            return px
                .trim()
                .parse::<f64>()
                .map(LineHeight::Px)
                .map_err(|_| expected("a line height", text));
        }
        text.parse::<f64>()
            .map(LineHeight::Scale)
            .map_err(|_| expected("a line height", text))
    }
}

impl EditText for CornerRadius {
    fn to_edit_text(&self) -> String {
        if self.x == self.y {
            self.x.to_edit_text()
        } else {
            format!("{} {}", self.x.to_edit_text(), self.y.to_edit_text())
        }
    }

    fn from_edit_text(text: &str) -> Result<Self, String> {
        let parts: Vec<&str> = text.split_whitespace().collect();
        let (x, y) = match parts.as_slice() {
            [one] => (Length::from_edit_text(one)?, Length::from_edit_text(one)?),
            [x, y] => (Length::from_edit_text(x)?, Length::from_edit_text(y)?),
            _ => return Err(expected("one or two lengths", text)),
        };
        Ok(CornerRadius { x, y })
    }
}

impl EditText for Display {
    fn to_edit_text(&self) -> String {
        match self {
            Display::Flex => "flex",
            Display::Grid => "grid",
            Display::Contents => "contents",
        }
        .to_owned()
    }

    fn from_edit_text(text: &str) -> Result<Self, String> {
        match text.trim() {
            "flex" => Ok(Display::Flex),
            "grid" => Ok(Display::Grid),
            "contents" => Ok(Display::Contents),
            other => Err(expected("flex, grid or contents", other)),
        }
    }
}

impl EditText for TextAlign {
    fn to_edit_text(&self) -> String {
        match self {
            TextAlign::Start => "start",
            TextAlign::Center => "center",
            TextAlign::End => "end",
            TextAlign::Justify => "justify",
        }
        .to_owned()
    }

    fn from_edit_text(text: &str) -> Result<Self, String> {
        match text.trim() {
            "start" => Ok(TextAlign::Start),
            "center" => Ok(TextAlign::Center),
            "end" => Ok(TextAlign::End),
            "justify" => Ok(TextAlign::Justify),
            other => Err(expected("start, center, end or justify", other)),
        }
    }
}

impl EditText for FontStyle {
    fn to_edit_text(&self) -> String {
        match self {
            FontStyle::Normal => "normal",
            FontStyle::Italic => "italic",
        }
        .to_owned()
    }

    fn from_edit_text(text: &str) -> Result<Self, String> {
        match text.trim() {
            "normal" => Ok(FontStyle::Normal),
            "italic" => Ok(FontStyle::Italic),
            other => Err(expected("normal or italic", other)),
        }
    }
}

/// Numbers and flags: written as Rust writes them, parsed by `str::parse`.
macro_rules! number_text {
    ($($t:ty),*) => {$(
        impl EditText for $t {
            fn to_edit_text(&self) -> String { format!("{self}") }
            fn from_edit_text(text: &str) -> Result<Self, String> {
                text.trim().parse::<$t>().map_err(|_| expected(stringify!($t), text))
            }
        }
    )*};
}
number_text!(f64, u16, u32, i32, bool);

/// Values DevTools shows (their `Debug`) but does not parse.
macro_rules! shown_only {
    ($($t:ty),*) => {$(
        impl EditText for $t {
            fn to_edit_text(&self) -> String { format!("{self:?}") }
            fn from_edit_text(text: &str) -> Result<Self, String> {
                Err(format!("{} is not editable as text: {text}", stringify!($t)))
            }
        }
    )*};
}

shown_only!(
    Align,
    Justify,
    kivact_types::FlexDirection,
    kivact_types::Position,
    kivact_types::Overflow,
    kivact_types::Visibility,
    kivact_types::ScrollSnap,
    kivact_types::ScrollAxes,
    kivact_types::Orientation,
    crate::grid::Tracks,
    crate::grid::GridAutoFlow,
    crate::grid::GridPlacement,
    crate::effect::BorderStyle,
    kivact_types::Isolation,
    crate::paint::BorderImage,
    super::kinds::ScrollbarWidth,
    kivact_types::Cursor,
    crate::effect::Filter,
    crate::effect::BlendMode,
    crate::paint::Mask,
    crate::effect::Shape,
    crate::draw::LocalPaint,
    (f64, f64),
    (Length, Length),
    crate::transform::Transform,
    kivact_types::TransformStyle,
    kivact_types::BackfaceVisibility,
    kivact_types::WillChange,
    crate::text::FontKerning,
    Vec<crate::text::FontFeature>,
    Vec<crate::text::FontVariation>,
    crate::text::TextDecorationLine,
    crate::text::TextDecorationStyle,
    crate::text::TextOverflow,
    crate::text::TextTransform,
    crate::text::PaintOrder,
    kivact_runtime::Transition
);

impl EditText for String {
    fn to_edit_text(&self) -> String {
        self.clone()
    }

    fn from_edit_text(text: &str) -> Result<Self, String> {
        Ok(text.trim().to_owned())
    }
}

impl<T: EditText> EditText for Box<T> {
    fn to_edit_text(&self) -> String {
        (**self).to_edit_text()
    }

    fn from_edit_text(text: &str) -> Result<Self, String> {
        T::from_edit_text(text).map(Box::new)
    }
}

fn expected(what: &str, text: &str) -> String {
    format!("expected {what}, got {text}")
}
