//! Role: The Review panel.

use kivact::prelude::*;
use kivact_devtools_protocol as p;

use crate::state::{Inspected, Panel, Session};

/// The comment being edited, with its text as typed so far.
#[derive(Clone, Debug, PartialEq)]
enum CommentEdit {
    /// No comment is being changed.
    Idle,
    /// The comment `id` is being changed, to `draft`.
    Editing { id: String, draft: String },
}

/// The comments of the review and their export.
pub(crate) fn view(session: Session) -> impl IntoView {
    let comment = signal(String::new());
    let edit = signal(CommentEdit::Idle);
    let items = memo(move || {
        session
            .review
            .latest()
            .map(|review| review.items)
            .unwrap_or_default()
    });
    scroll(
        column((
            row((
                text("review.json and review.md are saved on every change."),
                button("Open folder").on_press(move || open_folder(session)),
            ))
            .style(style().gap(8).align_items(Align::Center)),
            text_area("Comment", comment),
            button("Add comment")
                .disabled(move || comment.get().trim().is_empty())
                .on_press(move || add_comment(session, comment)),
            comment_editor(session, edit),
            for_each(
                items,
                |item| item.id.clone(),
                move |item| item_view(session, item, edit),
            ),
        ))
        .style(style().gap(8).padding(12).align_items(Align::Start)),
    )
    .style(style().grow(1.0).min_height(0.0))
}

/// The editor shown while a comment is being changed.
fn comment_editor(session: Session, edit: Signal<CommentEdit>) -> impl IntoView {
    let draft = binding(
        move || match edit.get() {
            CommentEdit::Idle => String::new(),
            CommentEdit::Editing { draft, .. } => draft,
        },
        move |text: String| {
            edit.update(|edit| {
                if let CommentEdit::Editing { draft, .. } = edit {
                    *draft = text;
                }
            });
        },
    );
    show(
        move || matches!(edit.get(), CommentEdit::Editing { .. }),
        move || {
            row((
                text_area("Edit comment", draft),
                button("Save").on_press(move || save_comment(session, edit)),
                button("Cancel").on_press(move || edit.set(CommentEdit::Idle)),
            ))
            .style(style().gap(8).align_items(Align::Center))
        },
    )
}

/// One review item with its comments.
fn item_view(
    session: Session,
    item: Value<p::ReviewItem>,
    edit: Signal<CommentEdit>,
) -> impl IntoView {
    let node = item.with(|item| match &item.target {
        p::ReviewTarget::Node { node, .. } => Some(*node),
        p::ReviewTarget::Window {} => None,
    });
    let window = item.with(|item| item.window);
    let mut actions: Vec<View> = Vec::new();
    if let Some(node) = node {
        actions.push(
            button("Go to node")
                .on_press(move || go_to(session, window, node))
                .into_view(),
        );
        actions.push(
            button("Highlight")
                .disabled(move || item.with(|item| is_removed(session, item)))
                .on_press(move || session.highlight(Some(node), false))
                .into_view(),
        );
    }
    let comments = memo(move || item.with(|item| item.comments.clone()));
    column((
        item_row(move || item.with(item_title))
            .description_content(row((
                text(move || item.with(item_description)),
                show(
                    move || item.with(|item| is_removed(session, item)),
                    || text("(removed)"),
                ),
            )))
            .actions(row(actions)),
        for_each(
            comments,
            |comment| comment.id.clone(),
            move |comment| comment_view(session, comment, edit),
        ),
    ))
    .style(style().gap(2))
}

/// One comment with its edit and remove actions.
fn comment_view(
    session: Session,
    comment: Value<p::Comment>,
    edit: Signal<CommentEdit>,
) -> impl IntoView {
    let edit_id = comment.with(|comment| comment.id.clone());
    let remove_id = edit_id.clone();
    row((
        text(move || comment.with(|comment| format!("{}  ({})", comment.text, comment.created))),
        button("Edit").on_press(move || {
            let draft = comment.with(|comment| comment.text.clone());
            edit.set(CommentEdit::Editing {
                id: edit_id.clone(),
                draft,
            });
        }),
        button("Remove").on_press(move || remove_comment(session, remove_id.clone())),
    ))
    .style(style().gap(8).padding_left(24).align_items(Align::Center))
}

/// Select the node of an item and show its tree.
fn go_to(session: Session, window: p::WindowId, node: p::NodeId) {
    if session
        .inspected
        .update(|inspected| inspected.select_in(window, node))
    {
        session.load_window(window, Some(node));
    }
    session.panel.set(Panel::Elements);
}

/// Whether the item's node is gone from the inspected tree.
fn is_removed(session: Session, item: &p::ReviewItem) -> bool {
    let p::ReviewTarget::Node { node, .. } = &item.target else {
        return false;
    };
    session.inspected.with(|inspected| match inspected {
        Inspected::Loaded { window, tree, .. } => {
            *window == item.window && !tree.nodes.iter().any(|element| element.id == *node)
        }
        Inspected::Loading { .. } => false,
    })
}

/// The title of an item.
fn item_title(item: &p::ReviewItem) -> String {
    match &item.target {
        p::ReviewTarget::Node { role, name, .. } => {
            let role = role
                .map(|role| format!("{role:?}"))
                .unwrap_or_else(|| "element".to_owned());
            match name {
                Some(name) => format!("{} {role} \"{name}\"", item.id),
                None => format!("{} {role}", item.id),
            }
        }
        p::ReviewTarget::Window {} => format!("{} window \"{}\"", item.id, item.window_title),
    }
}

/// The description of an item.
fn item_description(item: &p::ReviewItem) -> String {
    let style_changes = item.style_changes.len();
    let comments = item.comments.len();
    format!(
        "window \"{}\" · {style_changes} style change(s) · {comments} comment(s)",
        item.window_title
    )
}

/// Add the text of the draft to the selected node, or to the window.
fn add_comment(session: Session, draft: Signal<String>) {
    let text = draft.get_untracked();
    if text.trim().is_empty() {
        return;
    }
    session.send::<p::AddComment>(
        p::AddCommentParams {
            window: session.window(),
            node: session.inspected.with(Inspected::selected),
            text,
            screenshot: true,
        },
        move |_| {
            draft.set(String::new());
            session.review.refetch();
        },
    );
}

/// Change the text of a comment.
fn save_comment(session: Session, edit: Signal<CommentEdit>) {
    let (id, draft) = match edit.get_untracked() {
        CommentEdit::Idle => return,
        CommentEdit::Editing { id, draft } => (id, draft),
    };
    session.send::<p::EditComment>(
        p::EditCommentParams {
            comment: id,
            text: draft,
        },
        move |_| {
            edit.set(CommentEdit::Idle);
            session.review.refetch();
        },
    );
}

/// Remove a comment.
fn remove_comment(session: Session, comment: String) {
    session.send::<p::RemoveComment>(p::RemoveCommentParams { comment }, move |_| {
        session.review.refetch();
    });
}

/// Write the review and reveal the JSON file in the file manager.
fn open_folder(session: Session) {
    session.send::<p::ExportReview>(p::Empty {}, move |exported| {
        spawn(kivact::platform::reveal_path(exported.json));
    });
}
