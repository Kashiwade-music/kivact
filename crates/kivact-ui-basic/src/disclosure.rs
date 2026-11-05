//! Role: Basic visual variants for standard disclosures.
/// Visual treatment of a standard disclosure.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum DisclosureVariant {
    /// Content without accordion padding, used with a caller-owned trigger.
    #[default]
    Plain,
}
impl kivact::ui::Variant for DisclosureVariant {}
