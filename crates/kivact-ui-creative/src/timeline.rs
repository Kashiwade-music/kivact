//! Role: The timeline component over logical-time models.

use crate::editors::time::time_editor_builder;

time_editor_builder!(
    timeline_editor,
    crate::editors::shared::model::Clip,
    crate::editors::shared::metrics::CLIP_LANE,
    "A timeline that reports requests for application-owned items. Create it over read-only application-owned logical-time items, with the selection, playhead, zoom and scroll of `state`."
);
