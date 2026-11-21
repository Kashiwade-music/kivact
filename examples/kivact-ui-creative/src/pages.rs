//! Role: The catalog's pages: one module per page, and the helper modules they share.

use catalog_shell::Page;

mod animation_model;
mod areas_docking;
mod arrange;
mod arrange_timeline;
mod arrangement_model;
mod audio;
mod audio_samples;
mod audio_values;
mod buttons;
mod chrome;
mod color_grading_scopes;
mod color_picker;
mod common;
mod controls;
mod dialogs_popovers;
mod dope_sheet_graph;
mod editor_images;
mod editors;
mod feedback;
mod indicators;
mod indicators_labels;
mod jobs;
mod jobs_log;
mod knobs_faders;
mod layer_stack;
mod layout;
mod layout_reducers;
mod media_pool;
mod media_pool_data_table;
mod menus;
mod meters;
mod mixer_console;
mod node_editor;
mod node_model;
mod numeric_fields;
mod numeric_samples;
mod outliner;
mod overview;
mod property_editor;
mod selection_input;
mod transport;
mod txn;
mod viewport_3d;

/// Every page of the catalog, in list order.
pub fn pages() -> Vec<Page> {
    vec![
        overview::page(),
        buttons::page(),
        numeric_fields::page(),
        selection_input::page(),
        indicators_labels::page(),
        transport::page(),
        knobs_faders::page(),
        meters::page(),
        mixer_console::page(),
        property_editor::page(),
        areas_docking::page(),
        outliner::page(),
        media_pool_data_table::page(),
        layer_stack::page(),
        arrange_timeline::page(),
        dope_sheet_graph::page(),
        node_editor::page(),
        color_grading_scopes::page(),
        color_picker::page(),
        viewport_3d::page(),
        menus::page(),
        dialogs_popovers::page(),
        jobs_log::page(),
    ]
}
