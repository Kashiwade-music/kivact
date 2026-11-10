//! Role: The recent values each kind of shared node property was written with,
//! shared between the nodes that write an equal value (repeated components).
use kivact_types::cell::Recent;
use kivact_types::{FocusTarget, LayoutProps, PaintProps, Semantics, StateStyles, TextStyle};

/// The recent values of the shared node properties of one window.
#[derive(Default)]
pub(crate) struct RecentProps {
    pub(crate) layout: Recent<LayoutProps>,
    pub(crate) paint: Recent<PaintProps>,
    pub(crate) text: Recent<TextStyle>,
    pub(crate) states: Recent<StateStyles>,
    pub(crate) semantics: Recent<Semantics<FocusTarget>>,
}

/// A node property stored shared ([`crate::contract::NodeProperties`]).
pub(crate) trait SharedProp: PartialEq + Sized {
    /// The recent values of this kind.
    fn recent(props: &mut RecentProps) -> &mut Recent<Self>;
}

impl SharedProp for LayoutProps {
    fn recent(props: &mut RecentProps) -> &mut Recent<Self> {
        &mut props.layout
    }
}

impl SharedProp for PaintProps {
    fn recent(props: &mut RecentProps) -> &mut Recent<Self> {
        &mut props.paint
    }
}

impl SharedProp for TextStyle {
    fn recent(props: &mut RecentProps) -> &mut Recent<Self> {
        &mut props.text
    }
}

impl SharedProp for StateStyles {
    fn recent(props: &mut RecentProps) -> &mut Recent<Self> {
        &mut props.states
    }
}

impl SharedProp for Semantics<FocusTarget> {
    fn recent(props: &mut RecentProps) -> &mut Recent<Self> {
        &mut props.semantics
    }
}
