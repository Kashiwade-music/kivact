//! Role: Generic components built only from Kivact's public API.
//!
//! Generic components built **only** from Kivact's public API, the way a
//! third-party component library is written. Nothing here uses
//! crate-private hooks: every part is built from `node()`, `primitive()`,
//! `text`, `text_edit`, `image`, `Draw::region` and the reactive handles,
//! exactly what an application author can use. Applications define their
//! interactive roles, bindings and invocation handlers explicitly.
//!
//! | Part | Built from | Notes |
//! |---|---|---|
//! | [`SearchBox`] | a row `node()` + `text_edit` attachment (native IME) + monochrome `image` + [`parts::icon_button`] | Escape clears, optional status text |
//! | [`Timeline`] | a canvas `node()` + `Draw::region` + `on_pointer`/`on_key`/`on_a11y` | range selection by drag, handles are hit regions |
//! | [`LazyList`] | `node` + `NodeRef::rect` + `on_wheel`/`on_key` + keyed `for_each` + `draw_over` scrollbar | own virtualization, not the core `virtual_list` |
//! | [`PageDots`] | a primitive of its own (`kivact::primitive`, experimental): measure + `draw` + `Draw::region` + `on_pointer`/`on_key` | sized by its page count, one hit and a11y region per page |
//! | [`parts`] | `node()` + monochrome `image` (embedded SVG) | the icon button and its icons |
//! | [`skin`] | `Theme` + `Style` functions + context | two looks over the same components (style/skin invariance) |
//!
//! Every builder has the same shape: it holds its root `Node` and
//! implements `NodeBuilder`, so all standard modifiers (`style`,
//! `test_id`, `name`, ...) come from `NodeBuilder` and a modifier written
//! after the constructor wins. Constructors are `#[track_caller]` and
//! create the root, so every instance is located at its own call. State is
//! created in `into_view`, which runs in the setup of the place the part is
//! mounted, so two instances on one screen share nothing (no string keys,
//! no global ids).

pub mod lazy_list;
pub mod page_dots;
pub mod parts;
pub mod search_box;
pub mod skin;
pub mod timeline;

pub use lazy_list::{LazyList, lazy_list};
pub use page_dots::{PageDots, page_dots};
pub use search_box::{SearchBox, search_box};
pub use skin::{Skin, SkinCtx, provide_skin};
pub use timeline::{Span, Timeline, sparkline, timeline};
