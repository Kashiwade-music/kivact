//! Role: Expose application-owned voice dictation controls alongside canonical composers.
use kivact::i18n::message;
use kivact::state::{Callback, IntoText, IntoValue, Value, callback};
use kivact::style::{Align, Direction, Justify, style};
use kivact::ui::ControlSize;
use kivact::view::{IntoView, Node, NodeBuilder, Role, View, node, row, text};
fn control_side(size: ControlSize) -> i32 {
    match size {
        ControlSize::ExtraSmall => 24,
        ControlSize::Small => 28,
        ControlSize::Medium => 36,
        ControlSize::Large => 44,
    }
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DictationState {
    pub supported: bool,
    pub listening: bool,
    pub volume: f64,
    pub bands: Vec<f64>,
}

pub struct ChatDictationButton {
    root: Node,
    state: Value<DictationState>,
    on_toggle: Option<Callback<()>>,
    size: Value<ControlSize>,
    hide_when_unsupported: Value<bool>,
    label: Value<String>,
}

pub fn chat_dictation_button(state: impl IntoValue<DictationState>) -> ChatDictationButton {
    ChatDictationButton {
        root: node(),
        state: state.into_value(),
        on_toggle: None,
        size: (ControlSize::Medium).into_value(),
        hide_when_unsupported: (true).into_value(),
        label: message("soft.chat_dictation_button.label", "Dictation"),
    }
}

impl NodeBuilder for ChatDictationButton {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl ChatDictationButton {
    pub fn on_toggle(mut self, toggle: impl Into<Callback<()>>) -> Self {
        let toggle: Callback<()> = toggle.into();
        self.on_toggle = Some(callback(move |_| toggle.call(())));
        self
    }
    pub fn size(mut self, size: impl IntoValue<ControlSize>) -> Self {
        self.size = size.into_value();
        self
    }
    pub fn hide_when_unsupported(mut self, hide: impl IntoValue<bool>) -> Self {
        self.hide_when_unsupported = hide.into_value();
        self
    }
    pub fn label(mut self, label: impl IntoText) -> Self {
        self.label = label.into_text_value();
        self
    }
}

impl IntoView for ChatDictationButton {
    fn into_view(self) -> View {
        let Self {
            root,
            state,
            on_toggle,
            size,
            hide_when_unsupported,
            label,
        } = self;
        let side = control_side(size.get());
        let listening = move || state.with(|state| state.listening);
        let icon = text("🎙").hidden(listening);
        let waveform = row(dictation_bars(state, side))
            .hidden(move || !listening())
            .style(style().gap(2).align_items(Align::Center));
        root.with_node(move |node| node.role(Role::Button))
            .focusable(true)
            .name(label)
            .checked(listening)
            .disabled(move || !state.with(|state| state.supported))
            .on_press(move || {
                if let Some(toggle) = on_toggle {
                    toggle.call(());
                }
            })
            .hidden(move || hide_when_unsupported.get() && !state.with(|state| state.supported))
            .with_node(move |node| {
                node.base_style(move || {
                    let p = kivact::style::use_tokens::<crate::Tokens>();
                    style()
                        .direction(Direction::Row)
                        .size(side, side)
                        .radius(side as f64 / 2.0)
                        .align_items(Align::Center)
                        .justify_content(Justify::Center)
                        .color(p.colors.text_primary)
                        .hover(style().background(p.colors.overlay_hover))
                        .focus_visible(style().ring(p.sizes.focus_ring_width, p.colors.focus_ring))
                })
            })
            .child((icon, waveform))
            .into_view()
    }
}

fn dictation_bars(state: Value<DictationState>, side: i32) -> Vec<Node> {
    (0..5)
        .map(|index| {
            node().style(move || {
                let p = kivact::style::use_tokens::<crate::Tokens>();
                let (band, volume) = state
                    .with(|state| (state.bands.get(index).copied().unwrap_or(0.0), state.volume));
                let level = (band.max(0.0) / 0.2).sqrt().min(1.0);
                let height = if side <= 28 { 14.0 } else { 18.0 };
                style()
                    .size(2.5, height * (0.08 + level * 0.92))
                    .radius(2)
                    .background(if volume >= 0.3 {
                        p.colors.error
                    } else {
                        p.colors.accent
                    })
            })
        })
        .collect::<Vec<_>>()
}
