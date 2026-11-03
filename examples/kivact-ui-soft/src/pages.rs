//! Role: Register the Astryx component catalog pages in docsite category order.

/// One component's page in the catalog.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PageInfo {
    pub slug: &'static str,
    pub name: &'static str,
    pub category: &'static str,
}

/// All registered component pages, grouped by category.
pub fn pages() -> &'static [PageInfo] {
    &PAGES
}

const PAGES: [PageInfo; 113] = [
    PageInfo {
        slug: "button",
        name: "Button",
        category: "Action",
    },
    PageInfo {
        slug: "button-group",
        name: "Button Group",
        category: "Action",
    },
    PageInfo {
        slug: "context-menu",
        name: "Context Menu",
        category: "Action",
    },
    PageInfo {
        slug: "dropdown-menu",
        name: "Dropdown Menu",
        category: "Action",
    },
    PageInfo {
        slug: "icon-button",
        name: "Icon Button",
        category: "Action",
    },
    PageInfo {
        slug: "link",
        name: "Link",
        category: "Action",
    },
    PageInfo {
        slug: "more-menu",
        name: "More Menu",
        category: "Action",
    },
    PageInfo {
        slug: "segmented-control",
        name: "Segmented Control",
        category: "Action",
    },
    PageInfo {
        slug: "toggle-button",
        name: "Toggle Button",
        category: "Action",
    },
    PageInfo {
        slug: "toolbar",
        name: "Toolbar",
        category: "Action",
    },
    PageInfo {
        slug: "chat",
        name: "Chat",
        category: "Chat",
    },
    PageInfo {
        slug: "chat-dictation-button",
        name: "Chat Dictation Button",
        category: "Chat",
    },
    PageInfo {
        slug: "chat-tool-calls",
        name: "Chat Tool Calls",
        category: "Chat",
    },
    PageInfo {
        slug: "card",
        name: "Card",
        category: "Container",
    },
    PageInfo {
        slug: "carousel",
        name: "Carousel",
        category: "Container",
    },
    PageInfo {
        slug: "clickable-card",
        name: "Clickable Card",
        category: "Container",
    },
    PageInfo {
        slug: "collapsible",
        name: "Collapsible",
        category: "Container",
    },
    PageInfo {
        slug: "selectable-card",
        name: "Selectable Card",
        category: "Container",
    },
    PageInfo {
        slug: "avatar",
        name: "Avatar",
        category: "Content",
    },
    PageInfo {
        slug: "avatar-group",
        name: "Avatar Group",
        category: "Content",
    },
    PageInfo {
        slug: "blockquote",
        name: "Blockquote",
        category: "Content",
    },
    PageInfo {
        slug: "citation",
        name: "Citation",
        category: "Content",
    },
    PageInfo {
        slug: "code",
        name: "Code",
        category: "Content",
    },
    PageInfo {
        slug: "code-block",
        name: "Code Block",
        category: "Content",
    },
    PageInfo {
        slug: "empty-state",
        name: "Empty State",
        category: "Content",
    },
    PageInfo {
        slug: "heading",
        name: "Heading",
        category: "Content",
    },
    PageInfo {
        slug: "icon",
        name: "Icon",
        category: "Content",
    },
    PageInfo {
        slug: "kbd",
        name: "Kbd",
        category: "Content",
    },
    PageInfo {
        slug: "markdown",
        name: "Markdown",
        category: "Content",
    },
    PageInfo {
        slug: "text",
        name: "Text",
        category: "Content",
    },
    PageInfo {
        slug: "thumbnail",
        name: "Thumbnail",
        category: "Content",
    },
    PageInfo {
        slug: "timer",
        name: "Timer",
        category: "Content",
    },
    PageInfo {
        slug: "timestamp",
        name: "Timestamp",
        category: "Content",
    },
    PageInfo {
        slug: "token",
        name: "Token",
        category: "Content",
    },
    PageInfo {
        slug: "badge",
        name: "Badge",
        category: "Feedback & Status",
    },
    PageInfo {
        slug: "banner",
        name: "Banner",
        category: "Feedback & Status",
    },
    PageInfo {
        slug: "progress-bar",
        name: "Progress Bar",
        category: "Feedback & Status",
    },
    PageInfo {
        slug: "skeleton",
        name: "Skeleton",
        category: "Feedback & Status",
    },
    PageInfo {
        slug: "spinner",
        name: "Spinner",
        category: "Feedback & Status",
    },
    PageInfo {
        slug: "status-dot",
        name: "Status Dot",
        category: "Feedback & Status",
    },
    PageInfo {
        slug: "calendar",
        name: "Calendar",
        category: "Form Controls",
    },
    PageInfo {
        slug: "checkbox-input",
        name: "Checkbox Input",
        category: "Form Controls",
    },
    PageInfo {
        slug: "checkbox-list",
        name: "Checkbox List",
        category: "Form Controls",
    },
    PageInfo {
        slug: "complex-selector",
        name: "Complex Selector",
        category: "Form Controls",
    },
    PageInfo {
        slug: "date-input",
        name: "Date Input",
        category: "Form Controls",
    },
    PageInfo {
        slug: "date-range-input",
        name: "Date Range Input",
        category: "Form Controls",
    },
    PageInfo {
        slug: "date-time-input",
        name: "Date Time Input",
        category: "Form Controls",
    },
    PageInfo {
        slug: "field",
        name: "Field",
        category: "Form Controls",
    },
    PageInfo {
        slug: "field-status",
        name: "Field Status",
        category: "Form Controls",
    },
    PageInfo {
        slug: "file-input",
        name: "File Input",
        category: "Form Controls",
    },
    PageInfo {
        slug: "indicator",
        name: "Indicator",
        category: "Form Controls",
    },
    PageInfo {
        slug: "input-group",
        name: "Input Group",
        category: "Form Controls",
    },
    PageInfo {
        slug: "multi-selector",
        name: "Multi Selector",
        category: "Form Controls",
    },
    PageInfo {
        slug: "number-input",
        name: "Number Input",
        category: "Form Controls",
    },
    PageInfo {
        slug: "power-search",
        name: "Power Search",
        category: "Form Controls",
    },
    PageInfo {
        slug: "radio-list",
        name: "Radio List",
        category: "Form Controls",
    },
    PageInfo {
        slug: "rich-text-editor",
        name: "Rich Text Editor",
        category: "Form Controls",
    },
    PageInfo {
        slug: "selector",
        name: "Selector",
        category: "Form Controls",
    },
    PageInfo {
        slug: "slider",
        name: "Slider",
        category: "Form Controls",
    },
    PageInfo {
        slug: "switch",
        name: "Switch",
        category: "Form Controls",
    },
    PageInfo {
        slug: "text-area",
        name: "Text Area",
        category: "Form Controls",
    },
    PageInfo {
        slug: "text-input",
        name: "Text Input",
        category: "Form Controls",
    },
    PageInfo {
        slug: "time-input",
        name: "Time Input",
        category: "Form Controls",
    },
    PageInfo {
        slug: "tokenizer",
        name: "Tokenizer",
        category: "Form Controls",
    },
    PageInfo {
        slug: "typeahead",
        name: "Typeahead",
        category: "Form Controls",
    },
    PageInfo {
        slug: "app-shell",
        name: "App Shell",
        category: "Layout",
    },
    PageInfo {
        slug: "aspect-ratio",
        name: "Aspect Ratio",
        category: "Layout",
    },
    PageInfo {
        slug: "center",
        name: "Center",
        category: "Layout",
    },
    PageInfo {
        slug: "divider",
        name: "Divider",
        category: "Layout",
    },
    PageInfo {
        slug: "form-layout",
        name: "Form Layout",
        category: "Layout",
    },
    PageInfo {
        slug: "grid",
        name: "Grid",
        category: "Layout",
    },
    PageInfo {
        slug: "layout",
        name: "Layout",
        category: "Layout",
    },
    PageInfo {
        slug: "resizable",
        name: "Resizable",
        category: "Layout",
    },
    PageInfo {
        slug: "scrollable-area",
        name: "Scrollable Area",
        category: "Layout",
    },
    PageInfo {
        slug: "section",
        name: "Section",
        category: "Layout",
    },
    PageInfo {
        slug: "stack",
        name: "Stack",
        category: "Layout",
    },
    PageInfo {
        slug: "breadcrumbs",
        name: "Breadcrumbs",
        category: "Navigation",
    },
    PageInfo {
        slug: "mobile-nav",
        name: "Mobile Nav",
        category: "Navigation",
    },
    PageInfo {
        slug: "nav-icon",
        name: "Nav Icon",
        category: "Navigation",
    },
    PageInfo {
        slug: "nav-menu",
        name: "Nav Menu",
        category: "Navigation",
    },
    PageInfo {
        slug: "outline",
        name: "Outline",
        category: "Navigation",
    },
    PageInfo {
        slug: "pagination",
        name: "Pagination",
        category: "Navigation",
    },
    PageInfo {
        slug: "side-nav",
        name: "Side Nav",
        category: "Navigation",
    },
    PageInfo {
        slug: "step",
        name: "Step",
        category: "Navigation",
    },
    PageInfo {
        slug: "stepper",
        name: "Stepper",
        category: "Navigation",
    },
    PageInfo {
        slug: "tab-list",
        name: "Tab List",
        category: "Navigation",
    },
    PageInfo {
        slug: "tab-menu",
        name: "Tab Menu",
        category: "Navigation",
    },
    PageInfo {
        slug: "top-nav",
        name: "Top Nav",
        category: "Navigation",
    },
    PageInfo {
        slug: "alert-dialog",
        name: "Alert Dialog",
        category: "Overlay",
    },
    PageInfo {
        slug: "bottom-sheet",
        name: "Bottom Sheet",
        category: "Overlay",
    },
    PageInfo {
        slug: "bottom-sheet-switcher",
        name: "Bottom Sheet Switcher",
        category: "Overlay",
    },
    PageInfo {
        slug: "command-palette",
        name: "Command Palette",
        category: "Overlay",
    },
    PageInfo {
        slug: "dialog",
        name: "Dialog",
        category: "Overlay",
    },
    PageInfo {
        slug: "drawer",
        name: "Drawer",
        category: "Overlay",
    },
    PageInfo {
        slug: "hover-card",
        name: "Hover Card",
        category: "Overlay",
    },
    PageInfo {
        slug: "lightbox",
        name: "Lightbox",
        category: "Overlay",
    },
    PageInfo {
        slug: "overlay",
        name: "Overlay",
        category: "Overlay",
    },
    PageInfo {
        slug: "popover",
        name: "Popover",
        category: "Overlay",
    },
    PageInfo {
        slug: "toast",
        name: "Toast",
        category: "Overlay",
    },
    PageInfo {
        slug: "tooltip",
        name: "Tooltip",
        category: "Overlay",
    },
    PageInfo {
        slug: "item",
        name: "Item",
        category: "Table & List",
    },
    PageInfo {
        slug: "list",
        name: "List",
        category: "Table & List",
    },
    PageInfo {
        slug: "metadata-list",
        name: "Metadata List",
        category: "Table & List",
    },
    PageInfo {
        slug: "overflow-list",
        name: "Overflow List",
        category: "Table & List",
    },
    PageInfo {
        slug: "table",
        name: "Table",
        category: "Table & List",
    },
    PageInfo {
        slug: "tree-list",
        name: "Tree List",
        category: "Table & List",
    },
    PageInfo {
        slug: "internationalization-provider",
        name: "Internationalization Provider",
        category: "Utility",
    },
    PageInfo {
        slug: "layer",
        name: "Layer",
        category: "Utility",
    },
    PageInfo {
        slug: "link-provider",
        name: "Link Provider",
        category: "Utility",
    },
    PageInfo {
        slug: "media-theme",
        name: "Media Theme",
        category: "Utility",
    },
    PageInfo {
        slug: "syntax-theme",
        name: "Syntax Theme",
        category: "Utility",
    },
    PageInfo {
        slug: "theme",
        name: "Theme",
        category: "Utility",
    },
    PageInfo {
        slug: "visually-hidden",
        name: "Visually Hidden",
        category: "Utility",
    },
];
