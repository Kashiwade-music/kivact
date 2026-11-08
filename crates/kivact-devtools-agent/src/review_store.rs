//! Role: The review of one session: items, comments, images and saved files.

use std::collections::BTreeMap;
use std::path::PathBuf;

use kivact_devtools_protocol::{
    AppInfo, Comment, Emulation, ExportedReview, PseudoLocale, ReviewDocument, ReviewItem,
    ReviewTarget, StyleChange, WindowId, rfc3339_utc, to_markdown,
};

use crate::files::write_atomic;

/// The schema of the review document.
const SCHEMA: &str = "kivact-devtools-review/1";

/// Everything the core knows when it adds one comment.
pub struct NewComment {
    /// The window the comment points at.
    pub window: WindowId,
    /// The title of that window when the comment was added.
    pub window_title: String,
    /// The element (or window) the comment points at.
    pub target: ReviewTarget,
    /// The style overrides of the element when the comment was added.
    pub style_changes: Vec<StyleChange>,
    /// A screenshot to store for the item.
    pub image_png: Option<Vec<u8>>,
    /// The text of the comment.
    pub text: String,
}

/// The review of this session, saved on every change.
pub struct ReviewStore {
    dir: PathBuf,
    document: ReviewDocument,
    next_item: u64,
    next_comment: BTreeMap<String, u64>,
}

impl ReviewStore {
    pub(crate) fn new(dir: PathBuf, app: AppInfo, session: String, now_ms: u64) -> ReviewStore {
        let created = rfc3339_utc(now_ms);
        ReviewStore {
            dir,
            document: ReviewDocument {
                schema: SCHEMA.to_owned(),
                app,
                session,
                created: created.clone(),
                updated: created,
                environment: Emulation {
                    theme: None,
                    reduced_motion: None,
                    locale: None,
                    pseudo_locale: PseudoLocale::Off,
                    scale: None,
                    touch: None,
                },
                items: Vec::new(),
            },
            next_item: 1,
            next_comment: BTreeMap::new(),
        }
    }

    pub fn document(&self) -> &ReviewDocument {
        &self.document
    }

    /// Add a comment to the item of the same target in the same window (a new item when none),
    /// save, and return the item.
    pub fn add(&mut self, comment: NewComment, now_ms: u64) -> std::io::Result<ReviewItem> {
        let NewComment {
            window,
            window_title,
            target,
            style_changes,
            image_png,
            text,
        } = comment;
        let index = self
            .document
            .items
            .iter()
            .position(|item| same_target(item, window, &target));
        let item = match index {
            Some(index) => &mut self.document.items[index],
            None => {
                let id = format!("C{}", self.next_item);
                self.next_item += 1;
                self.document.items.push(ReviewItem {
                    id,
                    window_title,
                    window,
                    target,
                    style_changes: Vec::new(),
                    image: None,
                    comments: Vec::new(),
                });
                self.document
                    .items
                    .last_mut()
                    .expect("invariant: the item was just pushed")
            }
        };
        let number = self.next_comment.entry(item.id.clone()).or_insert(0);
        *number += 1;
        let comment_id = format!("{}.{}", item.id, number);
        item.style_changes = style_changes;
        if let Some(png) = image_png {
            let path = self.dir.join("images").join(format!("{}.png", item.id));
            write_atomic(&path, &png)?;
            item.image = Some(format!("images/{}.png", item.id));
        }
        item.comments.push(Comment {
            id: comment_id,
            text,
            created: rfc3339_utc(now_ms),
        });
        let result = item.clone();
        self.document.updated = rfc3339_utc(now_ms);
        self.save()?;
        Ok(result)
    }

    pub fn edit(&mut self, comment: &str, text: String, now_ms: u64) -> Result<ReviewItem, String> {
        let index = self.item_index(comment)?;
        let entry = self.document.items[index]
            .comments
            .iter_mut()
            .find(|entry| entry.id == comment)
            .ok_or_else(|| format!("no such comment: {comment}"))?;
        entry.text = text;
        let result = self.document.items[index].clone();
        self.document.updated = rfc3339_utc(now_ms);
        self.save().map_err(|error| error.to_string())?;
        Ok(result)
    }

    pub fn remove(&mut self, comment: &str, now_ms: u64) -> Result<(), String> {
        let index = self.item_index(comment)?;
        let item = &mut self.document.items[index];
        let position = item
            .comments
            .iter()
            .position(|entry| entry.id == comment)
            .ok_or_else(|| format!("no such comment: {comment}"))?;
        item.comments.remove(position);
        if item.comments.is_empty() {
            self.document.items.remove(index);
        }
        self.document.updated = rfc3339_utc(now_ms);
        self.save().map_err(|error| error.to_string())
    }

    pub fn set_environment(&mut self, environment: Emulation) {
        self.document.environment = environment;
    }

    /// Save both files and return their paths.
    pub fn export(&self) -> std::io::Result<ExportedReview> {
        self.save()?;
        Ok(ExportedReview {
            json: self.dir.join("review.json").to_string_lossy().into_owned(),
            markdown: self.dir.join("review.md").to_string_lossy().into_owned(),
        })
    }

    fn item_index(&self, comment: &str) -> Result<usize, String> {
        let item_id = comment.split('.').next().unwrap_or(comment);
        self.document
            .items
            .iter()
            .position(|item| item.id == item_id)
            .ok_or_else(|| format!("no such comment: {comment}"))
    }

    fn save(&self) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(&self.document)
            .expect("invariant: protocol values serialize");
        write_atomic(&self.dir.join("review.json"), json.as_bytes())?;
        write_atomic(
            &self.dir.join("review.md"),
            to_markdown(&self.document).as_bytes(),
        )?;
        Ok(())
    }
}

/// Whether `target` is the same target as the one of `item` in `window`.
fn same_target(item: &ReviewItem, window: WindowId, target: &ReviewTarget) -> bool {
    if item.window != window {
        return false;
    }
    match (&item.target, target) {
        (ReviewTarget::Node { node: left, .. }, ReviewTarget::Node { node: right, .. }) => {
            left == right
        }
        (ReviewTarget::Window {}, ReviewTarget::Window {}) => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    //! Role: Tests of the review ids, the saved files and the screenshot path.

    use kivact_devtools_protocol::{AppInfo, BuildKind, NodeId, Rect, ReviewTarget, Role};

    use super::{NewComment, ReviewStore};

    fn app() -> AppInfo {
        AppInfo {
            name: "test".to_owned(),
            pid: 12,
            kivact_version: "0.1.0".to_owned(),
            build: BuildKind::Debug,
            exe: "test.exe".to_owned(),
            cwd: "C:/work".to_owned(),
            workspace_root: "C:/work".to_owned(),
        }
    }

    fn target(node: NodeId) -> ReviewTarget {
        ReviewTarget::Node {
            node,
            role: Some(Role::Button),
            name: Some("button".to_owned()),
            test_id: None,
            source: None,
            call_chain: Vec::new(),
            bounds: Rect {
                x: 0.0,
                y: 0.0,
                width: 10.0,
                height: 10.0,
            },
            states: Vec::new(),
        }
    }

    fn comment(node: NodeId, text: &str) -> NewComment {
        NewComment {
            window: 1,
            window_title: "main".to_owned(),
            target: target(node),
            style_changes: Vec::new(),
            image_png: None,
            text: text.to_owned(),
        }
    }

    #[test]
    fn items_and_comments_keep_their_ids() {
        let dir = std::env::temp_dir().join(format!("kivact-review-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let mut store = ReviewStore::new(dir.clone(), app(), "session".to_owned(), 0);
        let first = store
            .add(
                NewComment {
                    image_png: Some(vec![0x89, b'P', b'N', b'G']),
                    ..comment(7, "first")
                },
                1_000,
            )
            .expect("invariant: the comment is saved");
        assert_eq!(first.id, "C1");
        assert_eq!(first.comments.len(), 1);
        assert_eq!(first.comments[0].id, "C1.1");
        assert_eq!(first.image.as_deref(), Some("images/C1.png"));
        assert!(dir.join("images").join("C1.png").exists());

        let second = store
            .add(comment(7, "second"), 2_000)
            .expect("invariant: the comment is saved");
        assert_eq!(second.id, "C1");
        assert_eq!(second.comments.len(), 2);
        assert_eq!(second.comments[1].id, "C1.2");

        let other = store
            .add(comment(8, "other"), 3_000)
            .expect("invariant: the comment is saved");
        assert_eq!(other.id, "C2");
        assert_eq!(store.document().items.len(), 2);

        let edited = store
            .edit("C1.1", "edited".to_owned(), 4_000)
            .expect("invariant: the comment exists");
        assert_eq!(edited.comments[0].text, "edited");

        store
            .remove("C1.1", 5_000)
            .expect("invariant: the comment exists");
        let third = store
            .add(comment(7, "third"), 6_000)
            .expect("invariant: the comment is saved");
        assert_eq!(
            third.comments.last().map(|entry| entry.id.as_str()),
            Some("C1.3")
        );

        store
            .remove("C1.2", 7_000)
            .expect("invariant: the comment exists");
        store
            .remove("C1.3", 8_000)
            .expect("invariant: the comment exists");
        assert_eq!(store.document().items.len(), 1);
        assert_eq!(store.document().items[0].id, "C2");
        let reused = store
            .add(comment(7, "again"), 9_000)
            .expect("invariant: the comment is saved");
        assert_eq!(reused.id, "C3");

        assert!(dir.join("review.json").exists());
        assert!(dir.join("review.md").exists());
        let exported = store.export().expect("invariant: the review is exported");
        assert!(exported.json.ends_with("review.json"));
        assert!(exported.markdown.ends_with("review.md"));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
