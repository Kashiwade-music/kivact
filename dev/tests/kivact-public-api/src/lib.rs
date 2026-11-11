//! Role: Public API consumer tests. The tests live in tests/ (e2e,
//! offscreen); code that must not compile is checked by the doc tests below.
//!
//! # What `#[component]` rejects
//!
//! A plain function with doc comments and `allow`, `warn`, `deny` and
//! `deprecated` compiles:
//!
//! ```
//! use kivact::prelude::*;
//! /// A card.
//! #[component]
//! #[allow(dead_code, reason = "only compiled")]
//! #[deprecated(note = "use another card")]
//! fn card(title: Value<String>) -> impl IntoView {
//!     text(title)
//! }
//! ```
//!
//! An `async` function does not:
//!
//! ```compile_fail
//! use kivact::prelude::*;
//! #[component]
//! async fn card(title: Value<String>) -> impl IntoView {
//!     text(title)
//! }
//! ```
//!
//! A `const` function does not:
//!
//! ```compile_fail
//! use kivact::prelude::*;
//! #[component]
//! const fn card(title: Value<String>) -> impl IntoView {
//!     text(title)
//! }
//! ```
//!
//! An `extern` function does not:
//!
//! ```compile_fail
//! use kivact::prelude::*;
//! #[component]
//! extern "C" fn card(title: Value<String>) -> impl IntoView {
//!     text(title)
//! }
//! ```
//!
//! `#[expect(..)]` does not (write `#[allow(..)]`):
//!
//! ```compile_fail
//! use kivact::prelude::*;
//! #[component]
//! #[expect(dead_code, reason = "only compiled")]
//! fn card(title: Value<String>) -> impl IntoView {
//!     text(title)
//! }
//! ```
//!
//! Any other attribute does not:
//!
//! ```compile_fail
//! use kivact::prelude::*;
//! #[component]
//! #[inline]
//! fn card(title: Value<String>) -> impl IntoView {
//!     text(title)
//! }
//! ```
//!
//! An `impl Trait` prop does not (write `Value<T>`):
//!
//! ```compile_fail
//! use kivact::prelude::*;
//! #[component]
//! fn card(title: impl Into<String>) -> impl IntoView {
//!     text(title.into())
//! }
//! ```
