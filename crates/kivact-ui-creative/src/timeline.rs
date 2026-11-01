//! Role: A dedicated timeline builder over logical-time models.

use crate::editors::time::time_editor_builder;

time_editor_builder!(
    Timeline,
    timeline,
    crate::editors::shared::model::Clip,
    crate::editors::shared::metrics::CLIP_LANE,
    "A timeline that reports requests for application-owned items.",
    "Create a timeline over read-only application-owned logical-time items, with the selection, playhead, zoom and scroll of `state`."
);
