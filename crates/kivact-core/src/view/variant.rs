//! Role: Variants: the looks a component takes with `.variant(..)`, kept on its root node.

use std::any::{Any, TypeId};
use std::rc::Rc;

use kivact_runtime::Prop;

/// A look a component takes with `.variant(value)` and its recipes read: a
/// value of the theme's or kit's own type. Implemented explicitly
/// (`impl Variant for Pill {}`); not sealed, so a kit defines its own.
pub trait Variant: Clone + PartialEq + std::fmt::Debug + 'static {}

/// The variant values of one node, one per type (a later value of a type
/// replaces the earlier one). A value may be reactive.
#[derive(Clone, Default)]
pub struct Variants(Rc<Vec<(TypeId, Rc<dyn Any>)>>);

impl Variants {
    /// Set the value of type `V`, replacing an earlier one.
    pub(crate) fn set<V: Variant>(&mut self, value: Prop<V>) {
        let entries = Rc::make_mut(&mut self.0);
        entries.retain(|(ty, _)| *ty != TypeId::of::<V>());
        entries.push((TypeId::of::<V>(), Rc::new(value)));
    }

    /// These values with `value` set for its type (a constant), replacing an earlier one.
    pub fn with<V: Variant>(mut self, value: V) -> Self {
        self.set(Prop::constant(value));
        self
    }

    /// Merge `later` over these values (a later value of a type wins).
    pub(crate) fn merge(&mut self, later: &Variants) {
        let entries = Rc::make_mut(&mut self.0);
        for (ty, value) in later.0.iter() {
            entries.retain(|(existing, _)| existing != ty);
            entries.push((*ty, value.clone()));
        }
    }

    /// The value of type `V` the component was given, if any. Tracked when
    /// the value is reactive.
    #[track_caller]
    pub fn get<V: Variant>(&self) -> Option<V> {
        self.0
            .iter()
            .find(|(ty, _)| *ty == TypeId::of::<V>())
            .and_then(|(_, value)| value.downcast_ref::<Prop<V>>())
            .map(Prop::get)
    }
}
