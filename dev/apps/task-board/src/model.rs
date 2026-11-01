//! Role: Board data model (lanes and tasks) and its JSON form.

use serde::ser::{SerializeSeq, SerializeStruct};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::sync::Arc;

const PAGE_SIZE: usize = 256;

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum Lane {
    #[default]
    Planned,
    InProgress,
    Done,
}

impl Lane {
    pub fn label(self) -> &'static str {
        match self {
            Self::Planned => "予定",
            Self::InProgress => "進行中",
            Self::Done => "完了",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Planned => Self::InProgress,
            Self::InProgress | Self::Done => Self::Done,
        }
    }

    pub fn previous(self) -> Self {
        match self {
            Self::Planned | Self::InProgress => Self::Planned,
            Self::Done => Self::InProgress,
        }
    }

    /// The three lanes in display order.
    pub const ALL: [Lane; 3] = [Lane::Planned, Lane::InProgress, Lane::Done];

    /// Stable id used for `test_id` and UI Automation.
    pub fn slug(self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::InProgress => "progress",
            Self::Done => "done",
        }
    }
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Task {
    pub id: u64,
    pub title: String,
    pub notes: String,
    pub lane: Lane,
}

// Deep `Task` clones are counted in unit tests to prove that a single edit on
// a 100k board copies one task (the pages are shared `Arc`s).
#[cfg(test)]
thread_local! {
    static TASK_CLONES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

#[cfg(test)]
pub(crate) fn reset_task_clones() {
    TASK_CLONES.with(|count| count.set(0));
}

#[cfg(test)]
pub(crate) fn task_clone_count() -> usize {
    TASK_CLONES.with(std::cell::Cell::get)
}

impl Clone for Task {
    fn clone(&self) -> Self {
        #[cfg(test)]
        TASK_CLONES.with(|count| count.set(count.get() + 1));
        Self {
            id: self.id,
            title: self.title.clone(),
            notes: self.notes.clone(),
            lane: self.lane,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Board {
    next_id: u64,
    pages: Vec<Arc<Vec<Arc<Task>>>>,
}

#[derive(Deserialize)]
struct BoardWire {
    next_id: u64,
    tasks: Vec<Task>,
}

impl From<BoardWire> for Board {
    fn from(wire: BoardWire) -> Self {
        let pages = wire
            .tasks
            .into_iter()
            .map(Arc::new)
            .collect::<Vec<_>>()
            .chunks(PAGE_SIZE)
            .map(|chunk| Arc::new(chunk.to_vec()))
            .collect();
        Self {
            next_id: wire.next_id,
            pages,
        }
    }
}

impl<'de> Deserialize<'de> for Board {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        BoardWire::deserialize(deserializer).map(Into::into)
    }
}

struct TasksRef<'a>(&'a Board);

impl Serialize for TasksRef<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut sequence = serializer.serialize_seq(Some(self.0.len()))?;
        for task in self.0.pages.iter().flat_map(|page| page.iter()) {
            sequence.serialize_element(task.as_ref())?;
        }
        sequence.end()
    }
}

impl Serialize for Board {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut board = serializer.serialize_struct("Board", 2)?;
        board.serialize_field("next_id", &self.next_id)?;
        board.serialize_field("tasks", &TasksRef(self))?;
        board.end()
    }
}

impl Default for Board {
    fn default() -> Self {
        BoardWire {
            next_id: 4,
            tasks: vec![
                Task {
                    id: 1,
                    title: "新しいタスクを追加する".into(),
                    notes: "上のフォームから追加できます。".into(),
                    lane: Lane::Planned,
                },
                Task {
                    id: 2,
                    title: "カードを選んで編集する".into(),
                    notes: "列のカードを選択して編集します。".into(),
                    lane: Lane::InProgress,
                },
                Task {
                    id: 3,
                    title: "作業を完了する".into(),
                    notes: "右端が完了列です。".into(),
                    lane: Lane::Done,
                },
            ],
        }
        .into()
    }
}

impl Board {
    pub fn len(&self) -> usize {
        self.pages.iter().map(|page| page.len()).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.pages.is_empty()
    }

    pub fn add(&mut self, title: String, notes: String) -> Option<u64> {
        let title = title.trim();
        if title.is_empty() {
            return None;
        }
        let id = self.next_id;
        self.next_id = self.next_id.saturating_add(1);
        let task = Arc::new(Task {
            id,
            title: title.into(),
            notes,
            lane: Lane::Planned,
        });
        if let Some(last) = self.pages.last_mut().filter(|page| page.len() < PAGE_SIZE) {
            Arc::make_mut(last).push(task);
        } else {
            self.pages.push(Arc::new(vec![task]));
        }
        Some(id)
    }

    pub fn edit(&mut self, id: u64, title: String, notes: String) -> bool {
        let title = title.trim();
        if title.is_empty() {
            return false;
        }
        let Some(task) = self.task_mut(id) else {
            return false;
        };
        task.title = title.into();
        task.notes = notes;
        true
    }

    pub fn move_task(&mut self, id: u64, forward: bool) -> bool {
        let Some(task) = self.task_mut(id) else {
            return false;
        };
        task.lane = if forward {
            task.lane.next()
        } else {
            task.lane.previous()
        };
        true
    }

    pub fn set_lane(&mut self, id: u64, lane: Lane) -> bool {
        let Some(task) = self.task_mut(id) else {
            return false;
        };
        task.lane = lane;
        true
    }

    pub fn delete(&mut self, id: u64) -> bool {
        for page_index in 0..self.pages.len() {
            if let Some(task_index) = self.pages[page_index].iter().position(|task| task.id == id) {
                let page = Arc::make_mut(&mut self.pages[page_index]);
                page.remove(task_index);
                if page.is_empty() {
                    self.pages.remove(page_index);
                }
                return true;
            }
        }
        false
    }

    pub fn task(&self, id: u64) -> Option<&Task> {
        self.pages
            .iter()
            .flat_map(|page| page.iter())
            .find(|task| task.id == id)
            .map(AsRef::as_ref)
    }

    fn task_mut(&mut self, id: u64) -> Option<&mut Task> {
        for page in &mut self.pages {
            if let Some(index) = page.iter().position(|task| task.id == id) {
                return Some(Arc::make_mut(&mut Arc::make_mut(page)[index]));
            }
        }
        None
    }

    /// Tasks of `lane` whose title or notes contain `query` (case-insensitive).
    /// Returns shared `Arc`s: no task is deep-cloned.
    pub fn matching(&self, lane: Lane, query: &str) -> Vec<Arc<Task>> {
        let query = query.to_lowercase();
        self.pages
            .iter()
            .flat_map(|page| page.iter())
            .filter(|task| {
                task.lane == lane
                    && (query.is_empty()
                        || task.title.to_lowercase().contains(&query)
                        || task.notes.to_lowercase().contains(&query))
            })
            .cloned()
            .collect()
    }

    pub fn generated(count: usize) -> Self {
        let pages = (1..=count as u64)
            .map(|id| {
                Arc::new(Task {
                    id,
                    title: format!("Generated task {id}"),
                    notes: String::new(),
                    lane: Lane::Planned,
                })
            })
            .collect::<Vec<_>>()
            .chunks(PAGE_SIZE)
            .map(|chunk| Arc::new(chunk.to_vec()))
            .collect();
        Self {
            next_id: count as u64 + 1,
            pages,
        }
    }
}
