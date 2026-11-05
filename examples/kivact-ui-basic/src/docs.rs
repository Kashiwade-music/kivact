//! Role: Page and example definitions of the catalog, one file per component.
//!
//! A page file defines its demos as functions between the marker lines
//! `// example: <id>` and `// end example`. The page reads its own source
//! with `include_str!` and shows the text between the markers, so the code on
//! screen is the code that runs. To add a page, copy a file of `docs/`, list
//! its examples and add the module and its `page()` here. The types are in
//! `model`.

use crate::model::Page;
#[expect(
    clippy::disallowed_types,
    reason = "Static immutable catalog metadata is shared process-wide; no instance lifecycle or late-filled application state."
)]
use std::sync::LazyLock;

mod accordion;
mod alert;
mod alert_dialog;
mod aspect_ratio;
mod attachment;
mod avatar;
mod badge;
mod breadcrumb;
mod bubble;
mod button;
mod button_group;
mod calendar;
mod card;
mod carousel;
mod chart;
mod checkbox;
mod code_block;
mod collapsible;
mod combobox;
mod command;
mod context_menu;
mod data_table;
mod date_picker;
mod dialog;
mod direction;
mod drawer;
mod dropdown_menu;
mod empty;
mod field;
mod hover_card;
mod input;
mod input_group;
mod input_otp;
mod item;
mod kbd;
mod label;
mod marker;
mod menubar;
mod message;
mod message_scroller;
mod native_select;
mod navigation_menu;
mod pagination;
mod popover;
mod progress;
mod questionnaire;
mod radio_group;
mod resizable;
mod scroll_area;
mod select;
mod separator;
mod sheet;
mod sidebar;
mod skeleton;
mod slider;
mod spinner;
mod switch;
mod table;
mod tabs;
mod textarea;
mod toast;
mod toggle;
mod toggle_group;
mod tooltip;
mod trace;
mod typography;

/// Every page in alphabetical order.
#[expect(
    clippy::disallowed_types,
    reason = "Static immutable catalog metadata is shared process-wide; no instance lifecycle or late-filled application state."
)]
pub(crate) static PAGES: LazyLock<Vec<Page>> = LazyLock::new(|| {
    vec![
        accordion::page(),
        alert::page(),
        alert_dialog::page(),
        aspect_ratio::page(),
        attachment::page(),
        avatar::page(),
        badge::page(),
        breadcrumb::page(),
        bubble::page(),
        button::page(),
        button_group::page(),
        calendar::page(),
        card::page(),
        carousel::page(),
        chart::page(),
        checkbox::page(),
        code_block::page(),
        collapsible::page(),
        combobox::page(),
        command::page(),
        context_menu::page(),
        data_table::page(),
        date_picker::page(),
        dialog::page(),
        direction::page(),
        drawer::page(),
        dropdown_menu::page(),
        empty::page(),
        field::page(),
        hover_card::page(),
        input::page(),
        input_group::page(),
        input_otp::page(),
        item::page(),
        kbd::page(),
        label::page(),
        marker::page(),
        menubar::page(),
        message::page(),
        message_scroller::page(),
        native_select::page(),
        navigation_menu::page(),
        pagination::page(),
        popover::page(),
        progress::page(),
        questionnaire::page(),
        radio_group::page(),
        resizable::page(),
        scroll_area::page(),
        select::page(),
        separator::page(),
        sheet::page(),
        sidebar::page(),
        skeleton::page(),
        slider::page(),
        spinner::page(),
        switch::page(),
        table::page(),
        tabs::page(),
        textarea::page(),
        toast::page(),
        toggle::page(),
        toggle_group::page(),
        tooltip::page(),
        trace::page(),
        typography::page(),
    ]
});
