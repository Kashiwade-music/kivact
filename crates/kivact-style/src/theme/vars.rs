//! Role: Theme variable values and their storage: the data a theme holds.

use std::fmt;
use std::sync::Arc;

use kivact_types::{LineHeight, ThemeVar, ThemeVarId, VarValue};

use super::sealed;
use crate::color_value::Color;

/// A font family list of a theme (`font::FAMILY_VAR`): a CSS family list,
/// or the system UI font.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FontFamily(Option<Arc<str>>);

impl FontFamily {
    /// The system UI font.
    pub const SYSTEM: FontFamily = FontFamily(None);

    /// The family list, `None` for the system UI font.
    pub fn as_str(&self) -> Option<&str> {
        self.0.as_deref()
    }
}

impl From<&str> for FontFamily {
    fn from(list: &str) -> Self {
        FontFamily(Some(Arc::from(list)))
    }
}

impl From<String> for FontFamily {
    fn from(list: String) -> Self {
        FontFamily(Some(Arc::from(list)))
    }
}

impl From<FontFamily> for VarValue<FontFamily> {
    fn from(v: FontFamily) -> Self {
        VarValue::Value(v)
    }
}

impl From<Color> for VarValue<Color> {
    fn from(v: Color) -> Self {
        VarValue::Value(v)
    }
}

/// What a theme stores for a variable.
#[doc(hidden)]
#[derive(Clone, Debug, PartialEq)]
pub enum VarSlot {
    Color(VarValue<Color>),
    Number(VarValue<f64>),
    Weight(VarValue<u16>),
    LineHeight(VarValue<LineHeight>),
    Family(VarValue<FontFamily>),
}

/// A type a [`ThemeVar`] holds: [`Color`], `f64` (lengths in px), `u16`
/// (font weights), [`LineHeight`] and [`FontFamily`]. Every one may be an
/// alias of another variable of its type, as a set value and as a fallback.
pub trait ThemeValue:
    Clone + PartialEq + fmt::Debug + Send + Sync + 'static + sealed::Sealed
{
    #[doc(hidden)]
    fn into_slot(value: VarValue<Self>) -> VarSlot;
    #[doc(hidden)]
    fn from_slot(slot: &VarSlot) -> Option<VarValue<Self>>;
    /// A value that is itself a variable (a `Color::var`), with its alpha.
    #[doc(hidden)]
    fn as_alias(&self) -> Option<(&'static ThemeVar<Self>, f64)> {
        None
    }
    /// The value with the alphas of the aliases followed applied.
    #[doc(hidden)]
    fn with_alias_alpha(self, _alpha: f64) -> Self {
        self
    }
    /// The value of a chain that does not end and whose first fallback is an alias.
    #[doc(hidden)]
    fn on_cycle() -> Self;
}

macro_rules! plain_value {
    ($($t:ty => $variant:ident, $cycle:expr;)*) => {$(
        impl sealed::Sealed for $t {}
        impl ThemeValue for $t {
            fn into_slot(value: VarValue<Self>) -> VarSlot {
                VarSlot::$variant(value)
            }
            fn from_slot(slot: &VarSlot) -> Option<VarValue<Self>> {
                match slot {
                    VarSlot::$variant(v) => Some(v.clone()),
                    _ => None,
                }
            }
            fn on_cycle() -> Self {
                $cycle
            }
        }
    )*};
}

plain_value! {
    f64 => Number, 0.0;
    u16 => Weight, 400;
    LineHeight => LineHeight, LineHeight::Normal;
    FontFamily => Family, FontFamily::SYSTEM;
}

impl sealed::Sealed for Color {}

impl ThemeValue for Color {
    fn into_slot(value: VarValue<Self>) -> VarSlot {
        VarSlot::Color(value)
    }
    fn from_slot(slot: &VarSlot) -> Option<VarValue<Self>> {
        match slot {
            VarSlot::Color(v) => Some(v.clone()),
            _ => None,
        }
    }
    fn as_alias(&self) -> Option<(&'static ThemeVar<Self>, f64)> {
        self.var_target()
    }
    fn with_alias_alpha(self, alpha: f64) -> Self {
        self.with_alpha(alpha)
    }
    fn on_cycle() -> Self {
        Color::BLACK
    }
}

/// The next variable an alias leads to, with the alpha it applies.
pub(super) fn next_of<T: ThemeValue>(held: &VarValue<T>) -> Option<(&'static ThemeVar<T>, f64)> {
    match held {
        VarValue::Var(next) => Some((next, 1.0)),
        VarValue::Value(v) => v.as_alias(),
    }
}

/// The variables a theme sets, sorted by identity.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct VarTable(pub(super) Vec<(ThemeVarId, &'static str, VarSlot)>);

impl VarTable {
    pub(crate) fn insert(&mut self, id: ThemeVarId, name: &'static str, value: VarSlot) {
        match self.0.binary_search_by_key(&id, |(k, _, _)| *k) {
            Ok(i) => self.0[i] = (id, name, value),
            Err(i) => self.0.insert(i, (id, name, value)),
        }
    }

    pub(crate) fn get(&self, id: ThemeVarId) -> Option<&VarSlot> {
        self.0
            .binary_search_by_key(&id, |(k, _, _)| *k)
            .ok()
            .map(|i| &self.0[i].2)
    }
}
