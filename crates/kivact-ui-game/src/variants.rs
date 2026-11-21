//! Role: The console's variant types, which a standard component takes with `.variant(..)`.
//!
//! Each one is a look of a standard component. A theme that does not know
//! the type draws the component's plain look.
use kivact::Prop;
use kivact::style::Color;
use kivact::ui::theme::Variant;

/// Selectable plan cards: the console's look of a standard radio group,
/// `kivact::ui::radio_group(..).variant(ChoiceCard)`. Other themes draw the
/// plain radio group.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Prop)]
pub struct ChoiceCard;
impl Variant for ChoiceCard {}

/// The looks of standard tabs: `Navigation` is the vertical console
/// navigation with a blue selection edge, `Boxed` the segments in a shaded
/// box. Give one with `kivact::ui::tabs(..).variant(TabsVariant::Navigation)`;
/// other themes draw the plain tabs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Prop)]
pub enum TabsVariant {
    /// Vertical navigation with a blue selection edge.
    Navigation,
    /// Segments in a shaded box, the selected one raised on the surface.
    Boxed,
}
impl Variant for TabsVariant {}

/// Draw a standard button slanted, the console's parallelogram:
/// `kivact::ui::button(..).variant(Slant)`. Another theme draws the plain button.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Prop)]
pub struct Slant;
impl Variant for Slant {}

/// Draw a standard progress bar with moving diagonal stripes over its fill:
/// `kivact::ui::progress(..).moving(true).variant(Striped)`. Another theme
/// draws the plain bar.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Prop)]
pub struct Striped;
impl Variant for Striped {}

/// Console avatar dimensions and the application-selected fill:
/// `kivact::ui::avatar(..).variant(AvatarVariant { side, color })`.
#[derive(Clone, Copy, Debug, PartialEq, Prop)]
pub struct AvatarVariant {
    /// The side of the square avatar in logical pixels.
    pub side: f64,
    /// The fill of the avatar's facet.
    pub color: Color,
}
impl Variant for AvatarVariant {}

/// Status, version and environment badge treatments:
/// `kivact::ui::badge(..).variant(BadgeVariant::Solid)`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Prop)]
pub enum BadgeVariant {
    /// A badge with a diamond dot before its label.
    #[default]
    Dot,
    /// A tinted badge without a dot.
    Plain,
    /// A badge filled with its tone.
    Solid,
    /// A badge with a border in its tone.
    Outline,
}
impl Variant for BadgeVariant {}

/// A console loading row with an avatar and two caption placeholders:
/// `kivact::ui::skeleton(..).variant(SkeletonVariant::Rows)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Prop)]
pub enum SkeletonVariant {
    /// The loading row.
    Rows,
}
impl Variant for SkeletonVariant {}

/// Display typography for a numbered section's title:
/// `kivact::ui::heading(..).variant(HeadingVariant::Section)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Prop)]
pub enum HeadingVariant {
    /// The section title in the display type.
    Section,
}
impl Variant for HeadingVariant {}

/// A console tile for an application-formatted statistic:
/// `kivact::ui::stat(..).variant(StatVariant::Kpi)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Prop)]
pub enum StatVariant {
    /// The tile with a top rule in the tone.
    Kpi,
}
impl Variant for StatVariant {}

/// The application's segment count of a standard gauge: the gauge draws
/// `count` segments, and `.variant(GaugeSegments(count))` reads a value
/// that may change.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Prop)]
pub struct GaugeSegments(pub usize);
impl Variant for GaugeSegments {}

/// Console input adornments: `Inline` cells beside the field, `Adorned`
/// glyphs inset in it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Prop)]
pub enum InputGroupVariant {
    /// Adjacent cells beside the field.
    Inline,
    /// Glyphs inset at the field's edges.
    Adorned,
}
impl Variant for InputGroupVariant {}
