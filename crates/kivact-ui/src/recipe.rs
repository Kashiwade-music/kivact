//! Role: The recipe type, its tables macro and the recipe_table! expansion.

use crate::variants::Look;
use kivact_core::style::{Style, Theme};
use std::sync::Arc;

/// A style derived from the theme and the actual state of a component's
/// part with the component's kit-defined variants ([`Look`]): what a
/// recipe table holds for one part.
///
/// A recipe returns only style (layout, paint, local `Style::paint`
/// drawing, and the hover, press, focus and disabled looks as state
/// styles); what a part shows is the component's, or a slot the caller
/// fills (`Tabs::item`, `Menu::item`, `Select::item`). Tables compare
/// their recipes by identity, so installing an unchanged table never
/// restyles.
pub struct Recipe<S> {
    pub(crate) f: Arc<RecipeFn<S>>,
    name: &'static str,
}

type RecipeFn<S> = dyn Fn(&Theme, &Look<S>) -> Style + Send + Sync;

impl<S> Clone for Recipe<S> {
    fn clone(&self) -> Self {
        Self {
            f: self.f.clone(),
            name: self.name,
        }
    }
}

impl<S, F> From<F> for Recipe<S>
where
    F: Fn(&Theme, &Look<S>) -> Style + Send + Sync + 'static,
{
    fn from(value: F) -> Self {
        Self {
            f: Arc::new(value),
            name: "",
        }
    }
}

impl<S> std::ops::Deref for Recipe<S> {
    type Target = dyn Fn(&Theme, &Look<S>) -> Style + Send + Sync;
    fn deref(&self) -> &Self::Target {
        &*self.f
    }
}

impl<S> Recipe<S> {
    pub(crate) fn same(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.f, &other.f)
    }

    pub(crate) fn named(mut self, name: &'static str) -> Self {
        self.name = name;
        self
    }

    /// The recipe's name (its table and field); empty for a recipe made
    /// without one.
    pub fn name(&self) -> &'static str {
        self.name
    }
}

/// A recipe table: one recipe per part and the tables of the parts'
/// components. A component reads a recipe with the part's `_recipe` method
/// (a table with `_recipes`); a theme changes one with the method of the
/// part's own name, which takes the current recipe and returns the new one,
/// so it wraps or replaces the recipes it changes and keeps the others. The
/// fields are private: that method is the only way to change a table.
macro_rules! recipe_table {
    (
        $(#[$meta:meta])*
        $name:ident {
            $( $(#[$fmeta:meta])* $field:ident ($read:ident) : $state:ty = $default:path, )*
        }
        $( tables { $( $(#[$tmeta:meta])* $table:ident ($tread:ident) : $tty:ty, )* } )?
    ) => {
        $(#[$meta])*
        #[derive(Clone)]
        pub struct $name {
            $( $field: $crate::recipe::Recipe<$state>, )*
            $( $( $table: $tty, )* )?
        }

        impl $name {
            pub(crate) fn default_table() -> &'static Self {
                #[expect(clippy::disallowed_types, reason = "Immutable global recipe defaults are initialized once and share stable recipe identities.")]
                static DEFAULT: std::sync::OnceLock<$name> = std::sync::OnceLock::new();
                DEFAULT.get_or_init(|| Self {
                    $( $field: $crate::recipe::Recipe::from(|theme: &kivact_core::style::Theme, look: &$crate::variants::Look<$state>| $default(theme, look)).named(concat!(stringify!($name), ".", stringify!($field))), )*
                    $( $( $table: Default::default(), )* )?
                })
            }

            $(
                $(#[$fmeta])*
                ///
                /// This reads the recipe; the method of the same name without
                /// `_recipe` replaces it.
                pub fn $read(&self) -> &$crate::recipe::Recipe<$state> {
                    &self.$field
                }

                $(#[$fmeta])*
                ///
                /// This replaces the recipe: `f` gets the current recipe and
                /// returns the new one, which may call it.
                pub fn $field<F>(mut self, f: impl FnOnce($crate::recipe::Recipe<$state>) -> F) -> Self
                where
                    F: Fn(&kivact_core::style::Theme, &$crate::variants::Look<$state>) -> kivact_core::style::Style + Send + Sync + 'static,
                {
                    self.$field = $crate::recipe::Recipe::from(f(self.$field))
                        .named(concat!(stringify!($name), ".", stringify!($field)));
                    self
                }
            )*
            $( $(
                $(#[$tmeta])*
                ///
                /// This reads the table; the method of the same name without
                /// `_recipes` replaces it.
                pub fn $tread(&self) -> &$tty {
                    &self.$table
                }

                $(#[$tmeta])*
                ///
                /// This replaces the table: `f` gets the current table and
                /// returns the new one.
                pub fn $table(mut self, f: impl FnOnce($tty) -> $tty) -> Self {
                    self.$table = f(self.$table);
                    self
                }
            )* )?
        }

        impl Default for $name {
            fn default() -> Self {
                Self::default_table().clone()
            }
        }

        impl PartialEq for $name {
            fn eq(&self, other: &Self) -> bool {
                true
                    $( && self.$field.same(&other.$field) )*
                    $( $( && self.$table == other.$table )* )?
            }
        }

        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(stringify!($name))
                    $( .field(stringify!($field), &std::sync::Arc::as_ptr(&self.$field.f).cast::<()>()) )*
                    $( $( .field(stringify!($table), &self.$table) )* )?
                    .finish()
            }
        }
    };
}

pub(crate) use recipe_table;
