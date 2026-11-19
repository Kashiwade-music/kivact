//! Role: Define passive layout projections and theme metrics for collections.

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
