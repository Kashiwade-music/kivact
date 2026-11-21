//! Role: The Pop variants of the standard components: the values a component takes with `.variant(..)`.
//!
//! Another theme ignores every variant here and draws the plain look.

use crate::color;
use kivact::Prop;
use kivact::style::Color;
use kivact::ui::theme::Variant;

/// A semantic color, which badges, alerts, avatars, tags and patterns take.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Prop)]
pub enum Hue {
    /// The blue hue.
    #[default]
    Blue,
    /// The mint hue.
    Mint,
    /// The yellow hue.
    Yellow,
    /// The pink hue.
    Pink,
    /// The red hue.
    Red,
    /// The gray hue.
    Gray,
}

impl Variant for Hue {}

impl Hue {
    /// The tint and the text color of this hue.
    pub fn colors(self) -> (Color, Color) {
        match self {
            Hue::Blue => (color::BLUE_BACKGROUND, color::BLUE_FOREGROUND),
            Hue::Mint => (color::MINT_BACKGROUND, color::MINT_FOREGROUND),
            Hue::Yellow => (color::YELLOW_BACKGROUND, color::YELLOW_FOREGROUND),
            Hue::Pink => (color::PINK_BACKGROUND, color::PINK_FOREGROUND),
            Hue::Red => (color::RED_BACKGROUND, color::RED_FOREGROUND),
            Hue::Gray => (color::GRAY_BACKGROUND, color::GRAY_FOREGROUND),
        }
    }

    /// The solid accent of this hue.
    pub fn accent(self) -> Color {
        match self {
            Hue::Blue => color::BLUE,
            Hue::Mint => color::MINT,
            Hue::Yellow => color::YELLOW,
            Hue::Pink => color::PINK,
            Hue::Red => color::RED,
            Hue::Gray => color::TEXT_MUTED,
        }
    }
}

/// A Pop look of a standard button beyond its [`ButtonVariant`](kivact::ui::ButtonVariant):
/// `kivact::ui::button(..).variant(ButtonTone::Pink)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Prop)]
pub enum ButtonTone {
    /// The yellow highlight.
    Accent,
    /// The pink gradient.
    Pink,
    /// A quiet filled button.
    Soft,
    /// A translucent button over a picture.
    Glass,
    /// The navy button.
    Dark,
}

impl Variant for ButtonTone {}

/// Cut a standard progress bar into this many segments:
/// `kivact::ui::progress(..).variant(Segments(10))`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Prop)]
pub struct Segments(pub u16);

impl Variant for Segments {}

/// The shape of a Pop badge: a soft tint, a dot, a solid, a slanted label or a count.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Prop)]
pub enum BadgeVariant {
    /// A soft tint in the hue.
    #[default]
    Soft,
    /// A dot before the label.
    Dot,
    /// A solid fill.
    Solid,
    /// A slanted gradient label.
    Slant,
    /// A round count.
    Count,
}

impl Variant for BadgeVariant {}

/// A skewed section-number badge beside a standard semantic heading:
/// `kivact::ui::badge(..).variant(SectionNumber)`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Prop)]
pub struct SectionNumber;

impl Variant for SectionNumber {}

/// Diamond indicators above centered labels, with ruled progress connectors:
/// `kivact::ui::stepper(..).variant(DiamondSteps)`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Prop)]
pub struct DiamondSteps;

impl Variant for DiamondSteps {}

/// Compact business-conversation bubbles with a hidden visible author line:
/// `kivact::ui::chat_message(..).variant(ChatBubbleVariant)`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Prop)]
pub struct ChatBubbleVariant;

impl Variant for ChatBubbleVariant {}

/// A loading indicator of dots: `kivact::ui::spinner(..).variant(SpinnerVariant::Dots)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Prop)]
pub enum SpinnerVariant {
    /// Three bouncing colored dots.
    Dots,
}

impl Variant for SpinnerVariant {}

/// The diameter of an avatar, in logical px (at least 16).
#[derive(Clone, Copy, Debug, PartialEq, Prop)]
pub struct AvatarSize(pub f64);

impl Variant for AvatarSize {}

/// The hue of an avatar's gradient, in degrees.
#[derive(Clone, Copy, Debug, PartialEq, Prop)]
pub struct AvatarHue(pub f64);

impl Variant for AvatarHue {}

/// The hue of each weekday of a calendar, Monday first; `None` keeps the plain color.
#[derive(Clone, Copy, Debug, Default, PartialEq, Prop)]
pub struct CalendarWeekdayHues(pub [Option<Hue>; 7]);

impl Variant for CalendarWeekdayHues {}
