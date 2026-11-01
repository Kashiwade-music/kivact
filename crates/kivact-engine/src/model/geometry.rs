//! Role: Read Model-owned layout columns through their normal geometry API.
use crate::model::Model;
use crate::tree::node::NodeIndex;
impl Model {
    #[inline]
    pub(crate) fn layout_at(&self, index: NodeIndex) -> Option<kivact_layout::NodeLayout> {
        self.layout_columns.layout_at(index.0)
    }
}
