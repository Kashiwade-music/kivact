#![deny(clippy::too_many_lines, clippy::disallowed_types)]
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
//! | [`checkbox`], [`switch`] | the visible label, also the name | [`CheckboxApi`](kivact_interaction::CheckboxApi) and [`SwitchApi`](kivact_interaction::SwitchApi) |
//! | [`select`] | the assistive name; not drawn | shared interaction parts |
//! | [`menu`] | the visible label of the trigger, also the name | shared interaction parts |
//! | [`dialog`] | the title, drawn unless hidden, and the dialog's name | shared interaction parts |
//! | [`tooltip`] | the tip's text, also its name | shared interaction parts |
//! | [`tabs`] | the assistive name of the tab list; not drawn | shared interaction parts |
//! | [`radio_group`] | the assistive name of the group; not drawn (each radio draws its item's label) | [`use_list`](kivact_interaction::use_list) with the radio policy |
//! | [`slider`], [`range_slider`] | the assistive name; not drawn | [`use_scalar`](kivact_interaction::use_scalar) and [`use_range_scalar`](kivact_interaction::use_range_scalar) |
//! | [`splitter`] | the assistive name of the separator; not drawn | [`use_splitter`](kivact_interaction::use_splitter) |
//! | [`split`] | none; it takes the panes and their fractions, and names its handles "Resize" | [`use_splitter`](kivact_interaction::use_splitter), one per pair of panes |
//! | [`toggle_button`] | the visible label, also the name | [`use_toggle_button`](kivact_interaction::use_toggle_button) |
//! | [`progress`] | none; it takes the value (`None` is indeterminate), and the caller names it | `Role::ProgressBar` with a range of 0 to 1 |
//! | [`disclosure`] | the title of the trigger, also its name | [`use_disclosure`](kivact_interaction::use_disclosure) |
//! | [`toast_region`] | none; it takes the [`Toaster`] | [`Toaster`]'s regions and items |
//! | [`command_palette`] | none; it takes the open state and the commands, and names its search field with the message `palette.search` | [`use_command_palette`](kivact_interaction::use_command_palette) |
//! | [`tag_input`] | the assistive name of the text input; not drawn | [`use_text_edit`](kivact_interaction::use_text_edit) |
//! | [`number_input`] | the assistive name of the field; not drawn | [`use_scalar`](kivact_interaction::use_scalar) as a spin button |
//! | [`list`] | the assistive name of the list; not drawn | [`use_list`](kivact_interaction::use_list) with the default policy and typeahead |
//! | [`data_table`] | the rows; the first argument is not text, the grid's name is the message `data_table.label` | [`use_data_table_model`](kivact_interaction::use_data_table_model) and [`use_data_table`](kivact_interaction::use_data_table) |
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
//!    [`Select::item`], [`List::item`]) is a view the caller builds, and
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
//! [`NodeBuilder`](kivact_core::view::NodeBuilder) modifiers on their roots.
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
//! `theme.ext::<Tokens>()`, its colors are also named by a field-less `Color`
//! enum implementing [`ThemeColor`](kivact_core::style::ThemeColor) (a style
//! takes `basic::ColorName::Border` and reads the theme it is painted in), and its
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
//! use kivact::ui::ButtonVariant;
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
mod command_palette;
mod data_column;
mod data_table;
mod disclosure;
mod field;
mod helpers;
mod layer;
mod list;
mod look;
mod look_collections;
mod look_controls;
mod look_extra;
mod look_input;
mod look_palette;
mod look_radio;
mod number_input;
mod progress;
mod recipes;
mod split;
mod switch_drag;
mod tag_input;
mod text_input;
mod toggle;
mod toggle_button;
mod toggle_parts;
mod tree;
mod values;
mod variants;

pub use button::{Button, ButtonBusyMotion, ButtonTokens, button};
pub use command_palette::{CommandPalette, command_palette};
mod column_values;
pub use column_values::DataColumn;
pub use data_table::{
    DataTable, DataTableContext, DataTableDensity, DataTableRowStatusPaint, DataTableStickyPaint,
    DataTableTokens, DataTableTreeIndent, FilterField, FilterUi, data_table,
};
pub use disclosure::{Disclosure, DisclosureMotion, DisclosureTokens, disclosure};
pub use field::{Field, field};
pub use list::{List, list};
pub use number_input::{NumberInput, number_input};
pub use progress::{Progress, ProgressMotion, ProgressTokens, progress};
pub use recipes::{
    CalendarRecipes, ComboboxRecipes, CommandPaletteRecipes, DataTableRecipes, DateInputRecipes,
    DateRangeInputRecipes, DateTimeInputRecipes, DialogRecipes, DisclosureRecipes, FieldRecipes,
    ListRecipes, MenuRecipes, NumberInputRecipes, ProgressRecipes, RadioGroupRecipes, Recipe,
    SelectRecipes, SliderRecipes, SplitRecipes, SplitterRecipes, TabsRecipes, TagInputRecipes,
    TimeInputRecipes, ToastRecipes, TooltipRecipes, TreeRecipes, UiRecipes,
};
pub use split::{Split, split};
pub use tag_input::{TagInput, tag_input};
pub use text_input::{TextInput, text_area, text_input};
pub use toggle::{Checkbox, Switch, checkbox, switch};
pub use toggle_button::{ToggleButton, ToggleButtonTokens, ToggleLabelReservation, toggle_button};
pub use tree::{Tree, TreeRows, tree, tree_rows};
pub use values::{
    AxisState, ButtonState, ButtonVariant, CheckableState, ControlSize, DataTableCellState,
    DataTableCheckState, DataTablePageState, DataTableRowState, DialogState, FieldState,
    ProgressState, RadioGroupVariant, SelectTriggerState, SwitchDragState, TextInputState,
    ToastGroupState, ToggleButtonState, TooltipArrowState, provide_control_size,
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
mod toast_swipe;
mod tooltip;
pub use dialog::{Dialog, DialogCenter, DialogTokens, dialog};
pub use menu::{Menu, menu};
pub use radio_group::{RadioGroup, radio_group};
pub use select::{Select, SelectSheetOption, select};
pub use slider::{Slider, range_slider, slider};
pub use splitter::{Splitter, splitter};
pub use tabs::{Tabs, tabs};
pub use toast::{ToastRegion, ToastRowMotion, ToastTokens, toast_region};
pub use tooltip::{Tooltip, tooltip};

mod tokens;
pub use tokens::UiTokens;

mod button_tokens;
mod dialog_tokens;
mod disclosure_tokens;
mod progress_tokens;
mod toast_tokens;
mod toggle_button_tokens;

mod data_table_tokens;

mod async_view;
pub use async_view::{AsyncView, async_view};

mod calendar;
pub use calendar::{Calendar, calendar};
mod calendar_values;
pub use calendar_values::{CalendarCaption, CalendarDayVisual, CalendarEvent};

mod calendar_tokens;
mod look_calendar;
pub use calendar_tokens::{CalendarEventVisual, CalendarTokens};

mod date_input;
mod date_trigger;
pub use date_input::{DateInput, DatePresentation, date_input};

mod date_wheel;

mod time_input;
pub use time_input::{TimeInput, time_input};

mod date_range_draft;
mod date_range_input;
pub use date_range_input::{DateRangeInput, DateRangePreset, date_range_input};

mod date_time_input;
pub use date_time_input::{DateTimeInput, date_time_input};

mod date_input_tokens;
pub use date_input_tokens::DateInputTokens;
mod carousel;
pub use carousel::{Carousel, Snap, carousel};

mod dropzone;
pub use dropzone::{Dropzone, dropzone};
mod carousel_tokens;
mod dropzone_tokens;
mod look_files;
pub use carousel_tokens::CarouselTokens;
pub use dropzone_tokens::DropzoneTokens;
pub use recipes::{CarouselRecipes, DropzoneRecipes};
mod carousel_geometry;
mod disclosure_values;
pub use disclosure_values::{DisclosureGroupPosition, DisclosureRecipeState};
mod disclosure_group;
pub use disclosure_group::{DisclosureGroup, disclosure_group};

mod hover_card;
pub use hover_card::{FocusTrigger, HoverCard, hover_card};

pub use recipes::HoverCardRecipes;

mod combobox;
mod drawer_swipe;
mod selection_field;
pub use selection_field::{SelectionField, selection_field};
mod selector_presentation;
pub use combobox::{Combobox, ComboboxTokens, combobox};
pub use kivact_interaction::ComboboxMode;
pub use selector_presentation::SelectPresentation;
mod button_group;
mod group_look;
mod group_types;
mod toggle_group;
mod toolbar;
pub use button_group::{
    ButtonGroup, ButtonGroupCell, ButtonGroupMember, button_group, button_group_separator,
    button_group_text,
};
pub use group_types::{
    ButtonGroupMemberState, ButtonGroupState, ButtonGroupTokens, ToggleGroupItemState,
    ToggleGroupState, ToggleGroupTokens,
};
pub use recipes::{ButtonGroupRecipes, ToggleGroupRecipes, ToolbarRecipes};
pub use toggle_group::{ToggleGroup, toggle_group};
pub use toolbar::{Toolbar, toolbar};

mod alert;
mod card;
mod empty_state;
mod empty_state_changes;
mod input_group;
mod input_group_context;
pub use input_group_context::use_input_group_state;
mod look_feedback;
pub use alert::{Alert, alert};
pub use card::{Card, card};
pub use empty_state::{EmptyState, empty_state};
pub use input_group::{InputGroup, input_addon, input_group, input_group_with};
pub use recipes::{AlertRecipes, CardRecipes, EmptyStateRecipes, InputGroupRecipes};
mod badge;
mod indicator_values;
mod look_indicators;
mod skeleton;
mod spinner;
pub use badge::{Badge, badge};
pub use indicator_values::{
    AvatarShape, AvatarState, BadgeState, IndicatorTokens, SkeletonShape, SkeletonState,
    SpinnerState,
};
pub use skeleton::{Skeleton, skeleton};
pub use spinner::{Spinner, spinner};

mod avatar;
mod avatar_group;
mod avatar_initials;
pub use avatar::{Avatar, avatar};
pub use avatar_group::{AvatarGroup, avatar_group};

mod kbd;
mod kbd_labels;
mod kbd_values;
pub use kbd::{Kbd, KbdCombo, kbd, kbd_combo};
pub use kbd_values::KbdKeyState;
mod item_list;
mod look_static_collections;
mod table;
pub use item_list::{ItemList, ItemRow, item_list, item_row};
pub use recipes::{ItemListRecipes, TableRecipes};
pub use table::{
    Table, TableCell, table, table_body, table_cell, table_footer, table_head, table_header,
    table_row,
};
mod breadcrumb;
mod look_navigation;
mod navigation_context;
mod navigation_link;
mod navigation_menu;
mod navigation_values;
pub use navigation_link::{NavigationMenuLink, navigation_menu_link};
pub use navigation_values::{
    NavigationLinkState, NavigationTriggerState, PageItem, PaginationItemState, PaginationMode,
    PaginationSummary,
};
mod navigation_tokens;
mod pagination;
mod pagination_draft;
mod stepper;
pub use breadcrumb::{Breadcrumb, breadcrumb};
pub use navigation_menu::{NavigationMenu, navigation_menu};
pub use navigation_tokens::{NavigationTokens, StepperTokens};
pub use pagination::{Pagination, pagination};
pub use recipes::{BreadcrumbRecipes, NavigationMenuRecipes, PaginationRecipes, StepperRecipes};
pub use stepper::{IndicatorContent, StepIndicator, Stepper, StepperItem, stepper};

mod description_list;
mod heading;
mod look_metrics;
mod stat;
pub use description_list::{Description, DescriptionList, description, description_list};
pub use heading::{Heading, heading};
pub use recipes::{DescriptionListRecipes, StatRecipes};
pub use stat::{Stat, stat};

mod chat_composer;
mod chat_message;
mod chat_thread;
mod chat_tokens;
mod chat_values;
mod code_block;
mod code_editing;
mod code_gutter;
mod code_values;
mod composer_history;
mod look_chat;
mod typing_indicator;
pub use chat_composer::{ChatComposer, chat_composer};
pub use chat_message::{ChatMessage, chat_message};
pub use chat_thread::{ChatThread, chat_thread};
pub use chat_tokens::{ChatTokens, CodeBlockTokens};
pub use chat_values::{
    ChatAlign, ChatBubbleGroup, ChatComposerState, ChatMessageState, ChatMessageStatus,
    ChatMessageVariant, ChatReactionsSide, ChatSender, ChatThreadState, ComposerStatus,
    ComposerStatusPosition, TypingIndicatorState,
};
pub use code_block::{CodeBlock, code_block};
pub use code_values::CodeBlockState;
pub use recipes::{
    ChatComposerRecipes, ChatMessageRecipes, ChatThreadRecipes, CodeBlockRecipes,
    TypingIndicatorRecipes,
};
pub use typing_indicator::{TypingIndicator, typing_indicator};

mod collection_values;
mod component_values;
mod hover_card_values;
pub use collection_values::{
    ItemListTokens, ItemRowState, TableTokens, ToolbarEdge, ToolbarState, ToolbarTokens, TreeTokens,
};
pub use component_values::{
    AlertState, CardState, EmptyStateState, HeadingState, InputGroupMemberKind,
    InputGroupMemberState, InputGroupState, StatState, Trend,
};
pub use hover_card_values::{HoverCardSurfaceState, HoverCardTokens};

mod carousel_values;
mod toolbar_look;
pub use carousel_values::{CarouselMode, CarouselSurfaceState};
pub use component_values::AlertVariant;
pub use navigation_values::{
    BreadcrumbItemState, CollapsedSteps, StepIndicatorKind, StepStatus, StepperItemState,
    StepperState,
};
pub use values::ProgressVariant;

mod date_sheet;

mod date_time_sheet;

mod table_values;
pub use table_values::{DataTableGroupState, RowStatus};
