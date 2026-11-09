//! Role: Soft collection appearance values independent of row interaction.
use kivact::state::{IntoValue, Value};
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
/// Vertical spacing of item content in Soft collection recipes.
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
/// Vertical alignment of a row icon and its text content.
pub enum ItemAlign {
    #[default]
    /// Center the icon against the full content block.
    Center,
    /// Align the icon to the top of the content block.
    Start,
    /// Align the icon with the first text line.
    FirstLine,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
/// Arrangement of an item label and description.
pub enum ItemLayout {
    #[default]
    /// Place the description below the label.
    Stacked,
    /// Place the label and description on one row.
    Inline,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
/// Logical edge treatment of a Soft list.
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

impl kivact::ui::Variant for ItemDensity {}
impl kivact::ui::Variant for ItemAlign {}
impl kivact::ui::Variant for ItemLayout {}
impl kivact::ui::Variant for ListEdge {}
kivact::state::constant_value!(ItemDensity, ItemAlign, ItemLayout, ListEdge);
/// Reactive visual choices for a canonical item row.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ItemLook {
    // xtask-allow(state-fields): option: absent means the component's default
    pub(crate) density: Option<Value<ItemDensity>>,
    // xtask-allow(state-fields): option: absent means the component's default
    pub(crate) align: Option<Value<ItemAlign>>,
    // xtask-allow(state-fields): option: absent means the component's default
    pub(crate) layout: Option<Value<ItemLayout>>,
    pub(crate) highlighted: Value<bool>,
}
impl Default for ItemLook {
    fn default() -> Self {
        Self {
            density: None,
            align: None,
            layout: None,
            highlighted: false.into_value(),
        }
    }
}
impl kivact::ui::Variant for ItemLook {}
impl ItemLook {
    /// Set reactive row padding.
    pub fn density(mut self, value: impl IntoValue<ItemDensity>) -> Self {
        self.density = Some(value.into_value());
        self
    }
    /// Set reactive alignment of leading content against the text block.
    pub fn align(mut self, value: impl IntoValue<ItemAlign>) -> Self {
        self.align = Some(value.into_value());
        self
    }
    /// Arrange the label and description vertically or inline.
    pub fn layout(mut self, value: impl IntoValue<ItemLayout>) -> Self {
        self.layout = Some(value.into_value());
        self
    }
    /// Paint the row with the highlighted background when enabled.
    pub fn highlighted(mut self, value: impl IntoValue<bool>) -> Self {
        self.highlighted = value.into_value();
        self
    }
}
/// Reactive density, separators and logical edge treatment of a static list.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ListLook {
    pub(crate) density: Value<ItemDensity>,
    pub(crate) dividers: Value<bool>,
    pub(crate) edge: Value<ListEdge>,
}
impl Default for ListLook {
    fn default() -> Self {
        Self {
            density: ItemDensity::Balanced.into_value(),
            dividers: false.into_value(),
            edge: ListEdge::Padded.into_value(),
        }
    }
}
impl kivact::ui::Variant for ListLook {}
impl ListLook {
    /// Set the padding used by list rows.
    pub fn density(mut self, value: impl IntoValue<ItemDensity>) -> Self {
        self.density = value.into_value();
        self
    }
    /// Show separators between list rows when enabled.
    pub fn dividers(mut self, value: impl IntoValue<bool>) -> Self {
        self.dividers = value.into_value();
        self
    }
    /// Set the logical inset or extension at the list edges.
    pub fn edge(mut self, value: impl IntoValue<ListEdge>) -> Self {
        self.edge = value.into_value();
        self
    }
}
