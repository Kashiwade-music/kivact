//! Role: Kit-defined variants every standard component passes to its recipes.
//!
//! Every standard builder has the same `.variant(value)`: one value per
//! type, passed to every recipe of the component through [`Look`], read
//! with `look.variant::<T>()`, and ignored by a theme that does not know
//! the type. A look within one theme (a button's [`ButtonVariant`](super::ButtonVariant),
//! Creative's segmented radio group, a boxed tabs list) is such a value,
//! not another component or a subtree theme. The values are read when the
//! builder is mounted and its styles first run.
use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

/// A look a standard component takes with `.variant(value)` and its recipes
/// read with [`Look::variant`]: a value of the theme's or kit's own type.
///
/// A component keeps one value per type, a later `.variant(..)` of the same
/// type replacing the earlier one, and a theme whose recipes do not know the
/// type draws the plain look. The trait is implemented explicitly by each
/// variant type (`impl Variant for Segmented {}`); it is not sealed, so a
/// kit or design system defines its own.
///
/// ```
/// use kivact::ui::Variant;
/// #[derive(Clone, Copy, Debug, PartialEq)]
/// struct Pill;
/// impl Variant for Pill {}
/// ```
pub trait Variant: PartialEq + std::fmt::Debug + 'static {}

/// The semantic [`Tone`](kivact_interaction::Tone) is a variant of every
/// standard component: a theme whose recipes know it draw the component in
/// that meaning (a progress bar's fill, say), and one that does not draws
/// the plain look.
impl Variant for kivact_interaction::Tone {}

/// One variant value a component carries for its recipes.
#[derive(Clone)]
struct Extension {
    value: Rc<dyn Any>,
    eq: fn(&dyn Any, &dyn Any) -> bool,
    debug: fn(&dyn Any, &mut std::fmt::Formatter<'_>) -> std::fmt::Result,
}

/// The variant values of a component, one per type.
#[derive(Clone, Default)]
struct Extensions(Rc<Vec<Extension>>);

impl Extensions {
    /// The values with `value` replacing an earlier value of its type.
    fn with<T: Variant>(mut self, value: T) -> Self {
        fn eq<T: PartialEq + 'static>(a: &dyn Any, b: &dyn Any) -> bool {
            a.downcast_ref::<T>() == b.downcast_ref::<T>()
        }
        fn debug<T: std::fmt::Debug + 'static>(
            value: &dyn Any,
            f: &mut std::fmt::Formatter<'_>,
        ) -> std::fmt::Result {
            match value.downcast_ref::<T>() {
                Some(value) => value.fmt(f),
                None => Ok(()),
            }
        }
        let entry = Extension {
            value: Rc::new(value),
            eq: eq::<T>,
            debug: debug::<T>,
        };
        let entries = Rc::make_mut(&mut self.0);
        entries.retain(|e| !e.value.is::<T>());
        entries.push(entry);
        self
    }

    fn get<T: 'static>(&self) -> Option<&T> {
        self.0.iter().find_map(|e| e.value.downcast_ref::<T>())
    }
}

impl PartialEq for Extensions {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
            || (self.0.len() == other.0.len()
                && self.0.iter().all(|a| {
                    other.0.iter().any(|b| {
                        (*a.value).type_id() == (*b.value).type_id() && (a.eq)(&*a.value, &*b.value)
                    })
                }))
    }
}

impl std::fmt::Debug for Extensions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut list = f.debug_list();
        for e in self.0.iter() {
            list.entry(&DebugWith(e));
        }
        list.finish()
    }
}

struct DebugWith<'a>(&'a Extension);

impl std::fmt::Debug for DebugWith<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (self.0.debug)(&*self.0.value, f)
    }
}

/// The variant values a component builder collects with
/// `.variant(..)`, shared with the recipe styles it has made: they are
/// read when the styles first run, once the builder is mounted.
#[derive(Clone, Default)]
pub(super) struct Variants(Rc<RefCell<Extensions>>);

impl Variants {
    pub(super) fn add<T: Variant>(&self, value: T) {
        let mut ext = self.0.borrow_mut();
        *ext = std::mem::take(&mut *ext).with(value);
    }

    /// An independent recipe look for one presentation of this component.
    pub(super) fn with<T: Variant>(&self, value: T) -> Self {
        Self(Rc::new(RefCell::new(self.0.borrow().clone().with(value))))
    }

    /// `state` with the values collected so far.
    pub(super) fn look<S>(&self, state: S) -> Look<S> {
        Look {
            state,
            ext: self.0.borrow().clone(),
        }
    }
}

/// What a recipe styles: the actual state `S` of a component's part and the
/// variants of the component.
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
/// use kivact::ui::{CheckState, Look, UiRecipes, Variant};
/// #[derive(Clone, Copy, Debug, PartialEq)]
/// struct Pill;
/// impl Variant for Pill {}
/// let recipes = UiRecipes::default().radio_group(|table| {
///     table.item(|prev| {
///         move |theme: &Theme, look: &Look<CheckState>| match look.variant::<Pill>() {
///             Some(Pill) => prev(theme, look).radius(999).background(rgb(0xe8eefc)),
///             None => prev(theme, look),
///         }
///     })
/// });
/// # let _ = recipes;
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Look<S> {
    state: S,
    ext: Extensions,
}

impl<S> Look<S> {
    /// `state` without variants: for a component that draws one of its own
    /// parts with a standard recipe (a kit's toggle drawn as the theme's
    /// button), adding its variants with [`with_variant`](Self::with_variant).
    pub fn new(state: S) -> Self {
        Look {
            state,
            ext: Extensions::default(),
        }
    }

    /// This look with `value`, replacing a value of its type.
    pub fn with_variant(mut self, value: impl Variant) -> Self {
        self.ext = self.ext.with(value);
        self
    }

    /// The variant of type `T` the component was given with
    /// `.variant(..)`, if any.
    pub fn variant<T: 'static>(&self) -> Option<&T> {
        self.ext.get::<T>()
    }
}

impl<S> std::ops::Deref for Look<S> {
    type Target = S;

    fn deref(&self) -> &S {
        &self.state
    }
}
