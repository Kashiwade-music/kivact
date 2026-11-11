//! Role: Answer the hit test's ink query from the paint list's kept inks.
use super::ink::drawn;
use crate::model::Model;
use crate::model::interact::hit::{HitInk, HitReach};
use crate::tree::node::HostNode;
use kivact_layout::NodeLayout;
use kivact_types::NodeId;

impl HitInk for Model {
    fn hit_ink(&self, id: NodeId, n: &HostNode, layout: &NodeLayout) -> HitReach {
        match drawn(self, id, n, layout, self.paint.list.ink_gen) {
            None => HitReach::Nothing,
            Some(d) if d.beyond => HitReach::Anywhere,
            Some(d) => HitReach::Within(d.rect),
        }
    }
}
