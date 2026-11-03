//! Role: Standard UI components with theme support.
//!
//! Standard components: the few controls a first screen needs without an
//! official UI kit. The construction functions and the types their required
//! arguments need are in the prelude; the builder types, recipes and state
//! types are here.
//!
//! | Component | First argument | Behavior and semantics from |
//! |---|---|---|
//! | [`button`] | the visible text, also the name | `Role::Button`; click, Enter, Space and the accessibility `Activate` action activate it |
//! | [`checkbox`], [`switch`] | the visible label, also the name | [`CheckboxApi`](kivact_interaction::CheckboxApi) and [`Toggle`](kivact_interaction::Toggle) of kind `Switch` |
//! | [`select`] | the assistive name; not drawn | shared interaction parts |
//! | [`menu`] | the visible label of the trigger, also the name | shared interaction parts |
//! | [`dialog`] | the title, drawn unless hidden, and the dialog's name | shared interaction parts |
//! | [`tooltip`] | the tip's text, also its name | shared interaction parts |
//! | [`tabs`] | the assistive name of the tab list; not drawn | shared interaction parts |
//! | [`radio_group`] | the assistive name of the group; not drawn (each radio draws its item's label) | [`use_list`](kivact_interaction::use_list) with the radio policy |
//! | [`slider`], [`range_slider`] | the assistive name; not drawn | [`use_scalar`](kivact_interaction::use_scalar) and [`use_range_scalar`](kivact_interaction::use_range_scalar) |
//! | [`splitter`] | the assistive name of the separator; not drawn | [`use_splitter`](kivact_interaction::use_splitter) |
//! | [`split`] | none; it takes the panes and their fractions, and names its handles "Resize" | [`use_splitter`](kivact_interaction::use_splitter), one per pair of panes |
//! | [`toggle`] | the visible label, also the name | [`use_toggle_button`](kivact_interaction::use_toggle_button) |
//! | [`progress`] | none; it takes the value (`None` is indeterminate), and the caller names it | `Role::ProgressBar` with a range of 0 to 1 |
//! | [`disclosure`] | the title of the trigger, also its name | [`use_disclosure`](kivact_interaction::use_disclosure) |
//! | [`toast_region`] | none; it takes the [`Toaster`] | [`Toaster`]'s regions and items |
//! | [`command_palette`] | none; it takes the open state and the commands, and names its search field with the message `palette.search` | [`use_command_palette`](kivact_interaction::use_command_palette) |
//! | [`tag_input`] | the assistive name of the text input; not drawn | [`use_text_edit`](kivact_interaction::use_text_edit) |
//! | [`number_input`] | the assistive name of the field; not drawn | [`use_scalar`](kivact_interaction::use_scalar) as a spin button |
//! | [`list`] | the assistive name of the list; not drawn | [`use_list`](kivact_interaction::use_list) with the default policy and typeahead |
//! | [`data_table`] | the rows; the first argument is not text, the grid's name is the message `data_table.label` | [`use_table_model`](kivact_interaction::use_table_model) and [`use_table`](kivact_interaction::use_table) |
//! | [`text_input`], [`text_area`] | the assistive name; not drawn | [`text_edit`](kivact_interaction::use_text_edit) with native editing and IME |
//! | [`field`] | the visible label, which also names the control | a visible label, hint and error wired to any control: `labelled_by`, `described_by` and `invalid` |
//!
//! A first argument that is "not drawn" names the control for assistive
//! technology only; draw a visible label next to it yourself.
//!
//! Every component that writes a value reports each edit to `on_edit`
//! ([`EditPhase`]) and writes the value from
//! before a canceled drag back.
//!
//! # Looks
//!
//! Behavior comes from shared interaction parts; appearance comes from the
//! [`Theme`](kivact_core::style::Theme). It is changed in three steps, from small
//! to large:
//!
//! 1. **Tokens.** Every standard component takes its colors, spacing,
//!    radii, type sizes, focus ring, disabled opacity and control height
//!    from the theme's tables (`ColorToken::Accent`, `Spacing`, `Radii`,
//!    `Typography`, `focus_ring`, `disabled_opacity`, `control_height`), and
//!    its icons and marks from the theme's [`IconSet`](kivact_core::style::IconSet).
//!    A different [`Theme`](kivact_core::style::Theme) re-skins them all. A kit adds
//!    its own tokens with `Theme::with`.
//! 2. **Recipes.** A [`UiRecipes`] installed in the theme
//!    (`theme.with(recipes)`) replaces the style function, including
//!    local `Style::paint` drawing, of a component's parts. A UI kit's theme
//!    does this to give the standard components its own look; an application
//!    changes one part of it by wrapping the recipe it has (see
//!    [`UiRecipes`]). A recipe returns style only; hover, press, focus and
//!    disabled looks are the style's state styles.
//! 3. **Slots.** What an item shows ([`Tabs::item`], [`Menu::item`],
//!    [`Select::option`], [`List::item`]) is a view the caller builds, and
//!    the regions of a [`data_table`] ([`DataTable::toolbar`],
//!    [`DataTable::footer`], [`DataTable::row_details`],
//!    [`DataTable::cell_controls`]) are filled the same way; without one
//!    the component shows the item's icon, label, description and marks.
//!
//! Without a [`UiRecipes`] in the theme, [`UiRecipes::default`] is used.
//! [`Recipe<S>`] is a function from theme and part state to [`Style`](kivact_core::style::Style).
//! Native text fields accept style properties and retain their existing editing identity;
//! their leaf renderer does not support `Style::paint`.
//! What you set on a component yourself (`.style(..)`, `.name(..)`,
//! `.draw_over(..)`, ...) always comes after the recipe and wins.
//!
//! Components are built like components outside the core: from `node()`,
//! `interaction::use_text_edit` and the unstyled [`interaction`](kivact_interaction)
//! behaviors, through the public API only. [`Button`], [`Checkbox`], [`Switch`],
//! [`Select`], [`Tabs`] and [`TextInput`] (the native field itself) take
//! [`NodeBuilder`](kivact_core::control::NodeBuilder) modifiers on their roots.
//! Overlays attach the same way everywhere: [`Menu::attach`],
//! [`Menu::attach_context`] and [`Tooltip::attach`] take the caller's trigger,
//! [`Dialog::content`]/[`Dialog::footer`], [`Tabs::content`] and
//! [`Tooltip::content`] take what is shown, and the builder itself is the view.
//! The method name `show` is kept for branching (`view::show`); a kit's
//! methods of the same roles take the same names.
//!
//! # Kits
//!
//! A UI kit does not reimplement or re-export these components: it supplies
//! a theme and its domain components. A light change of look is tokens and
//! recipes; a kit's own component is built from the interaction parts and
//! the primitives. Every theme of the official kits carries recipes for
//! every standard component. A kit's theme is one function of the shape
//! `theme(mode: ThemeMode, ..)`; its tokens are one type implementing
//! [`ThemeExtension`](kivact_core::style::ThemeExtension) (its `fallback` follows
//! the theme), installed with `Theme::with` and read only with
//! `theme.get::<Tokens>()`, its colors are also named by a field-less `Color`
//! enum implementing [`ThemeColor`](kivact_core::style::ThemeColor) (a style
//! takes `basic::Color::Border` and reads the theme it is painted in), and its
//! icons are added with `Theme::with_icon`. The
//! token type and the theme constructor that assembles the recipes are kept
//! apart, so a recipe reading tokens never depends back on the constructor.
//! The types and API reference are here; installing a kit is in the
//! official UI guide.
//!
//! Callbacks are plain closures (`impl Fn(T) -> R` or `impl Fn() -> R`, where
//! `R` is `()` or `Result<(), UiError>`), and bool options such as
//! [`Button::hide_label`] take a constant or a reactive value.
//!
//! ```no_run
//! use kivact::prelude::*;
//!
//! fn sign_up() -> impl IntoView {
//!     let name = signal(String::new());
//!     let agreed = signal(false);
//!     column((
//!         text_input("Name", name).placeholder("Your name"),
//!         checkbox("I agree", agreed),
//!         button("Create account")
//!             .variant(ButtonVariant::Primary)
//!             .disabled(move || !agreed.get())
//!             .on_press(move || println!("welcome, {}", name.get())),
//!     ))
//!     .style(style().gap(12).padding(16))
//! }
//! ```

mod button;
mod children;
mod command_palette;
mod data_column;
mod data_table;
mod disclosure;
mod field;
mod helpers;
mod inline_edit;
mod layer;
mod list;
mod look;
mod look_collections;
mod look_controls;
mod look_extra;
mod look_input;
mod look_palette;
mod number_input;
mod progress;
mod recipes;
mod split;
mod tag_input;
mod text_draft;
mod text_input;
mod toggle;
mod toggle_button;
mod toggle_parts;
mod tree;
mod values;
mod variants;

pub use button::{Button, ButtonBusyMotion, DisabledButtonFocus, button};
pub use children::IntoChildren;
pub use command_palette::{CommandPalette, command_palette};
pub use data_column::DataColumn;
pub use data_table::{DataTable, DataTableContext, data_table};
pub use disclosure::{Disclosure, disclosure};
pub use field::{Field, field};
pub use inline_edit::{InlineEdit, use_inline_edit};
/// The values and data the standard components and the official kits take,
/// so an app needs no `kivact::interaction` import (that module is the
/// component author's).
pub use kivact_interaction::{
    Command, DialogKind, Dismiss, EditPhase, ListActivation, ListItem, MenuEntry, ScalarRange,
    ScalarRangeError, Side, Toast, ToastOptions, Toaster, Tone, TreeNode, toaster,
};
pub use list::{List, list};
pub use number_input::{NumberInput, number_input};
pub use progress::{Progress, ProgressMotion, progress};
pub use recipes::{
    CommandPaletteRecipes, DataTableRecipes, DialogRecipes, DisclosureRecipes, FieldRecipes,
    ListRecipes, MenuRecipes, NumberInputRecipes, ProgressRecipes, RadioGroupRecipes, Recipe,
    SelectRecipes, SliderRecipes, SplitRecipes, SplitterRecipes, TabsRecipes, TagInputRecipes,
    ToastRecipes, TooltipRecipes, TreeRecipes, UiRecipes,
};
pub use split::{Split, split};
pub use tag_input::{TagInput, tag_input};
pub use text_input::{TextInput, text_area, text_input};
pub use toggle::{Checkbox, Switch, checkbox, switch};
pub use toggle_button::{Toggle, ToggleLabelReservation, toggle};
pub use tree::{TreeView, tree, tree_rows, tree_rows_with_reveal};
pub use values::{
    AxisState, ButtonState, ButtonVariant, CheckState, ControlSize, DataTableCellState,
    DataTableCheckState, DataTablePageState, DataTableRowState, DialogState, FieldState,
    ProgressState, SelectTriggerState, TextInputState, ToastGroupState, ToggleButtonState,
    TooltipArrowState, provide_control_size,
};
pub use variants::{Look, Variant};

mod dialog;
mod menu;
mod radio_group;
mod select;
mod slider;
mod splitter;
mod tabs;
mod toast;
mod toast_rows;
mod tooltip;
pub use dialog::{Dialog, DialogCenter, dialog};
pub use menu::{Menu, menu};
pub use radio_group::{RadioGroup, radio_group};
pub use select::{Select, select};
pub use slider::{Slider, range_slider, slider};
pub use splitter::{Splitter, splitter};
pub use tabs::{Tabs, tabs};
pub use toast::{ToastRegion, ToastRowMotion, toast_region};
pub use tooltip::{Tooltip, tooltip};
