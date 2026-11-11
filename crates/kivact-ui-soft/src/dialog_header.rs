//! Role: Lay out Soft dialog and drawer headings.
use crate::recipes_dialog::motion;

use kivact::state::Callback;
use kivact::state::IntoText;
use kivact::state::IntoValue;
use kivact::style::Align;
use kivact::style::style;
use kivact::ui::ButtonVariant;
use kivact::ui::button;
use kivact::view::Element;
use kivact::view::IntoView;
use kivact::view::Node;
use kivact::view::Role;
use kivact::view::View;
use kivact::view::column;
use kivact::view::icon;
use kivact::view::row;
use kivact::view::text;
use std::rc::Rc;

/// Negative inset applied around the trailing header controls.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EdgeCompensation {
    /// Compensate the inline end by eight pixels.
    Inline,
    /// Compensate the top and bottom by eight pixels.
    Block,
    /// Apply both inline and block compensation.
    All,
}

/// Dialog heading with optional subtitle, side content and close action.
// Independent once-mounted title/subtitle/start/end slots and optional close callback plus construction compensation/divider input; no dialog visibility or close phase is stored.
pub struct DialogHeader {
    title: View,
    // xtask-allow(state-fields): option: once-mounted subtitle slot, not dialog lifecycle.
    subtitle: Option<View>,
    // xtask-allow(state-fields): option: once-mounted leading slot, independent of other content slots.
    start: Option<View>,
    // xtask-allow(state-fields): option: once-mounted trailing slot, independent of other content slots.
    end: Option<View>,
    close: Option<Rc<dyn Fn()>>,
    // xtask-allow(state-fields): option: caller override from header_end_compensation; absent compensates both axes when a close button exists and nothing otherwise.
    compensation: Option<EdgeCompensation>,
    divider: kivact::state::Value<bool>,
}

/// Create a heading without subtitle, side content, divider or close action.
pub fn dialog_header(title: impl IntoText) -> DialogHeader {
    DialogHeader {
        title: text(title)
            .with_node(move |node| node.role(Role::Heading))
            .style(|| {
                let p = kivact::style::use_tokens::<crate::Tokens>();
                style()
                    .font_size(20)
                    .line_height(1.4)
                    .weight(600)
                    .color(p.colors.text_primary)
            })
            .into_view(),
        subtitle: None,
        start: None,
        end: None,
        close: None,
        compensation: None,
        divider: false.into_value(),
    }
}

/// Drawer headings are visual. The enclosing drawer supplies its own name.
pub fn drawer_header(title: impl IntoText) -> DrawerHeader {
    DrawerHeader(dialog_header(title))
}

/// Visual drawer heading; the enclosing drawer owns its accessible name.
pub struct DrawerHeader(DialogHeader);

impl DialogHeader {
    /// Add secondary text below the title.
    pub fn subtitle(mut self, subtitle: impl IntoText) -> Self {
        self.subtitle = Some(
            text(subtitle)
                .style(|| {
                    style().font_size(12).line_height(20.0 / 14.0).color(
                        kivact::style::use_tokens::<crate::Tokens>()
                            .colors
                            .text_secondary,
                    )
                })
                .into_view(),
        );
        self
    }

    /// Mount content once before the title.
    pub fn header_start(mut self, content: impl IntoView) -> Self {
        self.start = Some(content.into_view());
        self
    }

    /// Mount content once beside the trailing close control.
    pub fn header_end(mut self, content: impl IntoView) -> Self {
        self.end = Some(content.into_view());
        self
    }

    /// Override the trailing-control inset; a close control otherwise compensates both axes.
    pub fn header_end_compensation(mut self, compensation: EdgeCompensation) -> Self {
        self.compensation = Some(compensation);
        self
    }

    /// Enable the bottom divider from a read-only input; false by default.
    pub fn header_divider(mut self, divider: impl IntoValue<bool>) -> Self {
        self.divider = divider.into_value();
        self
    }

    /// Add a close button that calls the application callback.
    pub fn on_close(mut self, close: impl Into<Callback<()>>) -> Self {
        let close: Callback<()> = close.into();
        self.close = Some(Rc::new(move || close.call(())));
        self
    }

    fn render(self) -> Node {
        let close = self.close.map(|close| {
            let label = kivact::i18n::message("soft.dialog.close", "Close");
            button("")
                .variant(ButtonVariant::Ghost)
                .name(label)
                .child(icon("x").style(style().size(16, 16)))
                .style(|| {
                    let p = kivact::style::use_tokens::<crate::Tokens>();
                    motion(style().size(32, 32).padding(0).radius(10), &p, 125)
                })
                .on_press(move || close())
        });
        let compensation = self
            .compensation
            .or_else(|| close.is_some().then_some(EdgeCompensation::All));
        let end = row((self.end, close)).style({
            let mut end = style().gap(8).align_items(Align::Center).shrink(0);
            if matches!(
                compensation,
                Some(EdgeCompensation::Block | EdgeCompensation::All)
            ) {
                end = end.margin_top(-8).margin_bottom(-8);
            }
            if matches!(
                compensation,
                Some(EdgeCompensation::Inline | EdgeCompensation::All)
            ) {
                end = end.margin_right(-8);
            }
            end
        });
        let title = column((self.title, self.subtitle)).style(
            style()
                .grow(1)
                .min_width(0)
                .margin_top(-6)
                .margin_bottom(-6),
        );
        let start = self
            .start
            .map(|start| row(start).style(style().gap(8).shrink(0)));
        row((start, title, end)).style(move || {
            let base = style().gap(12).align_items(Align::Start);
            if self.divider.get() {
                base.border_bottom(
                    1,
                    kivact::style::use_tokens::<crate::Tokens>().colors.border,
                )
            } else {
                base
            }
        })
    }
}

impl DrawerHeader {
    /// Add secondary text below the title.
    pub fn subtitle(mut self, subtitle: impl IntoText) -> Self {
        self.0 = self.0.subtitle(subtitle);
        self
    }

    /// Mount content once before the title.
    pub fn header_start(mut self, content: impl IntoView) -> Self {
        self.0 = self.0.header_start(content);
        self
    }

    /// Mount content once beside the trailing close control.
    pub fn header_end(mut self, content: impl IntoView) -> Self {
        self.0 = self.0.header_end(content);
        self
    }

    /// Override the trailing-control inset; a close control otherwise compensates both axes.
    pub fn header_end_compensation(mut self, compensation: EdgeCompensation) -> Self {
        self.0 = self.0.header_end_compensation(compensation);
        self
    }

    /// Enable the bottom divider from a read-only input; false by default.
    pub fn header_divider(mut self, divider: impl IntoValue<bool>) -> Self {
        self.0 = self.0.header_divider(divider);
        self
    }

    /// Add a close button that calls the application callback.
    pub fn on_close(mut self, close: impl Into<Callback<()>>) -> Self {
        let close: Callback<()> = close.into();
        self.0 = self.0.on_close(close);
        self
    }
}

impl IntoView for DialogHeader {
    fn into_view(self) -> View {
        self.render().into_view()
    }
}

impl IntoView for DrawerHeader {
    fn into_view(self) -> View {
        self.0.into_view()
    }
}
