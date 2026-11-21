//! Role: Soft's variant values for the standard components: each look is one type that a caller gives with `.variant(..)`.
use crate::space::Space;
use kivact::Prop;
use kivact::state::{IntoValue, Value};
use kivact::style::{Align, Color};
use kivact::ui::{AvatarShape, ControlSize, theme::Variant};
use kivact::view::Orientation;

/// Surface elevation of a Soft surface; `None` keeps the style without shadows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Prop)]
pub enum Elevation {
    #[default]
    /// Keep the existing style without adding shadows; the default.
    None,
    /// Use the theme's low-elevation shadow layers.
    Low,
    /// Use the theme's medium-elevation shadow layers.
    Med,
    /// Use the theme's high-elevation shadow layers.
    High,
}

impl Variant for Elevation {}

/// Background treatment of a Soft section, shared by sections and toolbars.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SectionVariant {
    #[default]
    /// Use the surface background; the default.
    Section,
    /// Leave the section background transparent.
    Transparent,
    /// Use the muted background.
    Muted,
}

/// Palette of a canonical card.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Prop)]
pub enum CardVariant {
    /// Default card surface.
    #[default]
    Default,
    /// Transparent surface.
    Transparent,
    /// Muted neutral surface.
    Muted,
    /// Blue palette.
    Blue,
    /// Cyan palette.
    Cyan,
    /// Gray palette.
    Gray,
    /// Green palette.
    Green,
    /// Orange palette.
    Orange,
    /// Pink palette.
    Pink,
    /// Purple palette.
    Purple,
    /// Red palette.
    Red,
    /// Teal palette.
    Teal,
    /// Yellow palette.
    Yellow,
}

impl Variant for CardVariant {}

/// Padding override of a card; the card uses the theme's card inset without it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CardPadding(pub Space);

impl Variant for CardPadding {}

/// Spacing of a Soft conversation surface.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Prop)]
pub enum ChatDensity {
    /// Tight conversation spacing.
    Compact,
    /// Balanced conversation spacing, the default.
    #[default]
    Balanced,
    /// Roomy conversation spacing.
    Spacious,
}

impl Variant for ChatDensity {}

/// Bubble background treatment.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Prop)]
pub enum ChatBubbleVariant {
    /// Filled message bubbles, the default.
    #[default]
    Filled,
    /// Unfilled message bubbles.
    Ghost,
}

impl Variant for ChatBubbleVariant {}

/// Elevation of a composer surface.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Prop)]
pub enum ComposerElevation {
    /// Composer without a shadow.
    None,
    /// Low composer shadow, the default.
    #[default]
    Low,
}

impl Variant for ComposerElevation {}

/// System message framing.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Prop)]
pub enum SystemMessageVariant {
    /// Plain system message framing, the default.
    #[default]
    Default,
    /// System message with divider framing.
    Divider,
}

impl Variant for SystemMessageVariant {}

/// The place of a disclosure chevron relative to its title.
///
/// Choose it with `kivact::ui::disclosure(..).variant(DisclosureChevron::Start)`;
/// other themes are ignored and the disclosure draws its normal look.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DisclosureChevron {
    /// Before the title, following the current reading direction.
    Start,
    /// After the title (default).
    #[default]
    End,
}

impl Variant for DisclosureChevron {}

/// Vertical padding for disclosure triggers and contents.
///
/// Without this variant a disclosure has no padding, unless it draws dividers, which use `Balanced`.
/// Choose it with `kivact::ui::disclosure(..).variant(DisclosureDensity::Compact)`;
/// other themes are ignored and the disclosure draws its normal look.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DisclosureDensity {
    /// Use the theme's smallest spacing step.
    Compact,
    /// Use the theme's standard spacing step (default).
    #[default]
    Balanced,
    /// Use the theme's larger spacing step.
    Spacious,
}

impl Variant for DisclosureDensity {}

/// Draw a separator before each grouped disclosure panel after the first when true.
///
/// Choose it with `kivact::ui::disclosure_group(..).variant(DisclosureDividers(true))`;
/// other themes are ignored and the disclosure draws its normal look.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DisclosureDividers(pub bool);

impl Variant for DisclosureDividers {}

/// Visual options of a spinner: its ink and track pairing and an optional explicit ink.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SpinnerVariant {
    /// Ink and track palette; accent by default.
    pub shade: SpinnerShade,
    /// Optional ink used by `SpinnerShade::Inherit`.
    pub ink: Option<Color>,
}

impl Variant for SpinnerVariant {}

/// The diameter of an avatar.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Prop)]
pub enum AvatarSize {
    /// 20-pixel diameter.
    Xsm,
    /// 24-pixel diameter.
    Sm,
    /// 36-pixel diameter, the default.
    #[default]
    Md,
    /// 48-pixel diameter.
    Lg,
    /// 128-pixel diameter.
    Xl,
    /// Requested diameter in pixels, snapped to the nearest supported size.
    Px(u16),
}

impl AvatarSize {
    pub(crate) fn side(self) -> f64 {
        const SIZES: [u16; 14] = [16, 20, 24, 32, 36, 40, 48, 60, 64, 72, 96, 128, 144, 180];
        let size = match self {
            Self::Xsm => 20,
            Self::Sm => 24,
            Self::Md => 36,
            Self::Lg => 48,
            Self::Xl => 128,
            Self::Px(size) => SIZES
                .into_iter()
                .min_by_key(|side| side.abs_diff(size))
                .unwrap_or(36),
        };
        f64::from(size)
    }
}

impl Variant for AvatarSize {}

/// The silhouette of the avatars of a canonical avatar group.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AvatarGroupShape(pub AvatarShape);

impl Variant for AvatarGroupShape {}

/// The visual container treatment of a standard alert.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Prop)]
pub enum BannerVariant {
    /// Rounded card surface, the default.
    #[default]
    Card,
    /// Section surface without the card radius.
    Section,
}

impl Variant for BannerVariant {}

/// Reactive visual density of an empty message, without announcement state.
#[derive(Clone, Debug, PartialEq)]
pub struct EmptyStateVariant {
    compact: Value<bool>,
}

impl EmptyStateVariant {
    /// Stack actions and use the compact spacing scale.
    pub fn compact(compact: impl IntoValue<bool>) -> Self {
        Self {
            compact: compact.into_value(),
        }
    }
}

impl Variant for EmptyStateVariant {}

/// Vertical spacing of item content in Soft collection recipes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Prop)]
pub enum ItemDensity {
    /// Use the smallest row padding.
    Compact,
    #[default]
    /// Use the standard row padding.
    Balanced,
    /// Use the largest row padding.
    Spacious,
}

impl ItemDensity {
    pub(crate) fn padding(self) -> i32 {
        match self {
            Self::Compact => 4,
            Self::Balanced => 8,
            Self::Spacious => 12,
        }
    }
}

impl Variant for ItemDensity {}

/// Vertical alignment of a row icon and its text content.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Prop)]
pub enum ItemAlign {
    #[default]
    /// Center the icon against the full content block.
    Center,
    /// Align the icon to the top of the content block.
    Start,
    /// Align the icon with the first text line.
    FirstLine,
}

impl Variant for ItemAlign {}

/// Arrangement of an item label and description.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Prop)]
pub enum ItemLayout {
    #[default]
    /// Place the description below the label.
    Stacked,
    /// Place the label and description on one row.
    Inline,
}

impl Variant for ItemLayout {}

/// Paint the row with the highlighted background when true.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ItemHighlighted(pub bool);

impl Variant for ItemHighlighted {}

/// Logical edge treatment of a Soft list.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Prop)]
pub enum ListEdge {
    #[default]
    /// Keep the list and its rows inset from the surrounding edge.
    Padded,
    /// Remove the outer list inset.
    Flush,
    /// Use the outset profile with the standard padded row geometry.
    Outset,
    /// Use padded rows without rounded outer corners.
    Bleed,
}

impl Variant for ListEdge {}

/// Show separators between the rows of a list when true.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ListDividers(pub bool);

impl Variant for ListDividers {}

/// Categorical colors for badges; use the standard tone for semantic statuses.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BadgeVariant {
    #[default]
    /// Use the neutral badge palette.
    Neutral,
    /// Use the blue categorical badge palette.
    Blue,
    /// Use the cyan categorical badge palette.
    Cyan,
    /// Use the green categorical badge palette.
    Green,
    /// Use the orange categorical badge palette.
    Orange,
    /// Use the pink categorical badge palette.
    Pink,
    /// Use the purple categorical badge palette.
    Purple,
    /// Use the red categorical badge palette.
    Red,
    /// Use the teal categorical badge palette.
    Teal,
    /// Use the yellow categorical badge palette.
    Yellow,
    /// Use the gray categorical badge palette.
    Gray,
}

impl Variant for BadgeVariant {}

/// Source text size.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Prop)]
pub enum CodeBlockSize {
    /// Use small source typography.
    Sm,
    #[default]
    /// Use standard source typography.
    Md,
}

impl Variant for CodeBlockSize {}

/// Outer width policy.
#[derive(Clone, Copy, Debug, Default, PartialEq, Prop)]
pub enum CodeBlockWidth {
    #[default]
    /// Fit the source block within the available width.
    Fit,
    /// Fill the available width.
    Fill,
    /// Use the specified outer width in pixels.
    Px(f64),
}

impl Variant for CodeBlockWidth {}

/// Source block framing.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Prop)]
pub enum CodeBlockContainer {
    #[default]
    /// Paint a bordered, rounded source card.
    Card,
    /// Render source without the outer card frame.
    Section,
}

impl Variant for CodeBlockContainer {}

/// Visual chrome of a standard drop target, independent of file validation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DropzoneVariant {
    /// Use the compact field treatment for the drop target.
    Field,
    #[default]
    /// Use the larger drop-area treatment.
    Area,
}

impl Variant for DropzoneVariant {}

/// Surface elevation of a joined action group.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ButtonGroupVariant(pub Elevation);

impl Variant for ButtonGroupVariant {}

/// Neutral surface treatment for a standard toolbar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ToolbarVariant(pub SectionVariant);

impl Variant for ToolbarVariant {}

impl Default for ToolbarVariant {
    fn default() -> Self {
        Self(SectionVariant::Transparent)
    }
}

/// Display typography independent of the heading's semantic level.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeadingDisplay {
    /// Use the largest display heading typography.
    D1,
    /// Use the medium display heading typography.
    D2,
    /// Use the smallest display heading typography.
    D3,
}

impl Variant for HeadingDisplay {}

/// Column arrangement for standard description lists.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Prop)]
pub enum MetadataColumns {
    #[default]
    /// Arrange description pairs in one column.
    Single,
    /// Arrange description pairs in the standard multi-column grid.
    Multi,
    /// Arrange description pairs in the specified number of columns.
    Count(u8),
}

impl Variant for MetadataColumns {}

/// Place terms beside or above their details.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Prop)]
pub enum LabelPosition {
    /// Place terms at the logical start beside their details.
    Start,
    /// Place terms above their details.
    Top,
}

impl Variant for LabelPosition {}

/// Fix the beside-label column width instead of its intrinsic maximum.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MetadataLabelWidth(pub f64);

impl Variant for MetadataLabelWidth {}

/// Arrange description pairs in a wrapping row or the selected vertical grid.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MetadataOrientation(pub Orientation);

impl Variant for MetadataOrientation {}

/// Reactive visual sizing of standard pagination; mode remains canonical.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PaginationSize(pub ControlSize);

impl Variant for PaginationSize {}

/// Corner profile supplied to the standard skeleton's variant hook.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SkeletonRadius {
    /// Use square corners.
    None,
    /// Use the zero-radius profile.
    R0,
    /// Use the theme inner radius.
    R1,
    /// Use the theme element radius.
    R2,
    #[default]
    /// Use the theme container radius.
    R3,
    /// Use the theme container radius for the larger profile.
    R4,
    /// Use the theme fully rounded radius.
    Rounded,
}

impl Variant for SkeletonRadius {}

/// Whether a static table row paints separators.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TableRowDividers(pub bool);

impl Variant for TableRowDividers {}

/// Whether a static table row receives hover paint.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TableRowHover(pub bool);

impl Variant for TableRowHover {}

/// Whether alternating static table rows receive stripe paint.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TableRowStriped(pub bool);

impl Variant for TableRowStriped {}

/// Vertical alignment of a static table cell's content.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TableCellAlign(pub Align);

impl Variant for TableCellAlign {}

/// Whether a static table cell paints a separator.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TableCellDivider(pub bool);

impl Variant for TableCellDivider {}

/// Dot treatment of a canonical badge used in a table's status lane.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RowStatusDot;

impl Variant for RowStatusDot {}

/// How a segmented radio group distributes horizontal space.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Prop)]
pub enum SegmentedLayout {
    #[default]
    /// Size each segment to its content.
    Hug,
    /// Let the segments share the available width.
    Fill,
}

impl Variant for SegmentedLayout {}

/// Reserve compact geometry for caller-provided icon-only segment content when true.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SegmentedIconOnly(pub bool);

impl Variant for SegmentedIconOnly {}

/// Show or hide the ancestor continuation guides of a tree when true (the default).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TreeGuides(pub bool);

impl Variant for TreeGuides {}

/// Spinner ink and track pairing.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SpinnerShade {
    /// Accent ink over the standard track, the default.
    #[default]
    Default,
    /// Secondary text ink over the standard track.
    Subtle,
    /// Light ink and translucent track for dark media.
    OnMedia,
    /// Explicit ink, or primary text ink when no override is supplied.
    Inherit,
}

impl Variant for SpinnerShade {}

/// Spacing and indicator sizing for progress steps.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Prop)]
pub enum StepDensity {
    /// Use compact step spacing and indicators.
    Compact,
    #[default]
    /// Use the standard step spacing and indicators.
    Balanced,
    /// Use spacious step spacing and indicators.
    Spacious,
}

impl Variant for StepDensity {}

/// Placement of step indicators relative to connector tracks.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Prop)]
pub enum StepIndicatorPosition {
    #[default]
    /// Keep each indicator separate from the connector track.
    Separated,
    /// Place indicators directly on the connector track.
    OnTrack,
}

impl Variant for StepIndicatorPosition {}

/// Visible details of the responsive collapsed stepper.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Prop)]
pub enum StepCollapsedVariant {
    #[default]
    /// Show the current step label and navigation controls.
    WithLabelAndControls,
    /// Show the current step label without navigation controls.
    WithLabel,
    /// Hide the current step label in the collapsed presentation.
    HiddenLabel,
}

impl Variant for StepCollapsedVariant {}

/// Typography of a breadcrumb trail.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BreadcrumbsVariant {
    #[default]
    /// Use the regular breadcrumb text size.
    Default,
    /// Use the smaller supporting-text breadcrumb size.
    Supporting,
}

impl Variant for BreadcrumbsVariant {}
