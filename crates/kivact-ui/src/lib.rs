//! Role: Standard UI components with theme support.
//!
//! Standard components: the few controls a first screen needs without an
//! official UI kit. The everyday construction functions and the types their
//! required arguments need are in the prelude; every construction function,
//! the builder types, recipes and state types are here.
//!
//! | Component | First argument | Behavior and semantics from |
//! |---|---|---|
//! | [`button`] | the visible text, also the name | `Role::Button`; click, Enter, Space and the accessibility `Activate` action activate it |
//! | [`checkbox`], [`switch`] | the visible label, also the name | [`CheckboxApi`](kivact_interaction::CheckboxApi) and [`SwitchApi`](kivact_interaction::SwitchApi) |
//! | [`select`] | the selected value's signal; named with `.label(..)` | shared interaction parts |
//! | [`menu`] | the visible label of the trigger, also the name | shared interaction parts |
//! | [`dialog`] | the title, drawn unless hidden, and the dialog's name | shared interaction parts |
//! | [`tooltip`] | the tip's text; the trigger is the second argument | shared interaction parts |
//! | [`icon_button`] | the icon, then the label, which names the button and is its tooltip | `Role::Button` with a tooltip |
//! | [`stepper`] | the steps; the current step is a `Signal<usize>` that presses write | `Role::List`, `on_edit` |
//! | [`tabs`] | the assistive name of the tab list; not drawn | shared interaction parts |
//! | [`radio_group`] | the assistive name of the group; not drawn (each radio draws its item's label) | [`use_list`](kivact_interaction::use_list) with the radio policy |
//! | [`slider`], [`range_slider`] | the assistive name; not drawn | [`use_scalar`](kivact_interaction::use_scalar) and [`use_range_scalar`](kivact_interaction::use_range_scalar) |
//! | [`splitter`] | the assistive name of the separator; not drawn | [`use_splitter`](kivact_interaction::use_splitter) |
//! | [`split`] | none; it takes the panes and their fractions, and names its handles "Resize" | [`use_splitter`](kivact_interaction::use_splitter), one per pair of panes |
//! | [`toggle_button`] | the visible label, also the name | [`use_toggle_button`](kivact_interaction::use_toggle_button) |
//! | [`progress`] | none; it takes the value (`None` is indeterminate), and the caller names it | `Role::ProgressBar` with a range of 0 to 1 |
//! | [`sparkline`] | none; it takes the values, and is decorative until [`Sparkline::label`] names it | no role without a label; `Role::Image` with one, and a description of its values |
//! | [`disclosure`] | the title of the trigger, also its name | [`use_disclosure`](kivact_interaction::use_disclosure) |
//! | [`toast_region`] | none; it takes the [`ToasterApi`] | [`ToasterApi`]'s regions and items |
//! | [`command_palette`] | none; it takes the open state and the commands, and names its search field with the message `palette.search` | [`use_command_palette`](kivact_interaction::use_command_palette) |
//! | [`tag_input`] | the assistive name of the text input; not drawn | [`use_text_edit`](kivact_interaction::use_text_edit) |
//! | [`number_input`] | the assistive name of the field; not drawn | [`use_scalar`](kivact_interaction::use_scalar) as a spin button |
//! | [`list`] | the assistive name of the list; not drawn | [`use_list`](kivact_interaction::use_list) with the default policy and typeahead |
//! | [`data_table`] | the rows; the first argument is not text, the grid's name is the message `data_table.label` | [`use_data_table_model`](kivact_interaction::use_data_table_model) and [`use_data_table`](kivact_interaction::use_data_table) |
//! | [`text_input`], [`text_area`] | the value's signal; named with `.label(..)` | [`text_edit`](kivact_interaction::use_text_edit) with native editing and IME |
//! | [`field`] | the visible label, which also names the control | a visible label, hint and error wired to any control: `labelled_by`, `described_by` and `invalid` |
//!
//! A first argument that is "not drawn" names the control for assistive
//! technology only; draw a visible label next to it yourself.
//!
//! A component with an `on_edit` method (the toggles, choosers, sliders,
//! splitters, lists, tabs and text and number fields) reports each edit to
//! it ([`EditPhase`]) and writes the value from before a canceled drag back.
//! The calendar, the date and time inputs, pagination, the carousel, the
//! disclosures and the data table write their values without reporting
//! edits.
//!
//! # Looks
//!
//! Behavior comes from shared interaction parts; appearance comes from the
//! [`Theme`](kivact_core::style::Theme). It is changed in three steps, from small
//! to large:
//!
//! 1. **Theme variables.** Every standard component takes its colors, spacing,
//!    radii, type sizes, focus ring, disabled opacity and control height
//!    from the theme's variables (`kivact::style::color::ACCENT`, `space::MD`,
//!    `radius::MD`, `font::SIZE`, `focus_ring`, `control::DISABLED_OPACITY`,
//!    `control::HEIGHT`), and
//!    its icons and marks from the theme's [`IconSet`](kivact_core::style::IconSet).
//!    A different [`Theme`](kivact_core::style::Theme) re-skins them all. A kit adds
//!    its own variables with `Theme::set` and its recipes with `Theme::with`.
//! 2. **Recipes.** A `<Name>Theme` from [`theme`] installed in the theme
//!    (`theme.with(ButtonTheme::default().root(..))`) replaces the style function, including
//!    local `Style::paint` drawing, of a component's parts. A UI kit's theme
//!    does this to give the standard components its own look; an application
//!    changes one part of it by wrapping the recipe it has (see
//!    [`theme`]). A recipe returns style only; hover, press, focus and
//!    disabled looks are the style's state styles.
//! 3. **Slots.** What an item shows ([`Tabs::item`], [`Menu::item`],
//!    [`Select::item`], [`List::item`]) is a view the caller builds, and
//!    the regions of a [`data_table`] ([`DataTable::toolbar`],
//!    [`DataTable::footer`], [`DataTable::row_details`],
//!    [`DataTable::cell_controls`]) are filled the same way; without one
//!    the component shows the item's icon, label, description and marks.
//!
//! Without a recipe in the theme, the defaults of [`theme`] are used.
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
//! [`Element`](kivact_core::view::Element) modifiers on their roots.
//! Overlays attach the same way everywhere: [`Menu::attach`] and
//! [`Menu::attach_context`] take the caller's trigger, [`tooltip`] takes its
//! trigger as the second argument, [`Dialog::content`]/[`Dialog::footer`],
//! [`Tabs::content`] and [`Tooltip::content`] take what is shown, and the
//! builder itself is the view.
//! The method name `show` is kept for branching (`view::show`); a kit's
//! methods of the same roles take the same names.
//!
//! # Kits
//!
//! A UI kit does not reimplement or re-export these components: it supplies
//! a theme and its domain components. A kit's theme is one function of the
//! shape `theme(mode) -> Theme`. It sets the variables of the standard
//! components with `Theme::set`, and their recipes with `<Name>Theme`
//! (`Theme::with`); a kit's own component is built from the interaction
//! parts and the primitives. Every theme of the official kits carries recipes
//! for every standard component.
//!
//! The types and API reference are here; installing a kit is in the
//! official UI guide.
//!
//! Handlers are closures. One without a payload takes
//! `impl IntoHandler<(), M>` (a closure `move || ..` or a shared `Callback`);
//! one with a payload takes `impl Fn(T) -> R`, where `R` is `()` or
//! `Result<(), UiError>`. Bool options such as [`Button::hide_label`] take a constant or a reactive value.
//!
//! ```no_run
//! use kivact::prelude::*;
//!
//! fn sign_up() -> impl IntoView {
//!     let name = signal(String::new());
//!     let agreed = signal(false);
//!     column((
//!         text_input(name).label("Name").placeholder("Your name"),
//!         checkbox("I agree", agreed),
//!         button("Create account")
//!             .variant(ButtonVariant::Primary)
//!             .disabled(move || !agreed.get())
//!             .on_press(move || println!("welcome, {}", name.get())),
//!     ))
//!     .style(style().gap(12).padding(16))
//! }
//! ```

mod date_parts;

mod button;
mod command_palette;
mod control_size;
mod data_table;
mod disclosure;
mod field;
mod helpers;
mod layer;
mod list;
mod look_common;
mod number_input;
mod progress;
mod recipe;
pub use control_size::{ControlSize, provide_control_size};
mod split;
mod tag_input;
mod text_input;
pub mod theme;
mod toggle;
mod toggle_button;
mod tree;
mod variants;

pub use crate::button::ButtonVariant;
pub use crate::radio_group::RadioGroupVariant;
pub use button::{Button, button, icon_button};
pub use command_palette::{CommandPalette, command_palette};
pub use data_table::column::DataColumn;
pub use data_table::{
    DataTable, DataTableContext, DataTableDensity, DataTableTreeIndent, FilterField, FilterUi,
    Paging, data_table,
};
pub use disclosure::{Disclosure, DisclosureMotion, disclosure};
pub use field::{Field, field};
pub use list::{List, list};
pub use number_input::{NumberInput, number_input};
pub use progress::{Progress, ProgressMotion, progress};
pub use split::{Split, split};
pub use tag_input::{TagInput, tag_input};
pub use text_input::{TextInput, text_area, text_input};
pub use toggle::{Checkbox, Switch, checkbox, switch};
pub use toggle_button::look::ToggleLabelReservation;
pub use toggle_button::{ToggleButton, toggle_button};
pub use tree::{Tree, TreeRows, tree, tree_rows};

mod dialog;
mod menu;
mod radio_group;
mod select;
mod slider;
mod splitter;
mod tabs;
mod toast;
mod tooltip;
pub use dialog::{Dialog, DialogKind, dialog};
pub use menu::{Menu, menu};
pub use radio_group::{RadioGroup, radio_group};
pub use select::{Select, SelectSheetOption, select};
pub use slider::{Slider, range_slider, slider};
pub use splitter::{Splitter, splitter};
pub use tabs::{Tabs, tabs};
pub use toast::{ToastRegion, toast_region};
pub use tooltip::{Tooltip, tooltip};

pub use crate::sparkline::look::SparklineFill;

mod async_view;
pub use async_view::{AsyncView, async_view};

mod calendar;
pub use crate::calendar::CalendarCaption;
pub use crate::calendar::CalendarEvent;
pub use calendar::{Calendar, calendar};

mod date_input;
pub use date_input::{DateInput, DatePresentation, date_input};

mod time_input;
pub use time_input::{TimeInput, time_input};

mod date_range_input;
pub use date_range_input::{DateRangeInput, DateRangePreset, date_range_input};

mod date_time_input;
pub use date_time_input::{DateTimeInput, date_time_input};

mod carousel;
pub use carousel::{Carousel, carousel};

mod dropzone;
pub use disclosure::group::{DisclosureGroup, disclosure_group};
pub use dropzone::{Dropzone, dropzone};

mod hover_card;
pub use hover_card::{FocusTrigger, HoverCard, hover_card};

mod combobox;
pub use combobox::{Combobox, ComboboxMode, combobox};
pub use select::presentation::SelectPresentation;
mod button_group;
mod toggle_group;
mod toolbar;
pub use button_group::{
    ButtonGroup, ButtonGroupCell, button_group, button_group_separator, button_group_text,
};
pub use toggle_group::{ToggleGroup, toggle_group};
pub use toolbar::{Toolbar, toolbar};

mod alert;
mod card;
mod empty_state;
mod input_group;
pub use alert::{Alert, alert};
pub use card::{Card, card};
pub use empty_state::{EmptyState, empty_state};
pub use input_group::context::use_input_group_state;
pub use input_group::{InputGroup, input_addon, input_group};
mod badge;
mod skeleton;
mod spinner;
pub use crate::avatar::AvatarShape;
pub use crate::skeleton::SkeletonShape;
pub use badge::{Badge, badge};
pub use skeleton::{Skeleton, skeleton};
pub use spinner::{Spinner, spinner};

mod avatar;
mod avatar_group;
pub use avatar::{Avatar, avatar};
pub use avatar_group::{AvatarGroup, avatar_group};

mod kbd;
pub use kbd::{Kbd, KbdCombo, kbd, kbd_combo};
mod item_list;
mod table;
pub use crate::item_list::ItemListMarker;
pub use crate::item_list::ItemPosition;
pub use item_list::row::{ItemRow, item_row};
pub use item_list::{ItemList, item_list};
pub use table::{
    Table, TableCell, TableRow, table, table_body, table_cell, table_footer, table_head,
    table_header, table_row,
};
mod breadcrumb;
mod navigation_menu;
pub use crate::pagination::PageItem;
pub use crate::pagination::PaginationMode;
pub use crate::pagination::PaginationWindow;
pub use navigation_menu::link::{NavigationMenuLink, navigation_menu_link};
mod pagination;
mod stepper;
pub use breadcrumb::{Breadcrumb, breadcrumb};
pub use navigation_menu::{NavigationMenu, navigation_menu};
pub use pagination::{Pagination, pagination};
pub use stepper::{IndicatorContent, StepIndicator, Stepper, StepperItem, stepper};

mod description_list;
mod heading;
mod stat;
pub use description_list::{Description, DescriptionList, description, description_list};
pub use heading::{Heading, heading};
pub use stat::{Stat, stat};

mod sparkline;
pub use sparkline::{Sparkline, sparkline};

mod chat_composer;
mod chat_message;
mod chat_thread;
mod code_block;
mod typing_indicator;
pub use crate::chat_composer::ComposerStatus;
pub use crate::chat_composer::ComposerStatusPosition;
pub use crate::chat_message::ChatAlign;
pub use crate::chat_message::ChatBubbleGroup;
pub use crate::chat_message::ChatMessageStatus;
pub use crate::chat_message::ChatMessageVariant;
pub use crate::chat_message::ChatReactionsSide;
pub use crate::chat_message::ChatSender;
pub use crate::code_block::CodeCollapse;
pub use chat_composer::{ChatComposer, chat_composer};
pub use chat_message::{ChatMessage, chat_message};
pub use chat_thread::{ChatThread, chat_thread};
pub use code_block::{CodeBlock, code_block};
pub use typing_indicator::{TypingIndicator, typing_indicator};

pub use crate::card::CardAction;
pub use crate::stat::Trend;
pub use crate::toolbar::ToolbarEdge;

pub use crate::alert::AlertVariant;
pub use crate::carousel::CarouselMode;
pub use crate::progress::ProgressVariant;
pub use crate::stepper::CollapsedSteps;
pub use crate::stepper::StepIndicatorKind;
pub use crate::stepper::StepStatus;

pub use crate::data_table::types::RowStatus;
