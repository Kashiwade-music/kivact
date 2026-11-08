//! Role: The DevTools notifications: the `Notification` enum and its topics.

use serde::{Deserialize, Serialize};

use crate::app::{Topic, WindowList};
use crate::common::Empty;
use crate::console::ErrorEntry;
use crate::overlay::PickEvent;
use crate::scripts::ShowTableParams;
use crate::tree::TreeChanged;
use crate::updates::{FrameInfo, TurnGraph};

macro_rules! protocol_notifications {
    ($( $(#[doc = $doc:literal])* $variant:ident => $wire:literal ($params:ty) in $topic:ident; )*) => {
        /// A notification from the agent: the JSON-RPC `method` and its `params`.
        #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
        #[serde(tag = "method", content = "params")]
        #[allow(clippy::large_enum_variant, reason = "one short-lived value per notification; boxing would only add noise")]
        pub enum Notification {
            $( $(#[doc = $doc])* #[serde(rename = $wire)] $variant($params), )*
        }
        impl Notification {
            /// The method name on the wire.
            pub fn method(&self) -> &'static str {
                match self { $( Notification::$variant(_) => $wire, )* }
            }
            /// The topic a connection subscribes to for this notification.
            pub fn topic(&self) -> Topic {
                match self { $( Notification::$variant(_) => Topic::$topic, )* }
            }
        }
    };
}

protocol_notifications! {
    /// The element tree of a window changed.
    TreeChanged => "tree_changed" (TreeChanged) in Tree;
    /// A recorded turn is complete.
    TurnRecorded => "turn_recorded" (TurnGraph) in Turns;
    /// A recorded frame was presented.
    FrameRecorded => "frame_recorded" (FrameInfo) in Frames;
    /// Pick mode hovered, selected or cancelled.
    Pick => "pick" (PickEvent) in Pick;
    /// The app reported a new error.
    ErrorReported => "error_reported" (ErrorEntry) in Errors;
    /// A key asked for DevTools while a DevTools app is already connected.
    FocusRequested => "focus_requested" (Empty) in Devtools;
    /// A script showed a table.
    ScriptTable => "script_table" (ShowTableParams) in Scripts;
    /// A window opened, closed or changed its title.
    WindowsChanged => "windows_changed" (WindowList) in Windows;
}
