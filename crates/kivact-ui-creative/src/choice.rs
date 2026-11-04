//! Role: The Creative segmented look of the standard radio group.

/// Exclusive segments in a shaded row: the Creative look of a standard radio
/// group, `kivact::ui::radio_group(..).with_node(move |node| node.orientation(Orientation::Horizontal))
/// .variant(Segmented)`. A segmented control is a
/// radio group, not tabs; another theme draws the plain radio group.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Segmented;

impl kivact::ui::Variant for Segmented {}
