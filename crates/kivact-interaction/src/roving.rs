//! Role: Share roving membership and requested-focus bindings.
mod machine;
mod registry;
mod requested;
pub(crate) use registry::{Member, ObservationInput as Observation, Roving};
