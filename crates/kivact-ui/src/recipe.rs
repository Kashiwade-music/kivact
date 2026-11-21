//! Role: The recipe type and the macro that builds one component's recipe table.
use crate::theme::Look;
use kivact_core::style::{Style, Theme};
use std::sync::Arc;

type RecipeFn<S> = dyn Fn(&Theme, &Look<S>) -> Style + Send + Sync;

/// A style derived from the theme and the actual state of a component's
/// part, with the component's variants ([`Look`]). A theme changes it with
/// the method of the part's name on the component's `<Name>Theme`, which
/// receives the style the previous recipe returned.
pub struct Recipe<S> {
    f: Arc<RecipeFn<S>>,
    name: &'static str,
    overridden: bool,
}

impl<S> Clone for Recipe<S> {
    fn clone(&self) -> Self {
        Self {
            f: self.f.clone(),
            name: self.name,
            overridden: self.overridden,
        }
    }
}

impl<S: 'static> Recipe<S> {
    pub(crate) fn new(
        name: &'static str,
        f: impl Fn(&Theme, &Look<S>) -> Style + Send + Sync + 'static,
    ) -> Self {
        Self {
            f: Arc::new(f),
            name,
            overridden: false,
        }
    }

    /// The style of the part in `theme` for `look`.
    pub fn style(&self, theme: &Theme, look: &Look<S>) -> Style {
        (self.f)(theme, look)
    }

    /// The recipe's name (`<Name>Theme.<part>`), shown by DevTools.
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// Whether a theme replaced or wrapped the default recipe.
    pub(crate) fn is_overridden(&self) -> bool {
        self.overridden
    }

    pub(crate) fn layer(
        self,
        f: impl Fn(&Theme, &Look<S>, Style) -> Style + Send + Sync + 'static,
    ) -> Self {
        let previous = self.f;
        Self {
            f: Arc::new(move |theme, look| f(theme, look, previous(theme, look))),
            name: self.name,
            overridden: true,
        }
    }

    pub(crate) fn same(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.f, &other.f)
    }
}

/// One component's recipe table: one recipe per part and the component's
/// non-value settings. `<part>(f)` layers `f` over the current recipe;
/// `<part>_recipe()` reads it; `<setting>(v)` replaces a setting and
/// `<setting>_value()` reads it.
macro_rules! recipe_table {
    (
        $(#[$meta:meta])*
        $name:ident {
            $( $(#[$fmeta:meta])* $field:ident ($read:ident) : $state:ty = $default:path, )*
        }
        $( settings {
            $( $(#[$smeta:meta])* $setting:ident ($sread:ident) : $sty:ty = $sdefault:expr, )*
        } )?
    ) => {
        $(#[$meta])*
        #[derive(Clone)]
        pub struct $name {
            $( $field: $crate::recipe::Recipe<$state>, )*
            $( $( $setting: $sty, )* )?
        }
        impl $name {
            fn default_table() -> &'static Self {
                static DEFAULT: kivact_types::cell::SharedDeferred<$name> =
                    kivact_types::cell::SharedDeferred::new();
                DEFAULT.get_or_init(|| Self {
                    $( $field: $crate::recipe::Recipe::new(
                        concat!(stringify!($name), ".", stringify!($field)),
                        $default,
                    ), )*
                    $( $( $setting: $sdefault, )* )?
                })
            }
            $(
                $(#[$fmeta])*
                ///
                /// This reads the recipe.
                pub fn $read(&self) -> &$crate::recipe::Recipe<$state> {
                    &self.$field
                }
                $(#[$fmeta])*
                ///
                /// This layers `f` over the current recipe: `f` gets the
                /// theme, the look and the style the current recipe returns.
                pub fn $field(
                    mut self,
                    f: impl Fn(&kivact_core::style::Theme, &$crate::theme::Look<$state>, kivact_core::style::Style)
                        -> kivact_core::style::Style + Send + Sync + 'static,
                ) -> Self {
                    self.$field = self.$field.layer(f);
                    self
                }
            )*
            $( $(
                $(#[$smeta])*
                ///
                /// This reads the setting.
                pub fn $sread(&self) -> &$sty {
                    &self.$setting
                }
                $(#[$smeta])*
                ///
                /// This replaces the setting.
                pub fn $setting(mut self, value: $sty) -> Self {
                    self.$setting = value;
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
                true $( && self.$field.same(&other.$field) )*
                    $( $( && self.$setting == other.$setting )* )?
            }
        }
        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(stringify!($name))
                    $( .field(stringify!($field), &self.$field.name()) )*
                    $( $( .field(stringify!($setting), &self.$setting) )* )?
                    .finish()
            }
        }
        impl kivact_core::style::ThemeExtension for $name {
            fn fallback(_: &kivact_core::style::Theme) -> Self {
                Self::default()
            }
        }
    };
}
pub(crate) use recipe_table;
