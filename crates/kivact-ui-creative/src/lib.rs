//! Role: Dense creative-tool controls, panels and editor surfaces.
//!
//! The only dependency is the public `kivact` facade. Components expose
//! their root through `Element`, so ordinary `Element` modifiers
//! apply to official and third-party components in exactly the same way.
//! Install [`theme`] on an App or with `kivact::style::provide_theme`.
//!
//! The texts the components show themselves (a window's close button, the
//! transport's button names) are English and read through
//! `kivact::i18n::message`: an App replaces them with
//! `App::messages`.
//!
//! [`dock`] arranges panels in tabbed containers and splits that the
//! application keeps in a [`DockLayout`].

#![forbid(unsafe_code)]

mod color;
mod fonts;
mod icons;
mod options;
mod recipe_parts;
mod recipes_collections;
mod recipes_controls;
mod recipes_feedback;
mod recipes_input;
mod recipes_metrics;
mod recipes_navigation;
mod recipes_overlays;
mod size;
mod theme;
mod variants;

pub use fonts::MONO_FAMILY_VAR;
pub use options::{Accent, Density, LabelTone};
pub use theme::{ThemeOptions, theme, theme_with};
pub use variants::NoticeVariant;
