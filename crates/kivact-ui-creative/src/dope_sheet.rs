//! Role: The dope sheet component over keyframe models.

use crate::editors::time::time_editor_builder;

time_editor_builder!(
    dope_sheet,
    crate::editors::time::model::Keyframe,
    crate::editors::shared::metrics::KEY_LANE,
    "A dope sheet that reports requests for application-owned items. Create it over read-only application-owned logical-time items, with the selection, playhead, zoom and scroll of `state`."
);
