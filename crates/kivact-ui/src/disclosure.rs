//! Role: Build the themed disclosure, a trigger over content it shows and hides.
//!
//! The trigger is the shared disclosure trigger part (a button publishing
//! `expanded` and `controls`), the content its content part. All four
//! recipes (the root, the trigger, the indicator and the content) get the
//! interaction part's [`DisclosureState`](kivact_interaction::DisclosureState).
//! An accordion is a column of disclosures whose `open` bindings read and
//! write one `Binding<Vec<K>>`.
pub(crate) mod group;
pub(crate) mod look;
pub(crate) mod tokens;
mod view;

pub use crate::disclosure::tokens::DisclosureMotion;
pub use crate::disclosure::tokens::DisclosureTokens;
pub use view::{Disclosure, disclosure};
