//! Role: Find mutated types and functions that hold two or more state fields or locals.
//!
//! A struct, or the variants of an enum, with two or more fields of kind
//! `bool`, `Option`, named counter, `Cell`, `RefCell`, `Mutex`, `RwLock`,
//! `Atomic*`, `Signal` or `Var` needs one `// xtask-allow(state): <how the
//! combinations are closed>` line before the type when the type is mutated:
//! an impl of its package has a `&mut self` method, it has an
//! interior-mutable field, or one of its methods writes a `Signal`/`Var`
//! field. Fields written only by constructors, `Default` and by-value
//! builders are a passive record and need nothing. No name or derive
//! exempts a type. A function with two or more state locals needs the same
//! line before the function. A marker whose text starts with `legacy:` was
//! converted mechanically from the replaced rule; the run reports how many
//! are left (design 6.1, chapter 12 ST-01).

mod check;
mod fields;
mod index;
mod locals;
#[cfg(test)]
mod tests;

pub(crate) use check::{check, legacy_count};
pub(crate) use index::Index;
