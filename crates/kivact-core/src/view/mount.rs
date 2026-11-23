//! Role: Mounting turns descriptions into retained host nodes.
//!
//! Mounting: descriptions become retained nodes plus one render effect per
//! dynamic prop. Each render effect only computes its value and hands it to
//! the engine writer, which records only values that differ.
//!
//! A builder is an unmounted description, and a `View` runs once, when it
//! mounts. `mount_host` passes the typed properties and the payload straight
//! to the engine's create and makes a Render effect for each dynamic value
//! that calls a comparing setter (an equal value writes nothing). Core keeps
//! no unsent prop or scene and no index of creates or prop sets; a row
//! without a parent yet is made as a detached engine root. While native
//! edits wait, the controlled value is kept from overwriting them by the
//! count of `TextChanged` events still to deliver (`crate::view::tree`).

mod attachments;
mod create;
mod diagnostics;
mod draw;
mod editing;
mod facts;
mod node;
mod part_input;
mod part_pointer;
mod parts;
pub(crate) mod region_collection;
mod region_template;
pub(crate) mod regions;
mod resolve;
mod semantics;
mod style;

pub(crate) use node::{mount_host, mount_host_with};
pub(crate) use style::{NodeStyle, bind_style};
