//! Role: Accessibility attribute conversion.
//!
//! Converts validated core semantics to AccessKit and AccessKit's action
//! requests back; AccessKit types stay inside this module and `native`.
//!
//! What a node becomes:
//! - Its role is AccessKit's role of the same meaning: `AlertDialog` as
//!   `AlertDialog`, the header, body and footer row groups of a table as
//!   `RowGroup`, a group of navigation links (breadcrumbs, pagination) as
//!   the `Navigation` landmark. A generic root stands for the window
//!   (`Window`): platforms filter a generic root out of their trees.
//! - Its scroll and transform: its own coordinate space (its descendants')
//!   moves back by its scroll and is then mapped by its transform, which
//!   become AccessKit's node transform; its own bounds are moved forward by
//!   the scroll, so they stay where it is laid out
//!   (`kivact_types::render::SemanticsTree::window_bounds` composes the same way).
//! - Its relations (`described_by`, `controls`, `labelled_by`, in order)
//!   and `active_descendant` become AccessKit's relations of those names.
//! - Its `level`, `set_size` and `pos_in_set` become `set_level`,
//!   `set_size_of_set` and `set_position_in_set` as they are (1-based, as
//!   ARIA's; hosts leave a 0 level or position out), and `multiselectable`
//!   is set only for `Some(true)`.
//! - `checked` is the toggle state (`Indeterminate` is `Toggled::Mixed`); the
//!   label of a text node is also its value (AccessKit names a label from
//!   its value); `Dismiss` is a custom action.
//! - A text's runs become `TextRun` children ([`text_run`]): a CR LF pair
//!   is one character on the platform (one selection unit, its two
//!   scalars one position, a caret between them placed before the pair),
//!   and a line break has no width. A run's direction is guessed from
//!   the carets at its two ends (right to left when the last is left of
//!   the first), so a run of mixed bidi text is not split by direction;
//!   no word boundaries (AccessKit's word starts) are given.

use kivact_types::render::SemanticsNode;
use kivact_types::{A11yAction, CheckedState, CurrentItem, PopupKind, Role, SemanticsId};

/// The id of the one custom action, `Dismiss`.
const DISMISS: i32 = 1;

/// The AccessKit action a node declares for `action`. AccessKit has a
/// single default action, so `Activate`, `Toggle` and `Select` all declare
/// `Click`.
fn platform_action(action: A11yAction) -> accesskit::Action {
    match action {
        A11yAction::Activate | A11yAction::Toggle | A11yAction::Select => accesskit::Action::Click,
        A11yAction::Focus => accesskit::Action::Focus,
        A11yAction::Increment => accesskit::Action::Increment,
        A11yAction::Decrement => accesskit::Action::Decrement,
        A11yAction::Expand => accesskit::Action::Expand,
        A11yAction::Collapse => accesskit::Action::Collapse,
        A11yAction::ScrollIntoView => accesskit::Action::ScrollIntoView,
        A11yAction::SetValue => accesskit::Action::SetValue,
        A11yAction::ShowMenu => accesskit::Action::ShowContextMenu,
        A11yAction::Dismiss => accesskit::Action::CustomAction,
    }
}

/// The action an AccessKit request asks for: the inverse of
/// [`platform_action`], a click being `Activate` (the receiver resolves it
/// against the actions the node declared). `None`: no Arvane action.
pub(super) fn action(request: &accesskit::ActionRequest) -> Option<A11yAction> {
    use accesskit::Action;
    Some(match request.action {
        Action::Click => A11yAction::Activate,
        Action::Focus => A11yAction::Focus,
        Action::Increment => A11yAction::Increment,
        Action::Decrement => A11yAction::Decrement,
        Action::Expand => A11yAction::Expand,
        Action::Collapse => A11yAction::Collapse,
        Action::ScrollIntoView => A11yAction::ScrollIntoView,
        Action::SetValue => A11yAction::SetValue,
        Action::ShowContextMenu => A11yAction::ShowMenu,
        Action::CustomAction
            if matches!(
                request.data,
                Some(accesskit::ActionData::CustomAction(DISMISS))
            ) =>
        {
            A11yAction::Dismiss
        }
        _ => return None,
    })
}

/// An AccessKit update carrying `nodes` of the tree rooted at `root`,
/// with all their text runs when `runs` (else none: the caller adds the
/// runs that changed with [`text_run`]).
pub(super) fn update<'a>(
    root: SemanticsId,
    focus: Option<SemanticsId>,
    nodes: impl Iterator<Item = &'a SemanticsNode>,
    runs: bool,
) -> accesskit::TreeUpdate {
    let nodes = nodes
        .flat_map(|source| {
            let mut nodes = vec![(accesskit::NodeId(source.id.0), node(source, root))];
            if let Some(text) = source.semantics.text.as_ref().filter(|_| runs) {
                nodes.extend(text.runs.iter().map(text_run));
            }
            nodes
        })
        .collect();
    let mut metadata = accesskit::Tree::new(accesskit::NodeId(root.0));
    metadata.toolkit_name = Some("Arvane".into());
    metadata.toolkit_version = Some(env!("CARGO_PKG_VERSION").into());
    accesskit::TreeUpdate {
        nodes,
        tree: Some(metadata),
        tree_id: accesskit::TreeId::ROOT,
        focus: accesskit::NodeId(focus.unwrap_or(root).0),
    }
}

/// One node. A scrolling node moves its own coordinate space (and so its
/// descendants') back by its offset; its own bounds are moved forward by
/// it, so they stay where it is laid out.
/// A transformed node maps its coordinate space (its bounds included) by
/// its transform after that.
fn node(source: &SemanticsNode, root: SemanticsId) -> accesskit::Node {
    // A generic root is filtered out of platform trees unless focused,
    // which detaches its children from sibling navigation; the root
    // stands for the window.
    let mut node = accesskit::Node::new(
        if source.id == root && source.semantics.role.unwrap_or(Role::Generic) == Role::Generic {
            accesskit::Role::Window
        } else {
            role(source.semantics.role.unwrap_or(Role::Generic))
        },
    );
    node.set_children(
        source
            .children
            .iter()
            .map(|id| accesskit::NodeId(id.0))
            .collect::<Vec<_>>(),
    );
    if let Some(text) = source.semantics.text.as_ref() {
        let children: Vec<_> = source
            .children
            .iter()
            .map(|id| accesskit::NodeId(id.0))
            .chain(text.runs.iter().map(|run| accesskit::NodeId(run.id.0)))
            .collect();
        node.set_children(children);
        let [anchor, focus] = text_positions(text, [text.selection.0, text.selection.1]);
        node.set_text_selection(accesskit::TextSelection { anchor, focus });
    }
    set_labels(&mut node, source);
    let (sx, sy) = (source.scroll.x, source.scroll.y);
    // Its descendants moved back by the scroll, then everything mapped by
    // its own transform (`SemanticsTree::window_bounds`).
    let t = source.transform;
    if sx != 0.0 || sy != 0.0 || !t.is_identity() {
        let own = accesskit::Affine::new([t.a, t.b, t.c, t.d, t.e, t.f].map(f64::from));
        node.set_transform(own * accesskit::Affine::translate((-sx, -sy)));
    }
    node.set_bounds(accesskit::Rect {
        x0: source.bounds.x + sx,
        y0: source.bounds.y + sy,
        x1: source.bounds.x + source.bounds.width + sx,
        y1: source.bounds.y + source.bounds.height + sy,
    });
    set_states(&mut node, source);
    set_relations(&mut node, source);
    set_collection_properties(&mut node, source);
    for action in source.semantics.actions.iter().flatten() {
        node.add_action(platform_action(*action));
    }
    if source
        .semantics
        .actions
        .as_ref()
        .is_some_and(|actions| actions.contains(&A11yAction::Dismiss))
    {
        node.set_custom_actions(vec![accesskit::CustomAction {
            id: DISMISS,
            description: "Dismiss".into(),
        }]);
    }
    node
}

/// Convert one group of canonical semantics properties.
fn set_labels(node: &mut accesskit::Node, source: &SemanticsNode) {
    if !source
        .semantics
        .name
        .as_deref()
        .unwrap_or_default()
        .is_empty()
    {
        node.set_label(source.semantics.name.clone().unwrap_or_default());
    }
    if !source
        .semantics
        .description
        .as_deref()
        .unwrap_or_default()
        .is_empty()
    {
        node.set_description(source.semantics.description.clone().unwrap_or_default());
    }
    // AccessKit names a `Label` from its value, so text carries its
    // label there for platform names (UIA Name, AT-SPI name).
    if source.semantics.role.unwrap_or(Role::Generic) == Role::Text
        && !source
            .semantics
            .name
            .as_deref()
            .unwrap_or_default()
            .is_empty()
    {
        node.set_value(source.semantics.name.clone().unwrap_or_default());
    } else if let Some(value) = &source.semantics.value_text {
        node.set_value(value.clone());
    }
    if let Some(id) = &source.semantics.test_id {
        node.set_author_id(id.clone());
    }
}

/// Convert one group of canonical semantics properties.
fn set_states(node: &mut accesskit::Node, source: &SemanticsNode) {
    if source.semantics.disabled {
        node.set_disabled();
    }
    if source.semantics.hidden {
        node.set_hidden();
    }
    if source.semantics.focusable {
        node.add_action(accesskit::Action::Focus);
    }
    match source.semantics.checked {
        Some(CheckedState::Indeterminate) => node.set_toggled(accesskit::Toggled::Mixed),
        Some(CheckedState::Checked) => node.set_toggled(accesskit::Toggled::True),
        Some(CheckedState::Unchecked) => node.set_toggled(accesskit::Toggled::False),
        None => {}
    }
    if source.semantics.selected.unwrap_or(false)
        || matches!(
            source.semantics.role.unwrap_or(Role::Generic),
            Role::Option | Role::ListItem | Role::Tab | Role::TreeItem
        )
    {
        node.set_selected(source.semantics.selected.unwrap_or(false));
    }
    if let Some(expanded) = source.semantics.expanded {
        node.set_expanded(expanded);
    }
    if source.semantics.role.unwrap_or(Role::Generic) == Role::Alert {
        node.set_live(accesskit::Live::Assertive);
    }
    if source.semantics.role.unwrap_or(Role::Generic) == Role::Status {
        node.set_live(accesskit::Live::Polite);
    }
}

/// Convert one group of canonical semantics properties.
fn set_relations(node: &mut accesskit::Node, source: &SemanticsNode) {
    let props = &source.semantics;
    if let Some(v) = props.value_min {
        node.set_min_numeric_value(v.get());
    }
    if let Some(v) = props.value_max {
        node.set_max_numeric_value(v.get());
    }
    if let Some(v) = props.value_now {
        node.set_numeric_value(v.get());
    }
    if let Some(live) = props.live_region {
        node.set_live(match live {
            kivact_types::LiveRegion::Off => accesskit::Live::Off,
            kivact_types::LiveRegion::Polite => accesskit::Live::Polite,
            kivact_types::LiveRegion::Assertive => accesskit::Live::Assertive,
        });
    }
    match props.sort {
        kivact_types::SortDirection::None => {}
        kivact_types::SortDirection::Ascending => {
            node.set_sort_direction(accesskit::SortDirection::Ascending)
        }
        kivact_types::SortDirection::Descending => {
            node.set_sort_direction(accesskit::SortDirection::Descending)
        }
        kivact_types::SortDirection::Other => {
            node.set_sort_direction(accesskit::SortDirection::Other)
        }
    }
    match props.orientation {
        kivact_types::Orientation::Unspecified => {}
        kivact_types::Orientation::Horizontal => {
            node.set_orientation(accesskit::Orientation::Horizontal)
        }
        kivact_types::Orientation::Vertical => {
            node.set_orientation(accesskit::Orientation::Vertical)
        }
    }
    if let Some(target) = props.active_descendant {
        node.set_active_descendant(accesskit::NodeId(target.0));
    }
    if !props.described_by.is_empty() {
        node.set_described_by(accesskit_ids(&props.described_by));
    }
    if !props.controls.is_empty() {
        node.set_controls(accesskit_ids(&props.controls));
    }
    if !props.labelled_by.is_empty() {
        node.set_labelled_by(accesskit_ids(&props.labelled_by));
    }
}

/// Convert one group of canonical semantics properties.
fn set_collection_properties(node: &mut accesskit::Node, source: &SemanticsNode) {
    let props = &source.semantics;
    if let Some(count) = props.row_count {
        node.set_row_count(count as usize);
    }
    if let Some(count) = props.column_count {
        node.set_column_count(count as usize);
    }
    // Canonical semantics follows ARIA's one-based indices; AccessKit uses zero-based indices.
    if let Some(index) = props.row_index.and_then(|index| index.checked_sub(1)) {
        node.set_row_index(index as usize);
    }
    if let Some(span) = props.column_span.filter(|&span| span != 0) {
        node.set_column_span(span as usize);
    }
    if let Some(index) = props.column_index.and_then(|index| index.checked_sub(1)) {
        node.set_column_index(index as usize);
    }
    if let Some(level) = props.level {
        node.set_level(level as usize);
    }
    if let Some(size) = props.set_size {
        node.set_size_of_set(size as usize);
    }
    if let Some(position) = props.pos_in_set {
        node.set_position_in_set(position as usize);
    }
    if props.multiselectable == Some(true) {
        node.set_multiselectable();
    }
    if props.busy == Some(true) {
        node.set_busy();
    }
    if props.required == Some(true) {
        node.set_required();
    }
    if let Some(popup) = props.has_popup {
        node.set_has_popup(match popup {
            PopupKind::Menu => accesskit::HasPopup::Menu,
            PopupKind::ListBox => accesskit::HasPopup::Listbox,
            PopupKind::Tree => accesskit::HasPopup::Tree,
            PopupKind::Grid => accesskit::HasPopup::Grid,
            PopupKind::Dialog => accesskit::HasPopup::Dialog,
        });
    }
    if let Some(current) = props.current_item {
        node.set_aria_current(match current {
            CurrentItem::True => accesskit::AriaCurrent::True,
            CurrentItem::Page => accesskit::AriaCurrent::Page,
            CurrentItem::Step => accesskit::AriaCurrent::Step,
            CurrentItem::Location => accesskit::AriaCurrent::Location,
            CurrentItem::Date => accesskit::AriaCurrent::Date,
            CurrentItem::Time => accesskit::AriaCurrent::Time,
        });
    }
    if props.invalid == Some(true) {
        node.set_invalid(accesskit::Invalid::True);
    }
    if props.read_only == Some(true) {
        node.set_read_only();
    }
    if props.protected {
        node.set_role(accesskit::Role::PasswordInput);
    }
}

fn accesskit_ids(ids: &[SemanticsId]) -> Vec<accesskit::NodeId> {
    ids.iter().map(|id| accesskit::NodeId(id.0)).collect()
}

fn role(source: Role) -> accesskit::Role {
    match source {
        Role::Generic => accesskit::Role::GenericContainer,
        Role::Group => accesskit::Role::Group,
        Role::Text => accesskit::Role::Label,
        Role::Heading => accesskit::Role::Heading,
        Role::Button => accesskit::Role::Button,
        Role::CheckBox => accesskit::Role::CheckBox,
        Role::ComboBox => accesskit::Role::ComboBox,
        Role::ListBox => accesskit::Role::ListBox,
        Role::ListItem => accesskit::Role::ListItem,
        Role::Option => accesskit::Role::ListBoxOption,
        Role::TextBox => accesskit::Role::TextInput,
        Role::Image => accesskit::Role::Image,
        Role::Slider => accesskit::Role::Slider,
        Role::ProgressBar => accesskit::Role::ProgressIndicator,
        Role::Alert => accesskit::Role::Alert,
        Role::Status => accesskit::Role::Status,
        Role::List => accesskit::Role::List,
        Role::Dialog => accesskit::Role::Dialog,
        Role::AlertDialog => accesskit::Role::AlertDialog,
        Role::Menu => accesskit::Role::Menu,
        Role::MenuItem => accesskit::Role::MenuItem,
        Role::Tab => accesskit::Role::Tab,
        Role::TabList => accesskit::Role::TabList,
        Role::ScrollView => accesskit::Role::ScrollView,
        Role::Canvas => accesskit::Role::Canvas,
        Role::Link => accesskit::Role::Link,
        Role::Toolbar => accesskit::Role::Toolbar,
        Role::Tooltip => accesskit::Role::Tooltip,
        Role::RadioGroup => accesskit::Role::RadioGroup,
        Role::RadioButton => accesskit::Role::RadioButton,
        Role::Switch => accesskit::Role::Switch,
        Role::TabPanel => accesskit::Role::TabPanel,
        Role::Splitter => accesskit::Role::Splitter,
        Role::Table => accesskit::Role::Table,
        Role::Row => accesskit::Role::Row,
        Role::Cell => accesskit::Role::Cell,
        Role::ColumnHeader => accesskit::Role::ColumnHeader,
        Role::Tree => accesskit::Role::Tree,
        Role::TreeItem => accesskit::Role::TreeItem,
        Role::Meter => accesskit::Role::Meter,
        Role::SpinButton => accesskit::Role::SpinButton,
        Role::MenuBar => accesskit::Role::MenuBar,
        Role::MenuItemCheckBox => accesskit::Role::MenuItemCheckBox,
        Role::MenuItemRadio => accesskit::Role::MenuItemRadio,
        Role::Separator => accesskit::Role::Splitter,
        Role::Grid => accesskit::Role::Grid,
        Role::GridCell => accesskit::Role::Cell,
        Role::RowGroup => accesskit::Role::RowGroup,
        Role::Navigation => accesskit::Role::Navigation,
        Role::Log => accesskit::Role::Log,
        Role::Article => accesskit::Role::Article,
        Role::Main => accesskit::Role::Main,
        Role::Region => accesskit::Role::Region,
    }
}

/// The platform positions of the scalar offsets `offsets` of `text` (of
/// the complete text, in its runs), in one pass over its runs (each run
/// starts where the one before ends, the first at the text's start).
fn text_positions<const N: usize>(
    text: &kivact_types::SemanticsText,
    offsets: [u32; N],
) -> [accesskit::TextPosition; N] {
    let first = text
        .runs
        .get(0)
        .expect("invariant: validated text has a run");
    let offsets = offsets.map(|offset| offset.saturating_sub(text.start));
    let mut found = [(first, 0u32); N];
    let mut start = 0u32;
    for run in text.runs.iter() {
        if offsets.iter().all(|offset| start > *offset) {
            break;
        }
        for (slot, offset) in found.iter_mut().zip(offsets) {
            if start <= offset {
                *slot = (run, start);
            }
        }
        start += run.value.chars().count() as u32;
    }
    std::array::from_fn(|i| {
        let (run, start) = found[i];
        let local = offsets[i] - start;
        let units = text_units(&run.value);
        let character_index = units
            .iter()
            .take_while(|(start, _, _, _)| *start < local)
            .count();
        let character_index = units
            .iter()
            .position(|(start, count, _, _)| *count == 2 && local == start + 1)
            .unwrap_or(character_index);
        accesskit::TextPosition {
            node: accesskit::NodeId(run.id.0),
            character_index,
        }
    })
}

fn text_units(value: &str) -> Vec<(u32, u32, u8, bool)> {
    let mut chars = value.chars().peekable();
    let mut scalar = 0;
    let mut units = Vec::new();
    while let Some(c) = chars.next() {
        let crlf = c == '\r' && chars.peek() == Some(&'\n');
        if crlf {
            chars.next();
        }
        let count = if crlf { 2 } else { 1 };
        units.push((
            scalar,
            count,
            if crlf { 2 } else { c.len_utf8() as u8 },
            c == '\r' || c == '\n',
        ));
        scalar += count;
    }
    units
}

/// The platform node of a text run.
pub(super) fn text_run(
    run: &kivact_types::SemanticsTextRun,
) -> (accesskit::NodeId, accesskit::Node) {
    let mut node = accesskit::Node::new(accesskit::Role::TextRun);
    node.set_value(run.value.clone());
    node.set_bounds(accesskit::Rect {
        x0: run.bounds.x,
        y0: run.bounds.y,
        x1: run.bounds.right(),
        y1: run.bounds.bottom(),
    });
    let first = run.carets.iter().min_by_key(|c| c.offset);
    let last = run.carets.iter().max_by_key(|c| c.offset);
    let direction = match (first, last) {
        (Some(first), Some(last)) if last.rect.x < first.rect.x => {
            accesskit::TextDirection::RightToLeft
        }
        _ => accesskit::TextDirection::LeftToRight,
    };
    node.set_text_direction(direction);
    let units = text_units(&run.value);
    node.set_character_lengths(
        units
            .iter()
            .map(|(_, _, bytes, _)| *bytes)
            .collect::<Vec<_>>(),
    );
    let mut caret_positions = std::collections::HashMap::new();
    for caret in &run.carets {
        caret_positions.entry(caret.offset).or_insert(caret.rect.x);
    }
    let at = |offset| caret_positions.get(&offset).copied();
    let fallback = |offset| {
        if offset == 0 {
            run.bounds.x
        } else {
            run.bounds.right()
        }
    };
    node.set_character_positions(
        units
            .iter()
            .map(|(start, _, _, _)| match direction {
                accesskit::TextDirection::RightToLeft => {
                    run.bounds.right() - at(*start).unwrap_or_else(|| fallback(*start))
                }
                _ => at(*start).unwrap_or_else(|| fallback(*start)) - run.bounds.x,
            })
            .map(|position| position as f32)
            .collect::<Vec<_>>(),
    );
    node.set_character_widths(
        units
            .iter()
            .map(|(start, count, _, newline)| {
                if *newline {
                    return 0.0;
                }
                match (at(*start), at(start + count)) {
                    (Some(start), Some(end)) => (end - start).abs(),
                    _ => 0.0,
                }
            })
            .map(|width| width as f32)
            .collect::<Vec<_>>(),
    );
    (accesskit::NodeId(run.id.0), node)
}
