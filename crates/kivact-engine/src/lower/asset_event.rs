//! Role: Asset diagnostics emitted while lowering scene values.
use kivact_types::NodeId;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum AssetEvent {
    AssetFailed { node: NodeId, message: String },
}
