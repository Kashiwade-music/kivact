//! Role: End-to-end test suite for Kivact public API.
//! module checks one behaviour on real windows of the Arvane host through
//! `kivact::test`. One binary links kivact, Arvane and wgpu once instead of
//! once per file; the windows share its one event loop.

mod apg_breadcrumb;
#[cfg(feature = "embed")]
mod embed;
mod support;
mod theme_conformance;
mod ui_navigation;

mod apg_combobox;
mod apg_grid;
mod apg_listbox;
mod apg_radio_group;
mod apg_slider;
mod apg_spinbutton;
mod apg_tabs;
mod apg_tooltip;
mod apg_tree_view;
mod apg_window_splitter;
mod assets;
mod async_tests;
mod borders_backgrounds;
mod canvas;
mod canvas_worker;
mod color_spaces;
mod component_macro;
mod components;
mod compositor;
mod custom_text_field;
#[cfg(feature = "devtools")]
mod devtools;
mod diagnostics;
mod disposal;
#[cfg(all(feature = "gpu-interop", any(windows, target_os = "linux")))]
mod drawable_export_gpu;
mod drawables;
mod effects_filters;
mod environment;
mod flows;
mod focus;
#[cfg(all(feature = "gpu-interop", any(windows, target_os = "linux")))]
mod gpu_interop;
mod grid;
mod hover;
mod i18n;
mod image_semantics;
mod images_animated;
mod indicators;
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
mod interaction_state;
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
mod metrics_and_description;
mod motion;
mod motion_easing;
mod node_ref;
mod offscreen;
mod overlay;
mod paint;
mod parts;
mod path_algebra;
mod platform_paths;
mod pointer;
mod presence;
mod presence_modes;
mod primitive;
mod relations;
mod runtime;
mod scroll_timeline;
mod setup_functions;
mod shaders;
mod shared_layout;
mod style_paint;
mod task;
mod text_editing;
mod text_effects;
mod text_primitives;
mod text_style;
mod timeline;
mod touch_scroll;
mod transform;
mod transform_3d;
mod ui_combobox;
mod ui_command_palette;
mod ui_components;
mod ui_controls;
mod ui_data_table;
mod ui_field;
mod ui_standard;
mod ui_theming;
mod view;
mod windows;

mod launch_fonts;
mod launch_platform;
mod launch_support;

mod launch_real_window;
mod launch_window;

mod machine;
mod reactive_props;

mod apg_accordion;
mod apg_menu;
mod apg_menu_button;
mod apg_menubar;

mod apg_carousel;
mod apg_toolbar;
mod ui_calendar;
mod ui_carousel;
mod ui_collections;
mod ui_disclosure_surfaces;
mod ui_dropzone;
mod ui_toolbar;
mod viewport;
#[cfg(all(feature = "gpu-interop", any(windows, target_os = "linux")))]
mod viewport_gpu_interop;

mod apg_alert;
mod apg_date_picker;
mod feedback;

mod ui_chat_code;
