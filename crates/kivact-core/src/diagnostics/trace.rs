//! Role: Debug-only ordered comparison traces for test clients.

use std::cell::RefCell;
use std::fmt::Arguments;
use std::fs::File;
use std::io::Write as _;
use std::path::{Path, PathBuf};

use kivact_reactive::Runtime;
use kivact_wire::NodeId;

use crate::view::state::ViewState;

#[derive(Default)]
pub(crate) struct Trace {
    sink: RefCell<Option<Sink>>,
}

struct Sink {
    file: File,
    stage: bool,
    semantics: bool,
    host_events: bool,
    previous: Vec<(NodeId, String)>,
}

impl Trace {
    pub(crate) fn of(rt: &Runtime) -> &Self {
        rt.ext::<Self>()
    }

    pub(crate) fn open(&self, name: &str) {
        let Ok(selected) = std::env::var("KIVACT_TRACE") else {
            return;
        };
        let has = |value| selected.split(',').any(|part| part.trim() == value);
        if !has("stage") && !has("semantics") && !has("host_events") {
            return;
        }
        let target = std::env::var_os("CARGO_TARGET_DIR")
            .map(PathBuf::from)
            .or_else(|| {
                let exe = std::env::current_exe().ok()?;
                let deps = exe.ancestors().find(|p| p.ends_with("deps"))?;
                deps.parent()?.parent().map(Path::to_path_buf)
            })
            .unwrap_or_else(|| PathBuf::from("target"));
        let dir = target.join("kivact-trace");
        let result = std::fs::create_dir_all(&dir)
            .and_then(|()| File::create(dir.join(format!("{name}.txt"))));
        match result {
            Ok(file) => {
                *self.sink.borrow_mut() = Some(Sink {
                    file,
                    stage: has("stage"),
                    semantics: has("semantics"),
                    host_events: has("host_events"),
                    previous: Vec::new(),
                })
            }
            Err(error) => tracing::warn!(%error, "could not open Kivact comparison trace"),
        }
    }
}

impl Sink {
    fn line(&mut self, text: Arguments<'_>) {
        if let Err(error) = writeln!(self.file, "{text}") {
            tracing::warn!(%error, "could not write Kivact comparison trace");
        }
    }
}

pub(crate) fn event(rt: &Runtime, text: Arguments<'_>) {
    if let Some(sink) = Trace::of(rt).sink.borrow_mut().as_mut() {
        if sink.host_events {
            sink.line(text);
        }
    }
}

pub(crate) fn stage(rt: &Runtime, name: &str) {
    let mut state = Trace::of(rt).sink.borrow_mut();
    let Some(sink) = state.as_mut() else { return };
    if sink.stage {
        sink.line(format_args!("stage {name}"));
    }
    if !sink.semantics {
        return;
    }
    let tree = ViewState::tree(rt);
    let mut pending = vec![tree.root];
    let mut current = Vec::new();
    while let Some(id) = pending.pop() {
        if let Some(node) = tree.get(id) {
            pending.extend(node.children.iter().rev().copied());
            let value = tree.with_node(id, |node| {
                format!("{:?}", node.and_then(|n| n.semantics.as_deref()))
            });
            if !sink
                .previous
                .iter()
                .any(|old| old.0 == id && old.1 == value)
            {
                sink.line(format_args!("semantics n{} {value}", id.0));
            }
            current.push((id, value));
        }
    }
    let removed: Vec<_> = sink
        .previous
        .iter()
        .filter(|old| !current.iter().any(|node| node.0 == old.0))
        .map(|old| old.0)
        .collect();
    for id in removed {
        sink.line(format_args!("semantics n{} removed", id.0));
    }
    sink.previous = current;
}

pub(crate) fn record(rt: &Runtime) {
    let mut state = Trace::of(rt).sink.borrow_mut();
    let Some(sink) = state.as_mut() else { return };
    if sink.stage {
        if let Some(record) = rt.turns().last() {
            sink.line(format_args!("stage {:?} events={} causes={:?} writes={:?} reruns={:?} runs={} ops={} notes={:?}", record.turn, record.events, record.causes, record.writes, record.reruns, record.runs, record.ops, record.notes));
        }
    }
}
