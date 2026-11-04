//! Role: Passive list inputs, policies and observable item state.

use kivact_core::custom::PartState;
use kivact_core::state::Binding;
use kivact_core::state::IntoText;
use kivact_core::style::Icon;
use kivact_core::view::{Modifiers, Orientation, Reactive};

/// A labelled logical choice whose identity belongs to application data.
///
/// Official keyed Select, Tabs and Radio builders accept these options. Labels
/// can change without changing identity; disabled options keep their keys.
/// The caller owns ordering and filtering through the options value, while
/// the selected binding can retain a key that is currently absent.
///
/// The label is a constant, a closure, a `Signal` or a `message(..)` value.
/// An option is plain data, so it can be built anywhere (a handler, a memo, a
/// test) and the builders read the label reactively, so a localized label
/// follows a locale switch. Two labels are equal when they are equal
/// constants or the same reactive source, so renaming a constant label
/// reaches a mounted row.
///
/// An option may also carry an [`icon`](Self::icon) and a
/// [`description`](Self::description) for the component that draws it; the
/// behaviors only read the key, the label and the availability.
///
/// ```
/// # mod kivact {
/// #     //! Role: Supply facade paths for this documentation example.
/// #     pub extern crate kivact_interaction as interaction;
/// # }
/// use kivact::interaction::ListItem;
/// let item = ListItem::new("inbox", "Inbox").icon("inbox").description("12 unread");
/// assert_eq!(item.icon_value().and_then(|icon| icon.name()), Some("inbox"));
/// assert_eq!(item.description_value().as_deref(), Some("12 unread"));
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct ListItem<K> {
    pub(crate) key: K,
    pub(crate) label: Reactive<String>,
    pub(crate) disabled: bool,
    pub(crate) icon: Option<Icon>,
    pub(crate) description: Option<Reactive<String>>,
}

impl<K> ListItem<K> {
    /// Keep the label as given (no setup needed); borrowed `&str` labels are
    /// copied.
    pub fn new(key: K, label: impl IntoText) -> Self {
        Self {
            key,
            label: Reactive::new(label.into_text()),
            disabled: false,
            icon: None,
            description: None,
        }
    }

    /// Make the option unavailable: it keeps its key and place, but cannot
    /// be chosen. A constant: an option is plain data, so a changing
    /// availability is a new options value.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Show `icon` beside the label: a name the theme resolves
    /// (`"check"`) or an image [`Asset`](kivact_core::Asset).
    pub fn icon(mut self, icon: impl Into<Icon>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// A secondary line of text under the label, such as a shortcut hint or
    /// a count; like the label, a constant or a reactive value.
    pub fn description(mut self, text: impl IntoText) -> Self {
        self.description = Some(Reactive::new(text.into_text()));
        self
    }

    /// The application's identity of the option.
    pub fn key(&self) -> &K {
        &self.key
    }

    /// The label's current text; tracked when read inside a computation.
    #[track_caller]
    pub fn label(&self) -> String {
        self.label.get()
    }

    /// Whether the option is unavailable ([`disabled`](Self::disabled)).
    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    /// The icon given to [`icon`](Self::icon).
    pub fn icon_value(&self) -> Option<&Icon> {
        self.icon.as_ref()
    }

    /// The description's current text; tracked when read inside a
    /// computation.
    #[track_caller]
    pub fn description_value(&self) -> Option<String> {
        self.description.as_ref().map(Reactive::get)
    }
}

/// What happens at the first and last available choices.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ListBoundary {
    #[default]
    Wrap,
    Clamp,
}

/// Whether keyboard navigation also writes the selected key.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ListActivation {
    #[default]
    Automatic,
    Manual,
}

/// How an inclusive range changes the existing selection.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ListRangeMode {
    /// Replace the selection with the visible range.
    #[default]
    Replace,
    /// Append the visible range, preserving selections outside it.
    Add,
}

/// An explicit selection request, independent of pointer or keyboard input.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListGesture {
    /// Select only the target and establish a new anchor.
    Replace,
    /// Toggle the target and establish a new anchor.
    Toggle,
    /// Select an inclusive visible range from the anchor to the target.
    Extend { mode: ListRangeMode },
}

impl ListGesture {
    /// Interpret selection modifiers using the list's configured range policy.
    /// Shift extends with `shift` or `command_shift`; the configured command
    /// modifier alone toggles, and an ordinary press replaces the selection.
    pub fn from_modifiers(modifiers: Modifiers, policy: &ListPolicy) -> Self {
        let range = policy.range;
        let command = match range.command {
            ListCommandModifier::Control => modifiers.ctrl,
            ListCommandModifier::Meta => modifiers.meta,
            ListCommandModifier::ControlOrMeta => modifiers.ctrl || modifiers.meta,
        };
        if modifiers.shift {
            Self::Extend {
                mode: if command {
                    range.command_shift
                } else {
                    range.shift
                },
            }
        } else if command {
            Self::Toggle
        } else {
            Self::Replace
        }
    }
}

/// Fallback when the anchor no longer appears in the selectable visible order.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ListMissingAnchor {
    /// Start at the target, producing a one-item range.
    #[default]
    Target,
    /// Use the first selected key in visible order, otherwise the target.
    FirstSelected,
}

/// Which application command modifier toggles or adds to a selection.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ListCommandModifier {
    Control,
    Meta,
    #[default]
    ControlOrMeta,
}

/// Selection policies for adapting pointer and keyboard modifiers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ListRangePolicy {
    pub shift: ListRangeMode,
    pub command_shift: ListRangeMode,
    pub missing_anchor: ListMissingAnchor,
    pub command: ListCommandModifier,
}

impl Default for ListRangePolicy {
    fn default() -> Self {
        Self {
            shift: ListRangeMode::Replace,
            command_shift: ListRangeMode::Add,
            missing_anchor: ListMissingAnchor::Target,
            command: ListCommandModifier::ControlOrMeta,
        }
    }
}

/// The authoritative caller selection, with no internal selection mirror.
pub enum ListSelection<K: Clone + PartialEq + 'static> {
    Single(Binding<Option<K>>),
    Multiple(Binding<Vec<K>>),
}
impl<K: Clone + PartialEq + 'static> Copy for ListSelection<K> {}
impl<K: Clone + PartialEq + 'static> Clone for ListSelection<K> {
    fn clone(&self) -> Self {
        *self
    }
}

/// What choosing an item does.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ListMode {
    /// A press or Space selects with the gesture its modifiers ask for (a
    /// list box, a table's rows).
    #[default]
    Select,
    /// The native release, Enter, Space or assistive technology chooses once
    /// (tabs, radios, menus).
    Activate,
}

/// Shared navigation, activation, typeahead and range policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ListPolicy {
    /// Which arrow keys move through the logical order: the vertical ones
    /// ([`Orientation::Vertical`]), the horizontal ones
    /// ([`Orientation::Horizontal`]) or all four (`None`).
    /// `Some(Orientation::Unspecified)` is a usage error. A single axis is also
    /// published as the list's orientation.
    pub axis: Option<Orientation>,
    pub boundary: ListBoundary,
    pub activation: ListActivation,
    pub mode: ListMode,
    pub typeahead: bool,
    pub range: ListRangePolicy,
}
impl Default for ListPolicy {
    fn default() -> Self {
        Self {
            axis: Some(Orientation::Vertical),
            boundary: ListBoundary::Clamp,
            activation: ListActivation::Automatic,
            mode: ListMode::Select,
            typeahead: false,
            range: ListRangePolicy::default(),
        }
    }
}
impl ListPolicy {
    pub fn tabs() -> Self {
        Self {
            axis: Some(Orientation::Horizontal),
            boundary: ListBoundary::Wrap,
            mode: ListMode::Activate,
            ..Self::default()
        }
    }
    pub fn radio() -> Self {
        Self {
            axis: None,
            boundary: ListBoundary::Wrap,
            mode: ListMode::Activate,
            ..Self::default()
        }
    }
    pub fn menu() -> Self {
        Self {
            boundary: ListBoundary::Wrap,
            activation: ListActivation::Manual,
            mode: ListMode::Activate,
            typeahead: true,
            ..Self::default()
        }
    }
    pub fn listbox() -> Self {
        Self {
            activation: ListActivation::Manual,
            mode: ListMode::Activate,
            typeahead: true,
            ..Self::default()
        }
    }
}

/// Selection, active candidate and actual input state for an attached item.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ListItemState {
    pub input: PartState,
    pub selected: bool,
    pub active: bool,
}
