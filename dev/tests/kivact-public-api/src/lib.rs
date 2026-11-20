//! Role: Public API consumer tests and the E2E helpers their test binaries share.
//! The tests live in tests/ (e2e_core, e2e_ui, e2e_native and offscreen); code
//! that must not compile is checked by the doc tests below.
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
//!
//! An `Option<..>` prop with a default does not (the `Option` is the optional form):
//!
//! ```compile_fail
//! use kivact::prelude::*;
//! #[component]
//! fn card(#[prop(default = 1)] level: Option<u8>) -> impl Element {
//!     node()
//! }
//! ```
//!
//! `#[prop]` takes only a default; `optional` does not:
//!
//! ```compile_fail
//! use kivact::prelude::*;
//! #[component]
//! fn card(#[prop(optional)] level: Option<u8>) -> impl Element {
//!     node()
//! }
//! ```
//!
//! A prop type the file does not import does not compile:
//!
//! ```compile_fail
//! use kivact::component;
//! #[component]
//! fn card(title: Value<String>) -> impl kivact::view::IntoView {
//!     kivact::view::text(title)
//! }
//! ```
//!
//! A lifetime parameter does not (write type parameters only):
//!
//! ```compile_fail
//! use kivact::prelude::*;
//! #[component]
//! fn label<'a>(text: Value<String>) -> impl Element {
//!     node()
//! }
//! ```
//!
//! A const parameter does not:
//!
//! ```compile_fail
//! use kivact::prelude::*;
//! #[component]
//! fn dots<const N: usize>() -> impl Element {
//!     node()
//! }
//! ```
//!
//! A type parameter with the reserved `__Kv` prefix does not:
//!
//! ```compile_fail
//! use kivact::prelude::*;
//! #[component]
//! fn shown<__KvT: Clone>(value: Value<__KvT>) -> impl Element {
//!     node()
//! }
//! ```

pub mod paint;
pub mod support;
