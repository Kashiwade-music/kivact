//! Role: The catalog pages in category order: each page's name and category, and the demo it shows.
use catalog_shell::{Example, Page};
use kivact::style::style;
use kivact::view::{IntoView, Node, column, text};

mod collections;
mod content;
mod controls;
mod dates;
mod feedback;
mod forms;
mod navigation;
mod overlays;
mod utilities;

/// The demo of a page: its title over its content.
fn labelled(title: &str, view: impl IntoView) -> Node {
    column((
        text(title.to_owned()).style(style().font_weight(600)),
        column(view).style(style().gap(12)),
    ))
    .style(style().gap(8))
}

/// All component pages, in the order of the list.
pub fn pages() -> Vec<Page> {
    [
        part_1(),
        part_2(),
        part_3(),
        part_4(),
        part_5(),
        part_6(),
        part_7(),
        part_8(),
        part_9(),
        part_10(),
        part_11(),
        part_12(),
        part_13(),
        part_14(),
        part_15(),
        part_16(),
        part_17(),
        part_18(),
        part_19(),
        part_20(),
        part_21(),
        part_22(),
    ]
    .into_iter()
    .flatten()
    .collect()
}

/// Part 1 of the page list, in list order.
fn part_1() -> Vec<Page> {
    vec![
        Page::new(
            "button",
            "Button",
            "Action",
            include_str!("pages/controls.rs"),
        )
        .example(Example::new(
            "button",
            "Button",
            "",
            controls::example_button,
        )),
        Page::new(
            "button-group",
            "Button Group",
            "Action",
            include_str!("pages/controls.rs"),
        )
        .example(Example::new(
            "button-group",
            "Button Group",
            "",
            controls::example_button_group,
        )),
        Page::new(
            "context-menu",
            "Context Menu",
            "Action",
            include_str!("pages/overlays.rs"),
        )
        .example(Example::new(
            "context-menu",
            "Context Menu",
            "",
            overlays::example_context_menu,
        )),
        Page::new(
            "dropdown-menu",
            "Dropdown Menu",
            "Action",
            include_str!("pages/overlays.rs"),
        )
        .example(Example::new(
            "dropdown-menu",
            "Dropdown Menu",
            "",
            overlays::example_dropdown_menu,
        )),
        Page::new(
            "icon-button",
            "Icon Button",
            "Action",
            include_str!("pages/controls.rs"),
        )
        .example(Example::new(
            "icon-button",
            "Icon Button",
            "",
            controls::example_icon_button,
        )),
    ]
}

/// Part 2 of the page list, in list order.
fn part_2() -> Vec<Page> {
    vec![
        Page::new("link", "Link", "Action", include_str!("pages/content.rs"))
            .example(Example::new("link", "Link", "", content::example_link)),
        Page::new(
            "more-menu",
            "More Menu",
            "Action",
            include_str!("pages/overlays.rs"),
        )
        .example(Example::new(
            "more-menu",
            "More Menu",
            "",
            overlays::example_more_menu,
        )),
        Page::new(
            "segmented-control",
            "Segmented Control",
            "Action",
            include_str!("pages/controls.rs"),
        )
        .example(Example::new(
            "segmented-control",
            "Segmented Control",
            "",
            controls::example_segmented_control,
        )),
        Page::new(
            "toggle-button",
            "Toggle Button",
            "Action",
            include_str!("pages/controls.rs"),
        )
        .example(Example::new(
            "toggle-button",
            "Toggle Button",
            "",
            controls::example_toggle_button,
        )),
        Page::new(
            "toolbar",
            "Toolbar",
            "Action",
            include_str!("pages/controls.rs"),
        )
        .example(Example::new(
            "toolbar",
            "Toolbar",
            "",
            controls::example_toolbar,
        )),
        Page::new("chat", "Chat", "Chat", include_str!("pages/collections.rs"))
            .example(Example::new("chat", "Chat", "", collections::example_chat)),
    ]
}

/// Part 3 of the page list, in list order.
fn part_3() -> Vec<Page> {
    vec![
        Page::new(
            "chat-dictation-button",
            "Chat Dictation Button",
            "Chat",
            include_str!("pages/collections.rs"),
        )
        .example(Example::new(
            "chat-dictation-button",
            "Chat Dictation Button",
            "",
            collections::example_chat_dictation_button,
        )),
        Page::new(
            "chat-tool-calls",
            "Chat Tool Calls",
            "Chat",
            include_str!("pages/collections.rs"),
        )
        .example(Example::new(
            "chat-tool-calls",
            "Chat Tool Calls",
            "",
            collections::example_chat_tool_calls,
        )),
        Page::new(
            "card",
            "Card",
            "Container",
            include_str!("pages/collections.rs"),
        )
        .example(Example::new("card", "Card", "", collections::example_card)),
        Page::new(
            "carousel",
            "Carousel",
            "Container",
            include_str!("pages/collections.rs"),
        )
        .example(Example::new(
            "carousel",
            "Carousel",
            "",
            collections::example_carousel,
        )),
        Page::new(
            "clickable-card",
            "Clickable Card",
            "Container",
            include_str!("pages/collections.rs"),
        )
        .example(Example::new(
            "clickable-card",
            "Clickable Card",
            "",
            collections::example_clickable_card,
        )),
    ]
}

/// Part 4 of the page list, in list order.
fn part_4() -> Vec<Page> {
    vec![
        Page::new(
            "collapsible",
            "Collapsible",
            "Container",
            include_str!("pages/collections.rs"),
        )
        .example(Example::new(
            "collapsible",
            "Collapsible",
            "",
            collections::example_collapsible,
        )),
        Page::new(
            "selectable-card",
            "Selectable Card",
            "Container",
            include_str!("pages/collections.rs"),
        )
        .example(Example::new(
            "selectable-card",
            "Selectable Card",
            "",
            collections::example_selectable_card,
        )),
        Page::new(
            "avatar",
            "Avatar",
            "Content",
            include_str!("pages/content.rs"),
        )
        .example(Example::new(
            "avatar",
            "Avatar",
            "",
            content::example_avatar,
        )),
        Page::new(
            "avatar-group",
            "Avatar Group",
            "Content",
            include_str!("pages/content.rs"),
        )
        .example(Example::new(
            "avatar-group",
            "Avatar Group",
            "",
            content::example_avatar,
        )),
        Page::new(
            "blockquote",
            "Blockquote",
            "Content",
            include_str!("pages/content.rs"),
        )
        .example(Example::new(
            "blockquote",
            "Blockquote",
            "",
            content::example_blockquote,
        )),
    ]
}

/// Part 5 of the page list, in list order.
fn part_5() -> Vec<Page> {
    vec![
        Page::new(
            "citation",
            "Citation",
            "Content",
            include_str!("pages/content.rs"),
        )
        .example(Example::new(
            "citation",
            "Citation",
            "",
            content::example_citation,
        )),
        Page::new("code", "Code", "Content", include_str!("pages/content.rs"))
            .example(Example::new("code", "Code", "", content::example_code)),
        Page::new(
            "code-block",
            "Code Block",
            "Content",
            include_str!("pages/content.rs"),
        )
        .example(Example::new(
            "code-block",
            "Code Block",
            "",
            content::example_code_block,
        )),
        Page::new(
            "empty-state",
            "Empty State",
            "Content",
            include_str!("pages/content.rs"),
        )
        .example(Example::new(
            "empty-state",
            "Empty State",
            "",
            content::example_empty_state,
        )),
        Page::new(
            "heading",
            "Heading",
            "Content",
            include_str!("pages/content.rs"),
        )
        .example(Example::new(
            "heading",
            "Heading",
            "",
            content::example_heading,
        )),
        Page::new("icon", "Icon", "Content", include_str!("pages/content.rs"))
            .example(Example::new("icon", "Icon", "", content::example_icon)),
        Page::new("kbd", "Kbd", "Content", include_str!("pages/content.rs")).example(Example::new(
            "kbd",
            "Kbd",
            "",
            content::example_kbd,
        )),
    ]
}

/// Part 6 of the page list, in list order.
fn part_6() -> Vec<Page> {
    vec![
        Page::new(
            "markdown",
            "Markdown",
            "Content",
            include_str!("pages/content.rs"),
        )
        .example(Example::new(
            "markdown",
            "Markdown",
            "",
            content::example_markdown,
        )),
        Page::new("text", "Text", "Content", include_str!("pages/content.rs"))
            .example(Example::new("text", "Text", "", content::example_text)),
        Page::new(
            "thumbnail",
            "Thumbnail",
            "Content",
            include_str!("pages/content.rs"),
        )
        .example(Example::new(
            "thumbnail",
            "Thumbnail",
            "",
            content::example_thumbnail,
        )),
        Page::new(
            "timer",
            "Timer",
            "Content",
            include_str!("pages/content.rs"),
        )
        .example(Example::new("timer", "Timer", "", content::example_timer)),
        Page::new(
            "timestamp",
            "Timestamp",
            "Content",
            include_str!("pages/content.rs"),
        )
        .example(Example::new(
            "timestamp",
            "Timestamp",
            "",
            content::example_timestamp,
        )),
        Page::new(
            "token",
            "Token",
            "Content",
            include_str!("pages/feedback.rs"),
        )
        .example(Example::new("token", "Token", "", feedback::example_token)),
        Page::new(
            "badge",
            "Badge",
            "Feedback & Status",
            include_str!("pages/feedback.rs"),
        )
        .example(Example::new("badge", "Badge", "", feedback::example_badge)),
    ]
}

/// Part 7 of the page list, in list order.
fn part_7() -> Vec<Page> {
    vec![
        Page::new(
            "banner",
            "Banner",
            "Feedback & Status",
            include_str!("pages/feedback.rs"),
        )
        .example(Example::new(
            "banner",
            "Banner",
            "",
            feedback::example_banner,
        )),
        Page::new(
            "progress-bar",
            "Progress Bar",
            "Feedback & Status",
            include_str!("pages/controls.rs"),
        )
        .example(Example::new(
            "progress-bar",
            "Progress Bar",
            "",
            controls::example_progress_bar,
        )),
        Page::new(
            "skeleton",
            "Skeleton",
            "Feedback & Status",
            include_str!("pages/feedback.rs"),
        )
        .example(Example::new(
            "skeleton",
            "Skeleton",
            "",
            feedback::example_skeleton,
        )),
        Page::new(
            "spinner",
            "Spinner",
            "Feedback & Status",
            include_str!("pages/feedback.rs"),
        )
        .example(Example::new(
            "spinner",
            "Spinner",
            "",
            feedback::example_spinner,
        )),
        Page::new(
            "status-dot",
            "Status Dot",
            "Feedback & Status",
            include_str!("pages/feedback.rs"),
        )
        .example(Example::new(
            "status-dot",
            "Status Dot",
            "",
            feedback::example_status_dot,
        )),
    ]
}

/// Part 8 of the page list, in list order.
fn part_8() -> Vec<Page> {
    vec![
        Page::new(
            "calendar",
            "Calendar",
            "Form Controls",
            include_str!("pages/dates.rs"),
        )
        .example(Example::new(
            "calendar",
            "Calendar",
            "",
            dates::example_calendar,
        )),
        Page::new(
            "checkbox-input",
            "Checkbox Input",
            "Form Controls",
            include_str!("pages/controls.rs"),
        )
        .example(Example::new(
            "checkbox-input",
            "Checkbox Input",
            "",
            controls::example_checkbox_input,
        )),
        Page::new(
            "checkbox-list",
            "Checkbox List",
            "Form Controls",
            include_str!("pages/forms.rs"),
        )
        .example(Example::new(
            "checkbox-list",
            "Checkbox List",
            "",
            forms::example_checkbox_list,
        )),
        Page::new(
            "complex-selector",
            "Complex Selector",
            "Form Controls",
            include_str!("pages/forms.rs"),
        )
        .example(Example::new(
            "complex-selector",
            "Complex Selector",
            "",
            forms::example_complex_selector,
        )),
        Page::new(
            "date-input",
            "Date Input",
            "Form Controls",
            include_str!("pages/dates.rs"),
        )
        .example(Example::new(
            "date-input",
            "Date Input",
            "",
            dates::example_date_input,
        )),
    ]
}

/// Part 9 of the page list, in list order.
fn part_9() -> Vec<Page> {
    vec![
        Page::new(
            "date-range-input",
            "Date Range Input",
            "Form Controls",
            include_str!("pages/dates.rs"),
        )
        .example(Example::new(
            "date-range-input",
            "Date Range Input",
            "",
            dates::example_date_range_input,
        )),
        Page::new(
            "date-time-input",
            "Date Time Input",
            "Form Controls",
            include_str!("pages/dates.rs"),
        )
        .example(Example::new(
            "date-time-input",
            "Date Time Input",
            "",
            dates::example_date_time_input,
        )),
        Page::new(
            "field",
            "Field",
            "Form Controls",
            include_str!("pages/forms.rs"),
        )
        .example(Example::new("field", "Field", "", forms::example_field)),
        Page::new(
            "field-status",
            "Field Status",
            "Form Controls",
            include_str!("pages/forms.rs"),
        )
        .example(Example::new(
            "field-status",
            "Field Status",
            "",
            forms::example_field_status,
        )),
        Page::new(
            "file-input",
            "File Input",
            "Form Controls",
            include_str!("pages/forms.rs"),
        )
        .example(Example::new(
            "file-input",
            "File Input",
            "",
            forms::example_file_input,
        )),
    ]
}

/// Part 10 of the page list, in list order.
fn part_10() -> Vec<Page> {
    vec![
        Page::new(
            "indicator",
            "Indicator",
            "Form Controls",
            include_str!("pages/controls.rs"),
        )
        .example(Example::new(
            "indicator",
            "Indicator",
            "",
            controls::example_indicator,
        )),
        Page::new(
            "input-group",
            "Input Group",
            "Form Controls",
            include_str!("pages/forms.rs"),
        )
        .example(Example::new(
            "input-group",
            "Input Group",
            "",
            forms::example_input_group,
        )),
        Page::new(
            "multi-selector",
            "Multi Selector",
            "Form Controls",
            include_str!("pages/forms.rs"),
        )
        .example(Example::new(
            "multi-selector",
            "Multi Selector",
            "",
            forms::example_multi_selector,
        )),
        Page::new(
            "number-input",
            "Number Input",
            "Form Controls",
            include_str!("pages/forms.rs"),
        )
        .example(Example::new(
            "number-input",
            "Number Input",
            "",
            forms::example_number_input,
        )),
        Page::new(
            "power-search",
            "Power Search",
            "Form Controls",
            include_str!("pages/forms.rs"),
        )
        .example(Example::new(
            "power-search",
            "Power Search",
            "",
            forms::example_power_search,
        )),
    ]
}

/// Part 11 of the page list, in list order.
fn part_11() -> Vec<Page> {
    vec![
        Page::new(
            "radio-list",
            "Radio List",
            "Form Controls",
            include_str!("pages/controls.rs"),
        )
        .example(Example::new(
            "radio-list",
            "Radio List",
            "",
            controls::example_radio_list,
        )),
        Page::new(
            "rich-text-editor",
            "Rich Text Editor",
            "Form Controls",
            include_str!("pages/forms.rs"),
        )
        .example(Example::new(
            "rich-text-editor",
            "Rich Text Editor",
            "",
            forms::example_rich_text_editor,
        )),
        Page::new(
            "selector",
            "Selector",
            "Form Controls",
            include_str!("pages/forms.rs"),
        )
        .example(Example::new(
            "selector",
            "Selector",
            "",
            forms::example_selector,
        )),
        Page::new(
            "slider",
            "Slider",
            "Form Controls",
            include_str!("pages/controls.rs"),
        )
        .example(Example::new(
            "slider",
            "Slider",
            "",
            controls::example_slider,
        )),
        Page::new(
            "switch",
            "Switch",
            "Form Controls",
            include_str!("pages/controls.rs"),
        )
        .example(Example::new(
            "switch",
            "Switch",
            "",
            controls::example_switch,
        )),
    ]
}

/// Part 12 of the page list, in list order.
fn part_12() -> Vec<Page> {
    vec![
        Page::new(
            "text-area",
            "Text Area",
            "Form Controls",
            include_str!("pages/forms.rs"),
        )
        .example(Example::new(
            "text-area",
            "Text Area",
            "",
            forms::example_text_area,
        )),
        Page::new(
            "text-input",
            "Text Input",
            "Form Controls",
            include_str!("pages/forms.rs"),
        )
        .example(Example::new(
            "text-input",
            "Text Input",
            "",
            forms::example_text_input,
        )),
        Page::new(
            "time-input",
            "Time Input",
            "Form Controls",
            include_str!("pages/dates.rs"),
        )
        .example(Example::new(
            "time-input",
            "Time Input",
            "",
            dates::example_time_input,
        )),
        Page::new(
            "tokenizer",
            "Tokenizer",
            "Form Controls",
            include_str!("pages/forms.rs"),
        )
        .example(Example::new(
            "tokenizer",
            "Tokenizer",
            "",
            forms::example_tokenizer,
        )),
        Page::new(
            "typeahead",
            "Typeahead",
            "Form Controls",
            include_str!("pages/forms.rs"),
        )
        .example(Example::new(
            "typeahead",
            "Typeahead",
            "",
            forms::example_typeahead,
        )),
    ]
}

/// Part 13 of the page list, in list order.
fn part_13() -> Vec<Page> {
    vec![
        Page::new(
            "app-shell",
            "App Shell",
            "Layout",
            include_str!("pages/navigation.rs"),
        )
        .example(Example::new(
            "app-shell",
            "App Shell",
            "",
            navigation::example_app_shell,
        )),
        Page::new(
            "aspect-ratio",
            "Aspect Ratio",
            "Layout",
            include_str!("pages/collections.rs"),
        )
        .example(Example::new(
            "aspect-ratio",
            "Aspect Ratio",
            "",
            collections::example_aspect_ratio,
        )),
        Page::new(
            "center",
            "Center",
            "Layout",
            include_str!("pages/collections.rs"),
        )
        .example(Example::new(
            "center",
            "Center",
            "",
            collections::example_center,
        )),
        Page::new(
            "divider",
            "Divider",
            "Layout",
            include_str!("pages/collections.rs"),
        )
        .example(Example::new(
            "divider",
            "Divider",
            "",
            collections::example_divider,
        )),
        Page::new(
            "form-layout",
            "Form Layout",
            "Layout",
            include_str!("pages/collections.rs"),
        )
        .example(Example::new(
            "form-layout",
            "Form Layout",
            "",
            collections::example_form_layout,
        )),
    ]
}

/// Part 14 of the page list, in list order.
fn part_14() -> Vec<Page> {
    vec![
        Page::new(
            "grid",
            "Grid",
            "Layout",
            include_str!("pages/collections.rs"),
        )
        .example(Example::new("grid", "Grid", "", collections::example_grid)),
        Page::new(
            "layout",
            "Layout",
            "Layout",
            include_str!("pages/collections.rs"),
        )
        .example(Example::new(
            "layout",
            "Layout",
            "",
            collections::example_layout,
        )),
        Page::new(
            "resizable",
            "Resizable",
            "Layout",
            include_str!("pages/collections.rs"),
        )
        .example(Example::new(
            "resizable",
            "Resizable",
            "",
            collections::example_resizable,
        )),
        Page::new(
            "scrollable-area",
            "Scrollable Area",
            "Layout",
            include_str!("pages/collections.rs"),
        )
        .example(Example::new(
            "scrollable-area",
            "Scrollable Area",
            "",
            collections::example_scrollable_area,
        )),
        Page::new(
            "section",
            "Section",
            "Layout",
            include_str!("pages/collections.rs"),
        )
        .example(Example::new(
            "section",
            "Section",
            "",
            collections::example_section,
        )),
    ]
}

/// Part 15 of the page list, in list order.
fn part_15() -> Vec<Page> {
    vec![
        Page::new(
            "stack",
            "Stack",
            "Layout",
            include_str!("pages/collections.rs"),
        )
        .example(Example::new(
            "stack",
            "Stack",
            "",
            collections::example_stack,
        )),
        Page::new(
            "breadcrumbs",
            "Breadcrumbs",
            "Navigation",
            include_str!("pages/collections.rs"),
        )
        .example(Example::new(
            "breadcrumbs",
            "Breadcrumbs",
            "",
            collections::example_breadcrumbs,
        )),
        Page::new(
            "mobile-nav",
            "Mobile Nav",
            "Navigation",
            include_str!("pages/navigation.rs"),
        )
        .example(Example::new(
            "mobile-nav",
            "Mobile Nav",
            "",
            navigation::example_mobile_nav,
        )),
        Page::new(
            "nav-icon",
            "Nav Icon",
            "Navigation",
            include_str!("pages/collections.rs"),
        )
        .example(Example::new(
            "nav-icon",
            "Nav Icon",
            "",
            collections::example_nav_icon,
        )),
        Page::new(
            "nav-menu",
            "Nav Menu",
            "Navigation",
            include_str!("pages/navigation.rs"),
        )
        .example(Example::new(
            "nav-menu",
            "Nav Menu",
            "",
            navigation::example_nav_menu,
        )),
    ]
}

/// Part 16 of the page list, in list order.
fn part_16() -> Vec<Page> {
    vec![
        Page::new(
            "outline",
            "Outline",
            "Navigation",
            include_str!("pages/navigation.rs"),
        )
        .example(Example::new(
            "outline",
            "Outline",
            "",
            navigation::example_outline,
        )),
        Page::new(
            "pagination",
            "Pagination",
            "Navigation",
            include_str!("pages/collections.rs"),
        )
        .example(Example::new(
            "pagination",
            "Pagination",
            "",
            collections::example_pagination,
        )),
        Page::new(
            "side-nav",
            "Side Nav",
            "Navigation",
            include_str!("pages/navigation.rs"),
        )
        .example(Example::new(
            "side-nav",
            "Side Nav",
            "",
            navigation::example_sidebar,
        )),
        Page::new(
            "step",
            "Step",
            "Navigation",
            include_str!("pages/navigation.rs"),
        )
        .example(Example::new(
            "step",
            "Step",
            "",
            navigation::example_stepper,
        )),
        Page::new(
            "stepper",
            "Stepper",
            "Navigation",
            include_str!("pages/navigation.rs"),
        )
        .example(Example::new(
            "stepper",
            "Stepper",
            "",
            navigation::example_stepper,
        )),
    ]
}

/// Part 17 of the page list, in list order.
fn part_17() -> Vec<Page> {
    vec![
        Page::new(
            "tab-list",
            "Tab List",
            "Navigation",
            include_str!("pages/navigation.rs"),
        )
        .example(Example::new(
            "tab-list",
            "Tab List",
            "",
            navigation::example_tab_list,
        )),
        Page::new(
            "tab-menu",
            "Tab Menu",
            "Navigation",
            include_str!("pages/navigation.rs"),
        )
        .example(Example::new(
            "tab-menu",
            "Tab Menu",
            "",
            navigation::example_tab_menu,
        )),
        Page::new(
            "top-nav",
            "Top Nav",
            "Navigation",
            include_str!("pages/navigation.rs"),
        )
        .example(Example::new(
            "top-nav",
            "Top Nav",
            "",
            navigation::example_top_nav,
        )),
        Page::new(
            "alert-dialog",
            "Alert Dialog",
            "Overlay",
            include_str!("pages/overlays.rs"),
        )
        .example(Example::new(
            "alert-dialog",
            "Alert Dialog",
            "",
            overlays::example_alert_dialog,
        )),
        Page::new(
            "bottom-sheet",
            "Bottom Sheet",
            "Overlay",
            include_str!("pages/overlays.rs"),
        )
        .example(Example::new(
            "bottom-sheet",
            "Bottom Sheet",
            "",
            overlays::example_bottom_sheet,
        )),
    ]
}

/// Part 18 of the page list, in list order.
fn part_18() -> Vec<Page> {
    vec![
        Page::new(
            "bottom-sheet-switcher",
            "Bottom Sheet Switcher",
            "Overlay",
            include_str!("pages/overlays.rs"),
        )
        .example(Example::new(
            "bottom-sheet-switcher",
            "Bottom Sheet Switcher",
            "",
            overlays::example_bottom_sheet_switcher,
        )),
        Page::new(
            "command-palette",
            "Command Palette",
            "Overlay",
            include_str!("pages/overlays.rs"),
        )
        .example(Example::new(
            "command-palette",
            "Command Palette",
            "",
            overlays::example_command_palette,
        )),
        Page::new(
            "dialog",
            "Dialog",
            "Overlay",
            include_str!("pages/overlays.rs"),
        )
        .example(Example::new(
            "dialog",
            "Dialog",
            "",
            overlays::example_dialog,
        )),
        Page::new(
            "drawer",
            "Drawer",
            "Overlay",
            include_str!("pages/overlays.rs"),
        )
        .example(Example::new(
            "drawer",
            "Drawer",
            "",
            overlays::example_drawer,
        )),
        Page::new(
            "hover-card",
            "Hover Card",
            "Overlay",
            include_str!("pages/overlays.rs"),
        )
        .example(Example::new(
            "hover-card",
            "Hover Card",
            "",
            overlays::example_hover_card,
        )),
    ]
}

/// Part 19 of the page list, in list order.
fn part_19() -> Vec<Page> {
    vec![
        Page::new(
            "lightbox",
            "Lightbox",
            "Overlay",
            include_str!("pages/overlays.rs"),
        )
        .example(Example::new(
            "lightbox",
            "Lightbox",
            "",
            overlays::example_lightbox,
        )),
        Page::new(
            "overlay",
            "Overlay",
            "Overlay",
            include_str!("pages/overlays.rs"),
        )
        .example(Example::new(
            "overlay",
            "Overlay",
            "",
            overlays::example_overlay,
        )),
        Page::new(
            "popover",
            "Popover",
            "Overlay",
            include_str!("pages/overlays.rs"),
        )
        .example(Example::new(
            "popover",
            "Popover",
            "",
            overlays::example_popover,
        )),
        Page::new(
            "toast",
            "Toast",
            "Overlay",
            include_str!("pages/overlays.rs"),
        )
        .example(Example::new("toast", "Toast", "", overlays::example_toast)),
        Page::new(
            "tooltip",
            "Tooltip",
            "Overlay",
            include_str!("pages/overlays.rs"),
        )
        .example(Example::new(
            "tooltip",
            "Tooltip",
            "",
            overlays::example_tooltip,
        )),
    ]
}

/// Part 20 of the page list, in list order.
fn part_20() -> Vec<Page> {
    vec![
        Page::new(
            "item",
            "Item",
            "Table & List",
            include_str!("pages/collections.rs"),
        )
        .example(Example::new("item", "Item", "", collections::example_list)),
        Page::new(
            "list",
            "List",
            "Table & List",
            include_str!("pages/collections.rs"),
        )
        .example(Example::new("list", "List", "", collections::example_list)),
        Page::new(
            "metadata-list",
            "Metadata List",
            "Table & List",
            include_str!("pages/collections.rs"),
        )
        .example(Example::new(
            "metadata-list",
            "Metadata List",
            "",
            collections::example_metadata_list,
        )),
        Page::new(
            "overflow-list",
            "Overflow List",
            "Table & List",
            include_str!("pages/collections.rs"),
        )
        .example(Example::new(
            "overflow-list",
            "Overflow List",
            "",
            collections::example_overflow_list,
        )),
        Page::new(
            "table",
            "Table",
            "Table & List",
            include_str!("pages/collections.rs"),
        )
        .example(Example::new(
            "table",
            "Table",
            "",
            collections::example_table,
        )),
    ]
}

/// Part 21 of the page list, in list order.
fn part_21() -> Vec<Page> {
    vec![
        Page::new(
            "tree-list",
            "Tree List",
            "Table & List",
            include_str!("pages/collections.rs"),
        )
        .example(Example::new(
            "tree-list",
            "Tree List",
            "",
            collections::example_tree_list,
        )),
        Page::new(
            "internationalization-provider",
            "Internationalization Provider",
            "Utility",
            include_str!("pages/utilities.rs"),
        )
        .example(Example::new(
            "internationalization-provider",
            "Internationalization Provider",
            "",
            utilities::example_internationalization_provider,
        )),
        Page::new(
            "layer",
            "Layer",
            "Utility",
            include_str!("pages/overlays.rs"),
        )
        .example(Example::new(
            "layer",
            "Layer",
            "",
            overlays::example_popover,
        )),
        Page::new(
            "link-provider",
            "Link Provider",
            "Utility",
            include_str!("pages/utilities.rs"),
        )
        .example(Example::new(
            "link-provider",
            "Link Provider",
            "",
            utilities::example_link_provider,
        )),
        Page::new(
            "media-theme",
            "Media Theme",
            "Utility",
            include_str!("pages/utilities.rs"),
        )
        .example(Example::new(
            "media-theme",
            "Media Theme",
            "",
            utilities::example_media_theme,
        )),
    ]
}

/// Part 22 of the page list, in list order.
fn part_22() -> Vec<Page> {
    vec![
        Page::new(
            "syntax-theme",
            "Syntax Theme",
            "Utility",
            include_str!("pages/utilities.rs"),
        )
        .example(Example::new(
            "syntax-theme",
            "Syntax Theme",
            "",
            utilities::example_syntax_theme,
        )),
        Page::new(
            "theme",
            "Theme",
            "Utility",
            include_str!("pages/utilities.rs"),
        )
        .example(Example::new("theme", "Theme", "", utilities::example_theme)),
        Page::new(
            "visually-hidden",
            "Visually Hidden",
            "Utility",
            include_str!("pages/content.rs"),
        )
        .example(Example::new(
            "visually-hidden",
            "Visually Hidden",
            "",
            content::example_visually_hidden,
        )),
    ]
}
