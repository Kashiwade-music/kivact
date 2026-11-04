//! Role: Lay out Soft dialog and drawer headings.
use crate::recipes_dialog::motion;
use crate::theme_access::tokens;
use kivact::state::Callback;
use kivact::state::IntoText;
use kivact::state::IntoValue;
use kivact::style::Align;
use kivact::style::style;
use kivact::ui::ButtonVariant;
use kivact::ui::button;
use kivact::view::IntoView;
use kivact::view::Node;
use kivact::view::Role;
use kivact::view::View;
use kivact::view::column;
use kivact::view::icon;
use kivact::view::row;
use kivact::view::text;
use std::rc::Rc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EdgeCompensation {
    Inline,
    Block,
    All,
}

pub struct DialogHeader {
    title: View,
    subtitle: Option<View>,
    start: Option<View>,
    end: Option<View>,
    close: Option<Rc<dyn Fn()>>,
    compensation: Option<EdgeCompensation>,
    divider: kivact::state::Value<bool>,
}

pub fn dialog_header(title: impl IntoText) -> DialogHeader {
    DialogHeader {
        title: text(title)
            .role(Role::Heading)
            .style(|| {
                let p = tokens();
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

pub struct DrawerHeader(DialogHeader);

impl DialogHeader {
    pub fn subtitle(mut self, subtitle: impl IntoText) -> Self {
        self.subtitle = Some(
            text(subtitle)
                .style(|| {
                    style()
                        .font_size(12)
                        .line_height(20.0 / 14.0)
                        .color(tokens().colors.text_secondary)
                })
                .into_view(),
        );
        self
    }

    pub fn header_start(mut self, content: impl IntoView) -> Self {
        self.start = Some(content.into_view());
        self
    }

    pub fn header_end(mut self, content: impl IntoView) -> Self {
        self.end = Some(content.into_view());
        self
    }

    pub fn header_end_compensation(mut self, compensation: EdgeCompensation) -> Self {
        self.compensation = Some(compensation);
        self
    }

    pub fn header_divider(mut self, divider: impl IntoValue<bool>) -> Self {
        self.divider = divider.into_value();
        self
    }

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
                    let p = tokens();
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
                base.border_bottom(1, tokens().colors.border)
            } else {
                base
            }
        })
    }
}

impl DrawerHeader {
    pub fn subtitle(mut self, subtitle: impl IntoText) -> Self {
        self.0 = self.0.subtitle(subtitle);
        self
    }

    pub fn header_start(mut self, content: impl IntoView) -> Self {
        self.0 = self.0.header_start(content);
        self
    }

    pub fn header_end(mut self, content: impl IntoView) -> Self {
        self.0 = self.0.header_end(content);
        self
    }

    pub fn header_end_compensation(mut self, compensation: EdgeCompensation) -> Self {
        self.0 = self.0.header_end_compensation(compensation);
        self
    }

    pub fn header_divider(mut self, divider: impl IntoValue<bool>) -> Self {
        self.0 = self.0.header_divider(divider);
        self
    }

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
