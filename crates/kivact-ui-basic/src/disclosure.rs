//! Role: Collapsible: a disclosure trigger and its content without a look of its own.

use kivact::interaction::use_disclosure;
use kivact::state::Binding;
use kivact::state::IntoBinding;
use kivact::view::{Node, NodeBuilder, column, node};

use kivact::view::{IntoView, View, show};

/// A trigger and the content it shows (collapsible.tsx), without a look of
/// its own. Created by [`collapsible`]; [`Collapsible::attach`] it to the
/// caller's trigger.
pub struct Collapsible {
    open: Binding<bool>,
    content: Option<kivact::view::ViewFn>,
}

/// A collapsible section shown while `open` is true.
///
/// ```no_run
/// use kivact::prelude::*;
/// use kivact_ui_basic as basic;
///
/// fn details() -> impl IntoView {
///     let open = signal(false);
///     basic::collapsible(open)
///         .content(|| text("More about this."))
///         .attach(button("Details"))
/// }
/// ```
#[track_caller]
pub fn collapsible(open: impl kivact::state::IntoBinding<bool>) -> Collapsible {
    Collapsible {
        open: open.into_binding(),
        content: None,
    }
}

impl Collapsible {
    /// What the section shows while it is open, built each time it opens
    /// and removed while it is closed.
    pub fn content<V: IntoView>(mut self, content: impl kivact::view::IntoViewFn<V>) -> Self {
        self.content = Some(content.into_view_fn());
        self
    }

    /// Make `trigger` the disclosure's trigger (`Role::Button` with
    /// `expanded`: click, Enter and Space toggle `open`) and return a column
    /// of the trigger over the content, which takes the modifiers.
    pub fn attach<T: NodeBuilder + IntoView>(self, trigger: T) -> impl kivact::view::Component {
        let details = use_disclosure(self.open);
        let trigger = trigger.with_node(|root| root.part(details.trigger()));
        let content = self.content;
        column((
            trigger,
            show(details.is_open(), move || {
                node()
                    .with_node(|root| root.part(details.content()))
                    .child(content.as_ref().map(|content| content.call(())))
            }),
        ))
    }
}
