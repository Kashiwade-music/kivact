//! Role: The recipe type and the macro that builds one component's recipe table.
use kivact_core::style::{Style, Theme};
use kivact_core::view::{Variant, Variants};
use std::ops::Deref;
use std::sync::Arc;

/// What a recipe reads: the part's state and the component's variants
/// (the values given with `.variant(..)`, read by [`Look::variant`]).
///
/// A look beside the meanings a state carries (a button's
/// [`ButtonVariant`](super::ButtonVariant), a kit's tone, a shape, a
/// segmented radio group beside a plain one) is a [`Variant`] value: the
/// caller gives one to any standard component with `.variant(value)` (one
/// per type, a later one replacing it), and every recipe of that component
/// reads it with [`variant`](Self::variant). A theme that does not know the
/// type draws the plain look. The state's fields read through the look
/// (`look.selected`, `look.input`).
///
/// ```
/// use kivact::style::{Theme, rgb};
/// use kivact::ui::theme::{ButtonState, ButtonTheme, Look, Variant};
/// #[derive(Clone, Copy, Debug, PartialEq)]
/// struct Pill;
/// impl Variant for Pill {}
/// let theme = Theme::light().with(ButtonTheme::default().root(|_, look: &Look<ButtonState>, base| {
///     match look.variant::<Pill>() {
///         Some(Pill) => base.border_radius(999).background(rgb(0xe8eefc)),
///         None => base,
///     }
/// }));
/// # let _ = theme;
/// ```
#[derive(Clone)]
pub struct Look<S> {
    state: S,
    variants: Variants,
}

impl<S> Look<S> {
    /// `state` without variants: for a kit that draws its own part with a
    /// standard component's recipe, adding its variants with
    /// [`with_variant`](Self::with_variant).
    pub fn new(state: S) -> Self {
        Self {
            state,
            variants: Variants::default(),
        }
    }

    /// The look of `state` with the component's `variants`.
    pub(crate) fn of(state: S, variants: Variants) -> Self {
        Self { state, variants }
    }

    /// This look with `value`, replacing a value of its type.
    pub fn with_variant<V: Variant>(mut self, value: V) -> Self {
        self.variants = std::mem::take(&mut self.variants).with(value);
        self
    }

    /// The variant of type `V` the component was given with
    /// `.variant(..)`, if any (tracked when the value is reactive).
    pub fn variant<V: Variant>(&self) -> Option<V> {
        self.variants.get::<V>()
    }
}

impl<S> Deref for Look<S> {
    type Target = S;

    fn deref(&self) -> &S {
        &self.state
    }
}

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
