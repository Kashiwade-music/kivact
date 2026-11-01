//! Role: A dedicated dope sheet builder over logical-time models.

use crate::editors::time::time_editor_builder;

time_editor_builder!(
    DopeSheet,
    dope_sheet,
    crate::editors::time::model::Keyframe,
    crate::editors::shared::metrics::KEY_LANE,
    "A dope sheet that reports requests for application-owned items.",
    "Create a dope sheet over read-only application-owned logical-time items, with the selection, playhead, zoom and scroll of `state`."
);
