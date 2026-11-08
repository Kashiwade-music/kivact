//! Role: Server of the Kivact DevTools inside a target app.

mod agent;
mod config;
mod connection;
mod discovery;
mod files;
mod image;
mod incoming;
mod launcher;
mod review_store;
mod server;
mod session;
mod source;

pub use agent::{Agent, AgentConfig};
pub use image::{base64, encode_png};
pub use incoming::{Incoming, Responder};
pub use review_store::{NewComment, ReviewStore};
