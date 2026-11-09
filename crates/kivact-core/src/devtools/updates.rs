//! Role: Recording, turns and frames of the DevTools target.

use std::collections::{BTreeSet, HashMap};
use std::panic::Location;
use std::time::Duration;

use kivact_devtools_agent::Responder;
use kivact_devtools_protocol as p;
use kivact_engine::contract::{EngineRecords, FrameRecord, SyncRecord};
use kivact_runtime::{CauseEntry, CauseTag, DetailEdge, DetailRun, DetailWrite, Runtime};
use kivact_types::Revision;

use super::{convert, lifecycle};
use crate::view::devtools_state::{
    DevtoolsState, Inspector, Pending, PendingGraph, PropWrite, recording_off,
};
use crate::view::state::ViewState;

/// Completed turns kept for `list_turns`.
const TURN_CAP: usize = 1000;
/// Frames kept per window for `list_frames`.
const FRAME_CAP: usize = 3000;
/// Graph nodes kept for one turn.
const NODE_CAP: usize = 2000;
/// A group of nodes this large with the same kind, source and window is merged.
const MERGE_AT: usize = 8;

/// Whether any recording is on (all false means not recording).
fn is_recording(settings: &p::RecordingSettings) -> bool {
    settings.updates || settings.frames || settings.input
}

pub(super) fn start_recording(
    rt: &Runtime,
    params: p::RecordingSettings,
) -> Result<p::Empty, p::RpcError> {
    let i = DevtoolsState::inspector(rt);
    let recording = i.hooks.recording.borrow().clone();
    if !is_recording(&recording) {
        i.session.turns.borrow_mut().clear();
        i.session.frames.borrow_mut().clear();
        i.session.pending_graphs.borrow_mut().clear();
        i.session.update_counts.borrow_mut().clear();
        i.hooks.prop_writes.borrow_mut().clear();
    }
    *i.hooks.recording.borrow_mut() = params;
    lifecycle::apply_recording(rt);
    Ok(p::Empty {})
}

pub(super) fn stop_recording(rt: &Runtime, params: p::Empty) -> Result<p::Empty, p::RpcError> {
    let _ = params;
    stop(rt);
    Ok(p::Empty {})
}

pub(super) fn get_recording(
    rt: &Runtime,
    params: p::Empty,
) -> Result<p::RecordingSettings, p::RpcError> {
    let _ = params;
    let i = DevtoolsState::inspector(rt);
    Ok(i.hooks.recording.borrow().clone())
}

pub(super) fn list_turns(rt: &Runtime, params: p::ListTurnsParams, reply: Responder) {
    let i = DevtoolsState::inspector(rt);
    let turns = select_turns(&i.session.turns.borrow(), &params);
    if !turns.is_empty() || params.wait_ms.is_none() {
        reply.send(Ok(p::Reply::ListTurns(p::TurnList { turns })));
        return;
    }
    let wait_ms = params.wait_ms.unwrap_or(0);
    let deadline_ns = rt
        .now_ns()
        .saturating_add(u64::from(wait_ms).saturating_mul(1_000_000));
    i.session.pending.borrow_mut().push(Pending::Turns {
        params,
        deadline_ns,
        reply,
    });
    let _ = kivact_runtime::spawn_in(
        rt.root(),
        async move {
            kivact_runtime::sleep(Duration::from_millis(u64::from(wait_ms))).await;
        },
        Location::caller(),
    );
}

/// The turns after `since`, the newest `limit` of them, oldest first.
fn select_turns(
    turns: &std::collections::VecDeque<p::TurnGraph>,
    params: &p::ListTurnsParams,
) -> Vec<p::TurnGraph> {
    let mut selected: Vec<p::TurnGraph> = turns
        .iter()
        .filter(|turn| params.since.is_none_or(|since| turn.turn > since))
        .cloned()
        .collect();
    if let Some(limit) = params.limit {
        let limit = limit as usize;
        if selected.len() > limit {
            selected.drain(..selected.len() - limit);
        }
    }
    selected
}

/// Finish the graphs whose frame arrived, and record this turn's graph.
pub(super) fn after_turn(rt: &Runtime, records: &EngineRecords) {
    let i = DevtoolsState::inspector(rt);
    apply_records(i, records);
    if i.hooks.recording.borrow().updates {
        record_turn(rt, i);
    }
    answer_pending_turns(rt, i);
}

/// Finish every waiting graph and stop recording.
pub(super) fn stop(rt: &Runtime) {
    let i = DevtoolsState::inspector(rt);
    let records = ViewState::tree(rt).reader().take_engine_records();
    apply_records(i, &records);
    finish_all(i);
    answer_pending_turns(rt, i);
    *i.hooks.recording.borrow_mut() = recording_off();
    lifecycle::apply_recording(rt);
}

/// Turn the detailed trace of this turn into a graph waiting for its frame.
fn record_turn(rt: &Runtime, i: &Inspector) {
    let detail = rt.take_turn_details().pop();
    let writes = std::mem::take(&mut *i.hooks.prop_writes.borrow_mut());
    // A recording started in the middle of a turn leaves a partial record with
    // no turn number; the next turn's record is the first one with a number.
    let Some(detail) = detail.filter(|detail| detail.turn != 0) else {
        return;
    };
    let revision = Revision(ViewState::tree(rt).engine().stats().revision);
    let graph = build_graph(rt, i, &detail, &writes);
    queue_pending(i, PendingGraph { revision, graph });
}

/// Wait for a new graph's frame, completing the oldest waiting graph when
/// `TURN_CAP` graphs already wait.
fn queue_pending(i: &Inspector, item: PendingGraph) {
    let oldest = {
        let mut pending = i.session.pending_graphs.borrow_mut();
        let oldest = (pending.len() >= TURN_CAP)
            .then(|| pending.pop_front())
            .flatten();
        pending.push_back(item);
        oldest
    };
    if let Some(oldest) = oldest {
        push_completed(i, vec![oldest.graph]);
    }
}

/// Build one turn's cause graph from the detailed trace.
fn build_graph(
    rt: &Runtime,
    i: &Inspector,
    detail: &kivact_runtime::TurnDetail,
    writes: &[PropWrite],
) -> p::TurnGraph {
    let mut graph = GraphBuilder::new();
    graph.add_causes(&detail.causes);
    graph.add_writes(i, &detail.writes);
    graph.add_runs(i, &detail.runs);
    graph.add_prop_changes(rt, i, writes);
    graph.add_runtime_edges(&detail.edges);
    let started_ns = detail
        .stages
        .first()
        .map_or_else(|| rt.now_ns(), |span| span.start_ns);
    let stages = detail
        .stages
        .iter()
        .map(|span| p::StageTime {
            stage: convert::turn_stage(span.stage),
            start_ns: span.start_ns,
            duration_ns: span.duration_ns,
        })
        .collect();
    let finished = graph.finish(detail.dropped);
    p::TurnGraph {
        turn: detail.turn,
        started_ns,
        duration_ns: detail.duration_ns,
        stages,
        nodes: finished.nodes,
        edges: finished.edges,
        changed_nodes: finished.changed_nodes,
        layout: Vec::new(),
        frames: Vec::new(),
        wasted: finished.wasted,
        truncated: finished.truncated,
    }
}

/// The agent's name hint for a source location.
fn hint(i: &Inspector, at: &'static Location<'static>) -> Option<String> {
    i.agent.borrow_mut().name_hint(&convert::location(at))
}

/// One cause graph while it is assembled.
struct GraphBuilder {
    nodes: Vec<p::CauseNode>,
    edges: Vec<p::CauseEdge>,
    /// The graph node of each runtime node written or run in this turn.
    by_runtime: HashMap<kivact_runtime::NodeId, u32>,
    /// The graph node of each cause, by `TurnDetail.causes` index.
    causes: Vec<u32>,
    wasted: p::WastedWork,
    changed: BTreeSet<u64>,
}

impl GraphBuilder {
    fn new() -> GraphBuilder {
        GraphBuilder {
            nodes: Vec::new(),
            edges: Vec::new(),
            by_runtime: HashMap::new(),
            causes: Vec::new(),
            wasted: p::WastedWork {
                equal_writes: 0,
                equal_memos: 0,
                unchanged_props: 0,
            },
            changed: BTreeSet::new(),
        }
    }

    /// Add `node` and return the id it was given.
    fn push_node(&mut self, mut node: p::CauseNode) -> u32 {
        let id = self.nodes.len() as u32;
        node.id = id;
        self.nodes.push(node);
        id
    }

    fn add_causes(&mut self, causes: &[CauseEntry]) {
        for cause in causes {
            let id = self.push_node(p::CauseNode {
                id: 0,
                kind: cause_kind(cause.tag),
                label: cause.label.clone(),
                ty: None,
                name_hint: None,
                source: None,
                node: None,
                window: None,
                count: 1,
                wasted: false,
                members: Vec::new(),
            });
            self.causes.push(id);
        }
    }

    fn add_writes(&mut self, i: &Inspector, writes: &[DetailWrite]) {
        for write in writes {
            let id = self.push_node(p::CauseNode {
                id: 0,
                kind: p::CauseKind::SignalWrite,
                label: write.ty.to_owned(),
                ty: Some(write.ty.to_owned()),
                name_hint: hint(i, write.created_at),
                source: Some(convert::location(write.created_at)),
                node: None,
                window: None,
                count: 1,
                wasted: !write.changed,
                members: Vec::new(),
            });
            if !write.changed {
                self.wasted.equal_writes = self.wasted.equal_writes.saturating_add(1);
            }
            self.by_runtime.insert(write.node, id);
            self.cause_edge(write.cause, id);
        }
    }

    fn add_runs(&mut self, i: &Inspector, runs: &[DetailRun]) {
        for run in runs {
            let kind = if run.kind == "memo" {
                p::CauseKind::MemoRun
            } else {
                p::CauseKind::EffectRun
            };
            let id = self.push_node(p::CauseNode {
                id: 0,
                kind,
                label: run.kind.to_owned(),
                ty: None,
                name_hint: hint(i, run.at),
                source: Some(convert::location(run.at)),
                node: None,
                window: None,
                count: 1,
                wasted: !run.changed,
                members: Vec::new(),
            });
            if run.kind == "memo" && !run.changed {
                self.wasted.equal_memos = self.wasted.equal_memos.saturating_add(1);
            }
            self.by_runtime.insert(run.node, id);
            self.cause_edge(run.cause, id);
        }
    }

    fn add_prop_changes(&mut self, rt: &Runtime, i: &Inspector, writes: &[PropWrite]) {
        if writes.is_empty() {
            return;
        }
        let tree = ViewState::tree(rt);
        let reader = tree.reader();
        for write in writes {
            let window = reader.window_of(write.node).map(convert::window);
            let effect = write
                .effect
                .and_then(|effect| self.by_runtime.get(&effect).copied());
            let id = self.push_node(p::CauseNode {
                id: 0,
                kind: p::CauseKind::PropChange,
                label: write.prop.name().to_owned(),
                ty: None,
                name_hint: None,
                source: None,
                node: Some(write.node.0),
                window,
                count: 1,
                wasted: !write.changed,
                members: Vec::new(),
            });
            if write.changed {
                let mut counts = i.session.update_counts.borrow_mut();
                let count = counts.entry(write.node).or_insert(0);
                *count = count.saturating_add(1);
                self.changed.insert(write.node.0);
            } else {
                self.wasted.unchanged_props = self.wasted.unchanged_props.saturating_add(1);
            }
            if let Some(source) = effect {
                self.edges.push(p::CauseEdge { source, target: id });
            }
        }
    }

    fn add_runtime_edges(&mut self, edges: &[DetailEdge]) {
        for edge in edges {
            let Some(source) = self.by_runtime.get(&edge.source).copied() else {
                continue;
            };
            let Some(target) = self.by_runtime.get(&edge.target).copied() else {
                continue;
            };
            if source != target {
                self.edges.push(p::CauseEdge { source, target });
            }
        }
    }

    /// Add an edge from the cause at `cause` to `target`, if the cause exists.
    fn cause_edge(&mut self, cause: Option<u32>, target: u32) {
        let Some(source) = cause
            .and_then(|index| self.causes.get(index as usize))
            .copied()
        else {
            return;
        };
        self.edges.push(p::CauseEdge { source, target });
    }

    /// Merge repeated nodes, drop duplicate edges and truncate the graph.
    fn finish(mut self, dropped: u32) -> FinishedGraph {
        let remap = merge_nodes(&mut self.nodes);
        let mut edges = Vec::with_capacity(self.edges.len());
        for edge in self.edges {
            let source = remap.get(&edge.source).copied();
            let target = remap.get(&edge.target).copied();
            let (Some(source), Some(target)) = (source, target) else {
                continue;
            };
            if source != target {
                edges.push(p::CauseEdge { source, target });
            }
        }
        edges.sort_by_key(|edge| (edge.source, edge.target));
        edges.dedup();
        let mut truncated = dropped > 0;
        if self.nodes.len() > NODE_CAP {
            self.nodes.truncate(NODE_CAP);
            edges.retain(|edge| {
                (edge.source as usize) < NODE_CAP && (edge.target as usize) < NODE_CAP
            });
            truncated = true;
        }
        FinishedGraph {
            nodes: self.nodes,
            edges,
            changed_nodes: self.changed.into_iter().collect(),
            wasted: self.wasted,
            truncated,
        }
    }
}

/// One cause graph after the final merge, dedup and truncation.
struct FinishedGraph {
    nodes: Vec<p::CauseNode>,
    edges: Vec<p::CauseEdge>,
    changed_nodes: Vec<u64>,
    wasted: p::WastedWork,
    truncated: bool,
}

/// The key nodes are merged by: kind, source and window.
type GroupKey = (u8, Option<(String, u32, u32)>, Option<u64>);

fn group_key(node: &p::CauseNode) -> GroupKey {
    (
        kind_code(node.kind),
        node.source
            .as_ref()
            .map(|source| (source.file.clone(), source.line, source.column)),
        node.window,
    )
}

fn kind_code(kind: p::CauseKind) -> u8 {
    match kind {
        p::CauseKind::Input => 0,
        p::CauseKind::Timer => 1,
        p::CauseKind::Task => 2,
        p::CauseKind::Mount => 3,
        p::CauseKind::Devtools => 4,
        p::CauseKind::SignalWrite => 5,
        p::CauseKind::MemoRun => 6,
        p::CauseKind::EffectRun => 7,
        p::CauseKind::PropChange => 8,
        p::CauseKind::Layout => 9,
        p::CauseKind::Paint => 10,
        p::CauseKind::Frame => 11,
    }
}

fn cause_kind(tag: CauseTag) -> p::CauseKind {
    match tag {
        CauseTag::Input => p::CauseKind::Input,
        CauseTag::Timer => p::CauseKind::Timer,
        CauseTag::Task => p::CauseKind::Task,
        CauseTag::Mount => p::CauseKind::Mount,
        CauseTag::Devtools => p::CauseKind::Devtools,
    }
}

/// Merge every group of at least `MERGE_AT` nodes and return old-to-new ids.
fn merge_nodes(nodes: &mut Vec<p::CauseNode>) -> HashMap<u32, u32> {
    let mut groups: Vec<(GroupKey, Vec<usize>)> = Vec::new();
    for (index, node) in nodes.iter().enumerate() {
        let key = group_key(node);
        match groups.iter_mut().find(|(group, _)| *group == key) {
            Some((_, members)) => members.push(index),
            None => groups.push((key, vec![index])),
        }
    }
    if !groups.iter().any(|(_, members)| members.len() >= MERGE_AT) {
        return nodes.iter().map(|node| (node.id, node.id)).collect();
    }
    let old = std::mem::take(nodes);
    let mut remap = HashMap::with_capacity(old.len());
    for (_, members) in groups {
        let new_id = nodes.len() as u32;
        let mut merged = old[members[0]].clone();
        merged.id = new_id;
        merged.count = members
            .iter()
            .fold(0u32, |count, index| count.saturating_add(old[*index].count));
        if members.len() > 1 {
            merged.members = merged_members(members.iter().map(|index| &old[*index]));
        }
        for index in &members {
            remap.insert(old[*index].id, new_id);
        }
        nodes.push(merged);
    }
    remap
}

/// The element nodes of the merged members, in first-seen order without duplicates.
fn merged_members<'a>(members: impl Iterator<Item = &'a p::CauseNode>) -> Vec<u64> {
    let mut seen = BTreeSet::new();
    members
        .filter_map(|member| member.node)
        .filter(|node| seen.insert(*node))
        .collect()
}

/// The reasons of a `SyncRecord` as protocol layout reasons.
fn layout_reasons(bits: u8) -> Vec<p::LayoutReason> {
    let mut reasons = Vec::new();
    if bits & 1 != 0 {
        reasons.push(p::LayoutReason::Pass);
    }
    if bits & 2 != 0 {
        reasons.push(p::LayoutReason::Style);
    }
    if bits & 4 != 0 {
        reasons.push(p::LayoutReason::Viewport);
    }
    reasons
}

/// Add the layout and paint nodes and the layout pass of one sync record.
fn add_sync(graph: &mut p::TurnGraph, sync: &SyncRecord) {
    let window = convert::window(sync.window);
    let layout = (sync.reasons != 0).then(|| {
        graph.layout.push(p::LayoutPass {
            window,
            reasons: layout_reasons(sync.reasons),
            measured: sync.measured.len() as u32,
            laid_out: sync.laid_out.len() as u32,
        });
        push_graph_node(graph, p::CauseKind::Layout, "layout", window, 1)
    });
    if !sync.redrawn.is_empty() {
        let paint = push_graph_node(
            graph,
            p::CauseKind::Paint,
            "paint",
            window,
            sync.redrawn.len() as u32,
        );
        if let Some(layout) = layout {
            push_graph_edge(graph, layout, paint);
        }
    }
}

/// Add the frame node and the frame info of one frame record.
fn add_frame(graph: &mut p::TurnGraph, record: &FrameRecord) {
    let info = frame_info(record);
    let id = push_graph_node(graph, p::CauseKind::Frame, "frame", info.window, 1);
    let from = graph
        .nodes
        .iter()
        .rev()
        .find(|node| {
            node.id != id
                && node.window == Some(info.window)
                && matches!(node.kind, p::CauseKind::Paint | p::CauseKind::Layout)
        })
        .map(|node| node.id);
    if let Some(source) = from {
        push_graph_edge(graph, source, id);
    }
    graph.frames.push(info);
}

fn push_graph_node(
    graph: &mut p::TurnGraph,
    kind: p::CauseKind,
    label: &str,
    window: u64,
    count: u32,
) -> u32 {
    let id = graph.nodes.len() as u32;
    graph.nodes.push(p::CauseNode {
        id,
        kind,
        label: label.to_owned(),
        ty: None,
        name_hint: None,
        source: None,
        node: None,
        window: Some(window),
        count,
        wasted: false,
        members: Vec::new(),
    });
    id
}

fn push_graph_edge(graph: &mut p::TurnGraph, source: u32, target: u32) {
    graph.edges.push(p::CauseEdge { source, target });
}

/// The wire frame of an engine frame record.
pub(super) fn frame_info(record: &FrameRecord) -> p::FrameInfo {
    p::FrameInfo {
        window: convert::window(record.window),
        end_ns: record.end_ns,
        work_ns: record.work_ns,
        scene_ns: record.scene_ns,
        raster_ns: record.raster_ns,
        present_ns: record.present_ns,
        chunks_rastered: record.stats.chunks_rastered,
        pixels_rastered: record.stats.pixels_rastered,
        damage_pixels: record.stats.damage_pixels,
        renderer: if record.gpu {
            p::Renderer::Gpu
        } else {
            p::Renderer::Cpu
        },
        frame_period_ns: record.frame_period_ns,
        over_budget: record.frame_period_ns != 0 && record.work_ns > record.frame_period_ns,
    }
}

/// Store the new frames and complete the graphs their records finish.
fn apply_records(i: &Inspector, records: &EngineRecords) {
    record_frames(i, records);
    let pending = std::mem::take(&mut *i.session.pending_graphs.borrow_mut());
    let mut completed = Vec::new();
    for mut item in pending {
        let revision = item.revision;
        for sync in records
            .syncs
            .iter()
            .filter(|sync| sync.revision == revision)
        {
            add_sync(&mut item.graph, sync);
        }
        let mut framed = false;
        for frame in records
            .frames
            .iter()
            .filter(|frame| frame.revision == revision)
        {
            add_frame(&mut item.graph, frame);
            framed = true;
        }
        let newer = records
            .syncs
            .iter()
            .any(|sync| sync.revision.0 > revision.0)
            || records
                .frames
                .iter()
                .any(|frame| frame.revision.0 > revision.0);
        if framed || newer {
            completed.push(item.graph);
        } else {
            i.session.pending_graphs.borrow_mut().push_back(item);
        }
    }
    push_completed(i, completed);
}

/// Put the new frames into the per-window history and notify the client.
fn record_frames(i: &Inspector, records: &EngineRecords) {
    if records.frames.is_empty() {
        return;
    }
    let mut history = i.session.frames.borrow_mut();
    for record in &records.frames {
        let info = frame_info(record);
        let frames = history.entry(record.window).or_default();
        if frames.len() == FRAME_CAP {
            frames.pop_front();
        }
        frames.push_back(info.clone());
        if i.wants(p::Topic::Frames) {
            i.notify(p::Notification::FrameRecorded(info));
        }
    }
}

/// Move finished graphs to the turn history, keeping the newest `TURN_CAP`.
fn push_completed(i: &Inspector, graphs: Vec<p::TurnGraph>) {
    if graphs.is_empty() {
        return;
    }
    let mut turns = i.session.turns.borrow_mut();
    for graph in graphs {
        if turns.len() == TURN_CAP {
            turns.pop_front();
        }
        if i.wants(p::Topic::Turns) {
            i.notify(p::Notification::TurnRecorded(graph.clone()));
        }
        turns.push_back(graph);
    }
}

/// Complete every waiting graph without a frame record.
fn finish_all(i: &Inspector) {
    let pending = std::mem::take(&mut *i.session.pending_graphs.borrow_mut());
    let graphs = pending.into_iter().map(|item| item.graph).collect();
    push_completed(i, graphs);
}

/// Answer `list_turns` calls whose turns arrived or whose deadline passed.
fn answer_pending_turns(rt: &Runtime, i: &Inspector) {
    let now_ns = rt.now_ns();
    let pending = std::mem::take(&mut *i.session.pending.borrow_mut());
    for item in pending {
        let Pending::Turns {
            params,
            deadline_ns,
            reply,
        } = item
        else {
            i.session.pending.borrow_mut().push(item);
            continue;
        };
        let turns = select_turns(&i.session.turns.borrow(), &params);
        if !turns.is_empty() || now_ns >= deadline_ns {
            reply.send(Ok(p::Reply::ListTurns(p::TurnList { turns })));
        } else {
            i.session.pending.borrow_mut().push(Pending::Turns {
                params,
                deadline_ns,
                reply,
            });
        }
    }
}

pub(super) fn route(rt: &Runtime, call: p::Call, reply: Responder) -> Option<(p::Call, Responder)> {
    match call {
        p::Call::StartRecording(params) => {
            reply.send(start_recording(rt, params).map(p::Reply::StartRecording))
        }
        p::Call::StopRecording(params) => {
            reply.send(stop_recording(rt, params).map(p::Reply::StopRecording))
        }
        p::Call::GetRecording(params) => {
            reply.send(get_recording(rt, params).map(p::Reply::GetRecording))
        }
        p::Call::ListTurns(params) => list_turns(rt, params, reply),
        other => return Some((other, reply)),
    }
    None
}
