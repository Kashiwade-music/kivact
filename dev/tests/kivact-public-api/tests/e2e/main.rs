//! Role: End-to-end test suite for Kivact public API.
//! module checks one behaviour on real windows of the Arvane host through
//! `kivact::test`. One binary links kivact, Arvane and wgpu once instead of
//! once per file; the windows share its one event loop.

#[cfg(feature = "embed")]
mod embed;
mod support;
mod theme_conformance;

mod apg_combobox;
mod apg_grid;
mod apg_listbox;
mod apg_radio_group;
mod apg_slider;
mod apg_window_splitter;
mod apg_tabs;
mod apg_tooltip;
mod apg_tree_view;
mod assets;
mod async_tests;
mod canvas;
mod canvas_worker;
mod components;
mod custom_text_field;
mod diagnostics;
mod disposal;
mod environment;
mod flows;
mod focus;
#[cfg(all(feature = "gpu-interop", any(windows, target_os = "linux")))]
mod gpu_interop;
mod grid;
mod hover;
mod i18n;
mod image_semantics;
mod input_platform;
mod interaction_calendar;
mod interaction_combobox;
mod interaction_delayed;
mod interaction_dialog;
mod interaction_drag_edit;
mod interaction_edit;
mod interaction_list_nav;
mod interaction_menu;
mod interaction_palette;
mod interaction_select;
mod interaction_slider;
mod interaction_splitter;
mod interaction_table;
mod interaction_tabs;
mod interaction_toast;
mod interaction_toggle;
mod interaction_tooltip;
mod interaction_tree;
mod interaction_zoom_pan;
mod keyboard;
mod landmark_roles;
mod layout;
mod layout_check;
mod layout_scroll;
mod layout_units;
mod lifecycle;
mod motion;
mod offscreen;
mod overlay;
mod paint;
mod parts;
mod path_algebra;
mod platform_paths;
mod pointer;
mod presence;
mod primitive;
mod relations;
mod runtime;
mod setup_functions;
mod style_paint;
mod task;
mod text_editing;
mod text_primitives;
mod text_style;
mod touch_scroll;
mod transform;
mod ui_command_palette;
mod ui_components;
mod ui_controls;
mod ui_data_table;
mod ui_field;
mod ui_standard;
mod ui_theming;
mod view;

mod launch_fonts;
mod launch_platform;
mod launch_support;

mod launch_real_window;
mod launch_window;

mod machine;
mod reactive_props;

mod apg_menu;
mod apg_menu_button;
mod apg_menubar;
