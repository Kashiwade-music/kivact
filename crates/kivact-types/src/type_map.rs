//! Role: A map from a type to one value of that type, shared across threads or kept on one.

use std::any::{Any, TypeId};
use std::fmt;
use std::rc::Rc;
use std::sync::Arc;

/// Writes the two maps: the same code over `Arc` (Send + Sync values) and
/// `Rc` (UI thread values).
macro_rules! type_map {
    ($(#[$doc:meta])* $name:ident, $entry:ident, $ptr:ident, $dyn:ty, $($bound:tt)+) => {
        #[derive(Clone)]
        struct $entry {
            id: TypeId,
            value: $ptr<$dyn>,
            eq: fn(&dyn Any, &dyn Any) -> bool,
            debug: fn(&dyn Any, &mut fmt::Formatter<'_>) -> fmt::Result,
        }

        $(#[$doc])*
        #[derive(Clone, Default)]
        pub struct $name(Option<$ptr<Vec<$entry>>>);

        impl $name {
            /// An empty map; it allocates nothing until a value is added.
            pub const fn new() -> Self {
                Self(None)
            }

            /// The map with `value` in it, replacing a value of the same type.
            #[must_use]
            pub fn with<T: PartialEq + fmt::Debug + $($bound)+>(mut self, value: T) -> Self {
                fn eq<T: PartialEq + 'static>(a: &dyn Any, b: &dyn Any) -> bool {
                    a.downcast_ref::<T>() == b.downcast_ref::<T>()
                }
                fn debug<T: fmt::Debug + 'static>(
                    value: &dyn Any,
                    f: &mut fmt::Formatter<'_>,
                ) -> fmt::Result {
                    match value.downcast_ref::<T>() {
                        Some(value) => value.fmt(f),
                        None => Ok(()),
                    }
                }
                let entry = $entry {
                    id: TypeId::of::<T>(),
                    value: $ptr::new(value),
                    eq: eq::<T>,
                    debug: debug::<T>,
                };
                let entries = $ptr::make_mut(self.0.get_or_insert_with(|| $ptr::new(Vec::new())));
                match entries.iter_mut().find(|e| e.id == entry.id) {
                    Some(slot) => *slot = entry,
                    None => entries.push(entry),
                }
                self
            }

            /// The value of type `T`, borrowed.
            pub fn get<T: 'static>(&self) -> Option<&T> {
                self.find::<T>().and_then(|e| {
                    let value: &dyn Any = &*e.value;
                    value.downcast_ref::<T>()
                })
            }

            /// The value of type `T`, shared.
            pub fn get_shared<T: $($bound)+>(&self) -> Option<$ptr<T>> {
                self.find::<T>()
                    .and_then(|e| $ptr::downcast::<T>($ptr::clone(&e.value)).ok())
            }

            /// Whether the map holds no value.
            pub fn is_empty(&self) -> bool {
                self.0.as_ref().is_none_or(|entries| entries.is_empty())
            }

            fn find<T: 'static>(&self) -> Option<&$entry> {
                self.0.as_ref()?.iter().find(|e| e.id == TypeId::of::<T>())
            }

            fn entries(&self) -> &[$entry] {
                self.0.as_deref().map_or(&[][..], Vec::as_slice)
            }
        }

        impl PartialEq for $name {
            /// Equal when they hold equal values of the same types (a shared
            /// table is equal to itself at once).
            fn eq(&self, other: &Self) -> bool {
                if let (Some(a), Some(b)) = (&self.0, &other.0)
                    && $ptr::ptr_eq(a, b)
                {
                    return true;
                }
                let (a, b) = (self.entries(), other.entries());
                a.len() == b.len()
                    && a.iter().all(|x| {
                        b.iter().any(|y| {
                            x.id == y.id
                                && ($ptr::ptr_eq(&x.value, &y.value)
                                    || (x.eq)(&*x.value, &*y.value))
                        })
                    })
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                struct Value<'a>(&'a $entry);
                impl fmt::Debug for Value<'_> {
                    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                        (self.0.debug)(&*self.0.value, f)
                    }
                }
                f.debug_list().entries(self.entries().iter().map(Value)).finish()
            }
        }
    };
}

type_map!(
    /// A map from a type to one `Send + Sync` value, shared by its clones
    /// across threads (a theme's extensions).
    SharedTypeMap, SharedEntry, Arc, dyn Any + Send + Sync, Send + Sync + 'static
);

type_map!(
    /// A map from a type to one value, shared by its clones on one thread
    /// (a component's variants).
    LocalTypeMap, LocalEntry, Rc, dyn Any, 'static
);

#[cfg(test)]
mod tests {
    //! Role: Tests of the type maps.

    use super::{LocalTypeMap, SharedTypeMap};
    use std::sync::Arc;

    #[test]
    fn a_value_replaces_one_of_its_type() {
        let map = SharedTypeMap::new().with(1_u32).with(2_u32);
        assert_eq!(map.get::<u32>(), Some(&2));
        let map = map.with("a");
        assert_eq!(map.get::<u32>(), Some(&2));
    }

    #[test]
    fn maps_with_equal_values_are_equal() {
        let a = LocalTypeMap::new().with(1_u8).with("x");
        let b = LocalTypeMap::new().with("x").with(1_u8);
        assert_eq!(a, b);
        assert_ne!(a, LocalTypeMap::new().with(2_u8).with("x"));
        assert_eq!(LocalTypeMap::new(), LocalTypeMap::new());
    }

    #[test]
    fn get_shared_shares_the_value() {
        let m = SharedTypeMap::new().with(String::from("a"));
        assert!(Arc::ptr_eq(
            &m.get_shared::<String>().unwrap(),
            &m.get_shared::<String>().unwrap()
        ));
    }
}
