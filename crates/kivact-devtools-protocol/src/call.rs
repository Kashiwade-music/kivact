//! Role: The DevTools methods, declared once: the `Call` and `Reply` enums, wire names and typed markers.

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::a11y::{A11yTree, AuditReport, FocusInfo, TabOrder, WindowParams};
use crate::app::{AppInfo, InitializeParams, InitializeResult, SubscribeParams, WindowList};
use crate::common::Empty;
use crate::config::DevtoolsConfig;
use crate::console::{ErrorList, ListErrorsParams};
use crate::emulation::Emulation;
use crate::input::{ImeState, InjectInputParams, InputLog, ListInputsParams};
use crate::layout::LayoutLint;
use crate::motion::{AnimationList, TimeScaleParams};
use crate::node::{GetNodeParams, NodeDetail};
use crate::overlay::{HighlightParams, Image, OverlaySettings, ScreenshotParams, StartPickParams};
use crate::perf::{
    Counters, DiffSnapshotsParams, ExportTraceParams, ExportedFiles, OwnerTree, SnapshotDiff,
    SnapshotInfo,
};
use crate::query::{FoundNode, FoundNodes, MaybeNode, RoleQuery, TestIdQuery, TextQuery};
use crate::review::{
    AddCommentParams, EditCommentParams, ExportedReview, RemoveCommentParams, ReviewDocument,
    ReviewItem,
};
use crate::scripts::ShowTableParams;
use crate::style::{
    ClearStyleOverridesParams, ForceStateParams, SetStyleOverrideParams, StyleOverride,
    StyleOverrideList, ThemeTokenList,
};
use crate::tasks::TaskList;
use crate::tree::{GetTreeParams, TreeSnapshot};
use crate::updates::{FrameList, ListFramesParams, ListTurnsParams, RecordingSettings, TurnList};

/// A method of the protocol: its wire name and the types it takes and returns.
pub trait Method {
    /// The method name on the wire.
    const NAME: &'static str;
    /// The `params` type.
    type Params: Serialize + DeserializeOwned;
    /// The `result` type.
    type Result: Serialize + DeserializeOwned;
}

macro_rules! protocol_methods {
    ($( $(#[doc = $doc:literal])* $variant:ident => $wire:literal ($params:ty) -> $result:ty; )*) => {
        /// A request from a client: the JSON-RPC `method` and its `params`.
        #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
        #[serde(tag = "method", content = "params")]
        #[allow(clippy::large_enum_variant, reason = "one short-lived value per call; boxing would only add noise")]
        pub enum Call {
            $( $(#[doc = $doc])* #[serde(rename = $wire)] $variant($params), )*
        }
        /// The result of one call.
        #[derive(Clone, Debug, PartialEq)]
        #[allow(clippy::large_enum_variant, reason = "one short-lived value per call; boxing would only add noise")]
        pub enum Reply {
            $( $(#[doc = $doc])* $variant($result), )*
        }
        impl Call {
            /// The method name on the wire.
            pub fn method(&self) -> &'static str {
                match self { $( Call::$variant(_) => $wire, )* }
            }
        }
        impl Reply {
            /// The JSON-RPC `result` value.
            pub fn into_value(self) -> serde_json::Value {
                match self {
                    $( Reply::$variant(value) => serde_json::to_value(value)
                        .expect("invariant: protocol values serialize"), )*
                }
            }
        }
        /// Every method name, in declaration order.
        pub const METHOD_NAMES: &[&str] = &[$($wire),*];
        $(
            $(#[doc = $doc])*
            pub struct $variant;
            impl Method for $variant {
                const NAME: &'static str = $wire;
                type Params = $params;
                type Result = $result;
            }
        )*
    };
}

protocol_methods! {
    /// Start a session (the first request on a connection).
    Initialize => "initialize" (InitializeParams) -> InitializeResult;
    /// Choose the notification topics of this connection (replaces the earlier choice).
    Subscribe => "subscribe" (SubscribeParams) -> Empty;
    /// The app: name, process, build and workspace root.
    GetAppInfo => "get_app_info" (Empty) -> AppInfo;
    /// The DevTools configuration of the target, with defaults filled in.
    GetConfig => "get_config" (Empty) -> DevtoolsConfig;
    /// Every open window.
    ListWindows => "list_windows" (Empty) -> WindowList;
    /// The element tree of one window.
    GetTree => "get_tree" (GetTreeParams) -> TreeSnapshot;
    /// Everything known about one element.
    GetNode => "get_node" (GetNodeParams) -> NodeDetail;
    /// The one element with this role (and name); fails when there is none or more than one.
    GetByRole => "get_by_role" (RoleQuery) -> FoundNode;
    /// The element with this role (and name), or none; fails when there is more than one.
    QueryByRole => "query_by_role" (RoleQuery) -> MaybeNode;
    /// Like `get_by_role`, but waits up to `timeout_ms` for the element to appear.
    FindByRole => "find_by_role" (RoleQuery) -> FoundNode;
    /// Every element with this role (and name).
    AllByRole => "all_by_role" (RoleQuery) -> FoundNodes;
    /// The one element with this text; fails when there is none or more than one.
    GetByText => "get_by_text" (TextQuery) -> FoundNode;
    /// The element with this text, or none; fails when there is more than one.
    QueryByText => "query_by_text" (TextQuery) -> MaybeNode;
    /// Like `get_by_text`, but waits up to `timeout_ms` for the element to appear.
    FindByText => "find_by_text" (TextQuery) -> FoundNode;
    /// Every element with this text.
    AllByText => "all_by_text" (TextQuery) -> FoundNodes;
    /// The one element with this test id; fails when there is none or more than one.
    GetByTestId => "get_by_test_id" (TestIdQuery) -> FoundNode;
    /// The element with this test id, or none; fails when there is more than one.
    QueryByTestId => "query_by_test_id" (TestIdQuery) -> MaybeNode;
    /// Like `get_by_test_id`, but waits up to `timeout_ms` for the element to appear.
    FindByTestId => "find_by_test_id" (TestIdQuery) -> FoundNode;
    /// Every element with this test id.
    AllByTestId => "all_by_test_id" (TestIdQuery) -> FoundNodes;
    /// Highlight an element on screen (`node: None` clears it).
    Highlight => "highlight" (HighlightParams) -> Empty;
    /// Choose the debugging overlays drawn over every window.
    SetOverlays => "set_overlays" (OverlaySettings) -> Empty;
    /// The debugging overlays that are on now.
    GetOverlays => "get_overlays" (Empty) -> OverlaySettings;
    /// Start pick mode in a window: the next click selects an element instead of reaching the app.
    StartPick => "start_pick" (StartPickParams) -> Empty;
    /// Leave pick mode.
    StopPick => "stop_pick" (Empty) -> Empty;
    /// A PNG of a window or one element.
    Screenshot => "screenshot" (ScreenshotParams) -> Image;
    /// Override one style property of an element, a call site or a theme token.
    SetStyleOverride => "set_style_override" (SetStyleOverrideParams) -> StyleOverride;
    /// Remove one style override, or all of them.
    ClearStyleOverrides => "clear_style_overrides" (ClearStyleOverridesParams) -> Empty;
    /// Every style override that is set.
    ListStyleOverrides => "list_style_overrides" (Empty) -> StyleOverrideList;
    /// Force interactive states (hover, pressed, ...) on an element; an empty list releases them.
    ForceState => "force_state" (ForceStateParams) -> Empty;
    /// The theme tokens of the main window.
    ListThemeTokens => "list_theme_tokens" (Empty) -> ThemeTokenList;
    /// Start recording turns, frames and input (replaces the settings when already recording).
    StartRecording => "start_recording" (RecordingSettings) -> Empty;
    /// Finish the turns that are waiting for their frame, then stop recording.
    StopRecording => "stop_recording" (Empty) -> Empty;
    /// What is being recorded (all false when not recording).
    GetRecording => "get_recording" (Empty) -> RecordingSettings;
    /// Recorded turns with their cause graphs; `wait_ms` waits for a new one.
    ListTurns => "list_turns" (ListTurnsParams) -> TurnList;
    /// Recorded frames.
    ListFrames => "list_frames" (ListFramesParams) -> FrameList;
    /// Counts of live objects, owners, timers and so on.
    GetCounters => "get_counters" (Empty) -> Counters;
    /// Save the list of live owners and objects.
    TakeSnapshot => "take_snapshot" (Empty) -> SnapshotInfo;
    /// What was added and removed between two snapshots.
    DiffSnapshots => "diff_snapshots" (DiffSnapshotsParams) -> SnapshotDiff;
    /// The owner tree of a window.
    GetOwners => "get_owners" (WindowParams) -> OwnerTree;
    /// Write the recorded turns and frames as a Chrome trace file.
    ExportTrace => "export_trace" (ExportTraceParams) -> ExportedFiles;
    /// The accessibility tree of a window.
    GetA11yTree => "get_a11y_tree" (WindowParams) -> A11yTree;
    /// Accessibility problems found in a window.
    AuditA11y => "audit_a11y" (WindowParams) -> AuditReport;
    /// The focused element and the focus history of a window.
    GetFocus => "get_focus" (WindowParams) -> FocusInfo;
    /// The Tab order of a window.
    GetTabOrder => "get_tab_order" (WindowParams) -> TabOrder;
    /// Layout problems found in a window.
    LintLayout => "lint_layout" (WindowParams) -> LayoutLint;
    /// Inject input into a window; answers after the turn that handled it.
    InjectInput => "inject_input" (InjectInputParams) -> Empty;
    /// The recorded input events.
    ListInputs => "list_inputs" (ListInputsParams) -> InputLog;
    /// The text input (IME) state of a window.
    GetImeState => "get_ime_state" (WindowParams) -> ImeState;
    /// The running animations.
    ListAnimations => "list_animations" (Empty) -> AnimationList;
    /// Slow down, pause or restore the animation clock.
    SetTimeScale => "set_time_scale" (TimeScaleParams) -> Empty;
    /// The tasks, resources, actions and timers.
    ListTasks => "list_tasks" (Empty) -> TaskList;
    /// The errors the app reported.
    ListErrors => "list_errors" (ListErrorsParams) -> ErrorList;
    /// The environment emulation that is set.
    GetEmulation => "get_emulation" (Empty) -> Emulation;
    /// Emulate another environment (theme, locale, scale, touch); replaces the whole setting.
    SetEmulation => "set_emulation" (Emulation) -> Emulation;
    /// The review of this session.
    GetReview => "get_review" (Empty) -> ReviewDocument;
    /// Add a comment to an element (or a window) of the review.
    AddComment => "add_comment" (AddCommentParams) -> ReviewItem;
    /// Change the text of a comment.
    EditComment => "edit_comment" (EditCommentParams) -> ReviewItem;
    /// Remove a comment.
    RemoveComment => "remove_comment" (RemoveCommentParams) -> Empty;
    /// Write the review as `review.json` and `review.md` and return their paths.
    ExportReview => "export_review" (Empty) -> ExportedReview;
    /// Show a table in the DevTools app (for scripts).
    ShowTable => "show_table" (ShowTableParams) -> Empty;
}
