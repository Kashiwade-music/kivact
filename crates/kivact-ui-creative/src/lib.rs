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
mod control_style;
mod controls;
mod curve_preview;
mod dock;
mod dock_style;
mod dope_sheet;
mod editors;
mod feedback;
mod gradient;
mod icons;
mod key_glyph;
mod layout;
mod numeric;
mod outliner;
mod outliner_data;
mod panel_group;
mod picker;
mod pie;
mod property_row;
mod range;
mod recipes;
mod recipes_collections;
mod recipes_controls;
mod recipes_feedback;
mod recipes_indicators;
mod recipes_metrics;
mod tabs;
mod theme;
mod timeline;
mod tokens;

pub use crate::audio::view_state::ChannelState;
pub use crate::editors::arrangement::view_state::ArrangementState;
pub use crate::editors::graph::model::Interpolation;
pub use crate::editors::graph::model::{
    Curve, CurveKey, CurveTangent, GraphRequest, GraphView, KeyMove,
};
pub use crate::editors::graph::view_state::GraphState;
pub use crate::editors::node::model::{
    NodeControl, NodeControlValue, NodeData, NodeLink, NodeRequest, NodeSocket,
};
pub use crate::editors::node::view_state::NodeState;
pub use crate::editors::shared::model::{Clip, EditorContext, EditorFocus, ItemTime, Lane};
pub use crate::editors::time::model::{Keyframe, TimeRequest};
pub use crate::editors::time::view_state::TimeState;
pub use crate::outliner::view_state::OutlinerState;
pub use audio::{Meter, MixerStrip, Transport, TransportState, meter, mixer_strip, transport};
pub use bezier::{BezierCurve, CubicBezier, bezier_curve};
pub use color::ColorName;
pub use color_popup::color_popup;
pub use color_wheel::{ColorModel, ColorWheel, color_from_hsv, color_to_hsv, color_wheel};
pub use curve_preview::{CurvePreview, curve_preview};
pub use dock::{
    Dock, DockAxis, DockChild, DockLayout, DockNode, DockPlacement, DockSide, DockTab, DockTabs,
    dock,
};
pub use dope_sheet::{DopeSheet, dope_sheet};
pub use editors::arrangement::{
    Arrangement, ArrangementRequest, AutomationPoint, TimelineMarker, TimelineTool, TimelineTrack,
    TrackFlag, TrackKind, arrangement,
};
pub use editors::graph::{GraphEditor, graph_editor};
pub use editors::node::{NodeEditor, node_editor};
pub use editors::shared::model::EditorEvent;
pub use editors::shared::ruler::RulerTicks;
pub use feedback::{
    NoticeVariant, SplitButton, ToolWindow, ToolWindowApi, ToolWindowOptions, WindowPosition,
    popup_button, split_button, tool_window, tool_window_button,
};
pub use gradient::{GradientEditor, GradientStop, gradient_editor};
pub use key_glyph::KeyGlyph;
pub use layout::{Panel, media_tile, panel, tree_row};
pub use numeric::{
    Numeric, NumericKind, PropertyState, evaluate_expression, fader, knob, number, value_slider,
};
pub use outliner::{Outliner, Restriction, RestrictionFlag, TreeDrop, TreeItem, outliner};
pub use panel_group::{InspectorPanel, panel_group};
pub use picker::{PickerItem, SearchPicker, search_picker};
pub use pie::{PieEntry, PieMenu, PieMode, PieState, pie_menu};
pub use property_row::{PropertyRow, PropertyStateButton, property_row, property_state_button};
pub use range::{Range, range};
pub use tabs::{DocumentTabs, document_tabs};
pub use theme::{ThemeOptions, theme, theme_with};
pub use timeline::{Timeline, timeline_editor};
pub use tokens::{Accent, Density, LabelTone, Tokens};
