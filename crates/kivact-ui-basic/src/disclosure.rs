//! Role: Collapsible: a disclosure trigger and its content without a look of its own.

use kivact::Binding;
use kivact::control::{Node, NodeBuilder, column, node};
use kivact::interaction::use_disclosure;
use kivact::primitive::NodeExt;
use kivact::view::{IntoView, View, show};
use std::rc::Rc;

/// A trigger and the content it shows (collapsible.tsx), without a look of
/// its own. Created by [`collapsible`]; [`Collapsible::attach`] it to the
/// caller's trigger.
pub struct Collapsible {
    open: Binding<bool>,
    content: Option<Rc<dyn Fn() -> View>>,
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
pub fn collapsible(open: impl Into<Binding<bool>>) -> Collapsible {
    Collapsible {
        open: open.into(),
        content: None,
    }
}

impl Collapsible {
    /// What the section shows while it is open, built each time it opens
    /// and removed while it is closed.
    pub fn content<V: IntoView>(mut self, content: impl Fn() -> V + 'static) -> Self {
        self.content = Some(Rc::new(move || content().into_view()));
        self
    }

    /// Make `trigger` the disclosure's trigger (`Role::Button` with
    /// `expanded`: click, Enter and Space toggle `open`) and return a column
    /// of the trigger over the content, which takes the modifiers.
    pub fn attach<T: NodeBuilder + IntoView>(self, trigger: T) -> Node {
        let details = use_disclosure(self.open);
        let trigger = trigger.part(details.trigger());
        let content = self.content;
        column((
            trigger,
            show(details.shown(), move || {
                node()
                    .part(details.content())
                    .child(content.as_ref().map(|content| content()))
            }),
        ))
    }
}
