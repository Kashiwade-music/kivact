//! Role: Compose Soft bulk-selection and status presentation on canonical controls.
use kivact::state::{Binding, IntoBinding, IntoText, IntoValue, PropValue, Value};
use kivact::style::{Align, Direction, style};
use kivact::view::{IntoView, Node, NodeBuilder, View, text};

type SelectionLabel = kivact::view::ViewFn<Value<usize>>;

pub struct TableSelectionToolbar<K: PropValue> {
    root: kivact::ui::Toolbar,
    selection: Binding<Vec<K>>,
    clear_label: Value<String>,
    selection_label: Option<SelectionLabel>,
    actions: Option<View>,
}

pub fn table_selection_toolbar<K: PropValue>(
    selection: impl IntoBinding<Vec<K>>,
) -> TableSelectionToolbar<K> {
    TableSelectionToolbar {
        root: kivact::ui::toolbar("Bulk actions"),
        selection: selection.into_binding(),
        clear_label: "Unselect All".into_text_value(),
        selection_label: None,
        actions: None,
    }
}

impl<K: PropValue> NodeBuilder for TableSelectionToolbar<K> {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        self.root.node_mut()
    }
}

impl<K: PropValue> TableSelectionToolbar<K> {
    pub fn clear_label(mut self, label: impl IntoText) -> Self {
        self.clear_label = label.into_text_value();
        self
    }
    pub fn actions(mut self, actions: impl IntoView) -> Self {
        self.actions = Some(actions.into_view());
        self
    }
    pub fn selection_label<V: IntoView>(
        mut self,
        label: impl Fn(Value<usize>) -> V + 'static,
    ) -> Self {
        self.selection_label = Some(label.into());
        self
    }
}

impl<K: PropValue> IntoView for TableSelectionToolbar<K> {
    fn into_view(self) -> View {
        let Self {
            root,
            selection,
            clear_label,
            selection_label,
            actions,
        } = self;
        let count = kivact::state::memo(move || selection.with(Vec::len));
        let count_label = match selection_label {
            Some(render) => render.call(count),
            None => text(move || format!("{} selected", count.get())).into_view(),
        };
        let clear = kivact::ui::button(clear_label)
            .variant(kivact::ui::ButtonVariant::Ghost)
            .size(kivact::ui::ControlSize::Small)
            .on_press(move || selection.set(Vec::new()));
        root.hidden(move || count.get() == 0)
            .with_node(move |node| {
                node.base_style(move || {
                    let p = kivact::style::use_tokens::<crate::Tokens>();
                    style()
                        .direction(Direction::Row)
                        .align_items(Align::Center)
                        .padding_x(8)
                        .padding_y(4)
                        .gap(8)
                        .radius(p.radius.element)
                        .background(p.colors.bg_muted)
                        .font_size(14)
                        .color(p.colors.text_primary)
                })
            })
            .start((count_label, clear))
            .end((actions,))
            .into_view()
    }
}

/// The semantic colors of a table's status marker.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RowStatusTone {
    #[default]
    Neutral,
    Success,
    Warning,
    Error,
}
kivact::state::constant_value!(RowStatusTone);

/// Dot treatment of a canonical badge used in a table's status lane.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RowStatusDot;
impl kivact::ui::Variant for RowStatusDot {}

/// Show a named semantic status marker without adding another status owner.
pub fn table_row_status(
    label: impl IntoText,
    tone: impl IntoValue<RowStatusTone>,
) -> kivact::ui::Badge {
    let tone = tone.into_value();
    kivact::ui::badge("")
        .tone(move || match tone.get() {
            RowStatusTone::Neutral => kivact::interaction::Tone::Neutral,
            RowStatusTone::Success => kivact::interaction::Tone::Success,
            RowStatusTone::Warning => kivact::interaction::Tone::Warning,
            RowStatusTone::Error => kivact::interaction::Tone::Danger,
        })
        .variant(RowStatusDot)
        .name(label)
        .with_node(|node| node.role(kivact::view::Role::Image))
}
