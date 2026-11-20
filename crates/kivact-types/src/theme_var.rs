//! Role: Theme variables: a typed name with a fallback, which a theme may set or alias.

use std::fmt;
use std::hash::{Hash, Hasher};

/// A theme variable, CSS's custom property (`var(--name)`): a value a
/// theme sets, with the fallback used where it does not. Declare it as a
/// `static` (its address is its identity) and give its value form as a
/// `const` beside it:
///
/// ```text
/// pub static BRAND_VAR: ThemeVar<Color> = ThemeVar::new("brand", rgb(0x7c3aed));
/// pub const BRAND: Color = Color::var(&BRAND_VAR);
/// // A component's variable that follows the core radius unless set.
/// pub static FIELD_RADIUS_VAR: ThemeVar<f64> = ThemeVar::alias("field.radius", &radius::MD_VAR);
/// pub const FIELD_RADIUS: Length = Length::var(&FIELD_RADIUS_VAR);
/// ```
///
/// Two variables are equal only when they are the same `static`.
pub struct ThemeVar<T: 'static> {
    name: &'static str,
    fallback: VarValue<T>,
}

/// What a variable holds: a value, or another variable of the same type
/// (an alias, CSS `--a: var(--b)`), followed when it is read.
#[derive(Clone, Debug, PartialEq)]
pub enum VarValue<T: 'static> {
    /// A value.
    Value(T),
    /// The value of another variable.
    Var(&'static ThemeVar<T>),
}

impl<T: 'static> ThemeVar<T> {
    /// A variable named `name` (diagnostics and DevTools) with `fallback`.
    pub const fn new(name: &'static str, fallback: T) -> Self {
        Self {
            name,
            fallback: VarValue::Value(fallback),
        }
    }

    /// A variable whose fallback is the value of `of`: a theme that sets
    /// `of` changes this one too, unless it sets this one.
    pub const fn alias(name: &'static str, of: &'static ThemeVar<T>) -> Self {
        Self {
            name,
            fallback: VarValue::Var(of),
        }
    }

    /// The name.
    pub const fn name(&self) -> &'static str {
        self.name
    }

    /// What the variable holds where a theme does not set it.
    pub const fn fallback(&self) -> &VarValue<T> {
        &self.fallback
    }
}

impl<T: 'static> From<&'static ThemeVar<T>> for VarValue<T> {
    fn from(var: &'static ThemeVar<T>) -> Self {
        VarValue::Var(var)
    }
}

/// The values of the pure types a variable may hold, from what converts
/// to them (`Theme::set(&radius::MD_VAR, 20)`).
macro_rules! value_from {
    ($t:ty: $($src:ty),*) => {$(
        impl From<$src> for VarValue<$t> {
            fn from(v: $src) -> Self {
                VarValue::Value(<$t>::from(v))
            }
        }
    )*};
}
value_from!(f64: f64, f32, i32, u32);
value_from!(u16: u16);
value_from!(crate::LineHeight: crate::LineHeight, f64);

impl<T: 'static> PartialEq for ThemeVar<T> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

impl<T: 'static> Eq for ThemeVar<T> {}

impl<T: 'static> Hash for ThemeVar<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::ptr::from_ref(self).hash(state);
    }
}

impl<T: 'static> fmt::Debug for ThemeVar<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "var(--{})", self.name)
    }
}

/// The identity of a variable (its address), the key a theme stores its
/// value under.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ThemeVarId(usize);

impl ThemeVarId {
    /// The identity of `var`.
    pub fn of<T: 'static>(var: &'static ThemeVar<T>) -> ThemeVarId {
        ThemeVarId(std::ptr::from_ref(var).cast::<()>() as usize)
    }
}
