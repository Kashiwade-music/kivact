//! Role: Shared control values and edit snapshots.
/// A retained inspector panel. Identities must be unique within its group.
#[derive(Clone, Debug, PartialEq)]
pub struct InspectorPanel {
    pub id: u64,
    pub title: String,
    pub expanded: bool,
}
impl InspectorPanel {
    pub fn new(id: u64, title: impl Into<String>) -> Self {
        Self {
            id,
            title: title.into(),
            expanded: true,
        }
    }
    pub fn collapsed(mut self) -> Self {
        self.expanded = false;
        self
    }
}
