//! Role: Soft collection appearance values independent of row interaction.
use kivact::state::{IntoValue, Value};
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ItemDensity {
    Compact,
    #[default]
    Balanced,
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
pub enum ItemAlign {
    #[default]
    Center,
    Start,
    FirstLine,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ItemLayout {
    #[default]
    Stacked,
    Inline,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ListEdge {
    #[default]
    Padded,
    Flush,
    Outset,
    Bleed,
}

impl kivact::ui::Variant for ItemDensity {}
impl kivact::ui::Variant for ItemAlign {}
impl kivact::ui::Variant for ItemLayout {}
impl kivact::ui::Variant for ListEdge {}
kivact::state::constant_value!(ItemDensity, ItemAlign, ItemLayout, ListEdge);
// xtask-allow(state-fields): Independent readonly visual inputs for density, alignment, layout and highlighting; the variant owns no interaction or lifecycle state.
/// Reactive visual choices for a canonical item row.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ItemLook {
    pub(crate) density: Option<Value<ItemDensity>>,
    pub(crate) align: Option<Value<ItemAlign>>,
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
    pub fn density(mut self, value: impl IntoValue<ItemDensity>) -> Self {
        self.density = Some(value.into_value());
        self
    }
    pub fn align(mut self, value: impl IntoValue<ItemAlign>) -> Self {
        self.align = Some(value.into_value());
        self
    }
    pub fn layout(mut self, value: impl IntoValue<ItemLayout>) -> Self {
        self.layout = Some(value.into_value());
        self
    }
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
    pub fn density(mut self, value: impl IntoValue<ItemDensity>) -> Self {
        self.density = value.into_value();
        self
    }
    pub fn dividers(mut self, value: impl IntoValue<bool>) -> Self {
        self.dividers = value.into_value();
        self
    }
    pub fn edge(mut self, value: impl IntoValue<ListEdge>) -> Self {
        self.edge = value.into_value();
        self
    }
}
