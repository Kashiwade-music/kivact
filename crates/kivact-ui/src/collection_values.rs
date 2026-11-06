//! Role: Define passive layout projections and theme metrics for collections.
use crate::values::ControlSize;
use kivact_core::style::Color;
use kivact_core::view::Orientation;

/// Logical edges on which a toolbar draws a divider.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolbarEdge {
    Top,
    Bottom,
    Start,
    End,
}

/// Passive toolbar colors and metrics supplied through [`crate::UiTokens`].
#[derive(Clone, Debug, PartialEq)]
pub struct ToolbarTokens {
    pub background: Color,
    pub border: Color,
    pub padding: f64,
    pub gap: f64,
    pub min_height: f64,
}
impl Default for ToolbarTokens {
    fn default() -> Self {
        Self {
            background: Color::TRANSPARENT,
            border: Color::TRANSPARENT,
            padding: 8.0,
            gap: 4.0,
            min_height: 32.0,
        }
    }
}

/// Layout passed to toolbar recipes.
#[derive(Clone, Debug, PartialEq)]
pub struct ToolbarState {
    pub orientation: Orientation,
    pub size: ControlSize,
    // xtask-allow(state-fields): Independent presentation or construction input, not interaction state.
    pub gap: Option<f64>,
    // xtask-allow(state-fields): Independent presentation or construction input, not interaction state.
    pub centered: bool,
    // xtask-allow(state-fields): Independent presentation or construction input, not interaction state.
    pub rtl: bool,
    pub dividers: Vec<ToolbarEdge>,
}

/// Theme overrides for tree rows.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct TreeTokens {
    pub marker_width: Option<f64>,
    pub indent: Option<f64>,
    pub padding: Option<f64>,
    pub gap: Option<f64>,
    pub row_height: Option<f64>,
    pub radius: Option<f64>,
}
/// Visual overrides for static lists and item rows.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct ItemListTokens {
    pub gap: Option<f64>,
    pub padding: Option<f64>,
    pub row_gap: Option<f64>,
    pub description_size: Option<f64>,
    pub title_lines: Option<u32>,
    pub description_lines: Option<u32>,
}
/// Theme overrides for static tables.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct TableTokens {
    pub cell_padding: Option<f64>,
    pub header_height: Option<f64>,
    pub font_size: Option<f64>,
}

// xtask-allow(state-fields): Read-only recipe projection of actual input, description-slot availability and activation capability; no independent runtime state.
/// Actual item-row input and independent construction facts exposed to recipes.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ItemRowState {
    pub input: kivact_core::custom::PartState,
    pub has_description: bool,
    pub actionable: bool,
    pub position: Option<ItemPosition>,
    pub marker: ItemListMarker,
    pub start: u32,
    pub title_lines: Option<u32>,
    pub description_lines: Option<u32>,
}
impl std::ops::Deref for ItemRowState {
    type Target = kivact_core::custom::PartState;
    fn deref(&self) -> &Self::Target {
        &self.input
    }
}

/// Actual zero-based position and collection size; this record owns no ordering state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ItemPosition {
    pub index: usize,
    pub count: usize,
}
/// Structural marker used by a semantic static list.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ItemListMarker {
    #[default]
    None,
    Disc,
    Circle,
    Decimal,
}
kivact_core::state::constant_value!(ItemListMarker);
