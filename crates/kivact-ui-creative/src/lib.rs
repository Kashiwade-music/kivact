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

mod audio;
mod bezier;
mod color;
mod color_popup;
mod color_wheel;
mod curve_preview;
mod dock;
mod editors;
mod feedback;
mod fonts;
mod icons;
mod key_glyph;
mod layout;
mod numeric;
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

pub use audio::view_state::ChannelState;
pub use audio::{Meter, Transport, TransportState, meter, transport};
pub use bezier::{BezierCurve, CubicBezier, bezier_curve};
pub use color_popup::{ColorPopup, color_popup};
pub use color_wheel::{ColorModel, ColorWheel, color_from_hsv, color_to_hsv, color_wheel};
pub use curve_preview::{CurvePreview, curve_preview};
pub use dock::{
    Dock, DockAxis, DockChild, DockLayout, DockNode, DockPlacement, DockSide, DockTab, DockTabs,
    dock,
};
pub use feedback::{
    SplitButton, ToolWindowApi, ToolWindowOptions, WindowPosition, popup_button, split_button,
};
pub use fonts::MONO_FAMILY_VAR;
pub use key_glyph::KeyGlyph;
pub use layout::{Panel, media_tile, panel, tree_row};
pub use numeric::{
    Fader, Knob, Number, NumericKind, PropertyState, ValueSlider, evaluate_expression, fader, knob,
    number, value_slider,
};
pub use options::{Accent, Density, LabelTone};
pub use theme::{ThemeOptions, theme, theme_with};
pub use variants::NoticeVariant;
