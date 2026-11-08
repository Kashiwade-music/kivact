//! Role: Chrome Trace Event Format text of recorded turns and frames (pure).

use serde_json::{Value, json};

use crate::updates::{FrameInfo, TurnGraph};

/// `{"traceEvents": [...]}`: a `turn N` event with one child event per stage, and a `frame` event per frame.
pub fn chrome_trace(turns: &[TurnGraph], frames: &[FrameInfo]) -> String {
    let mut events: Vec<Value> = Vec::new();
    for turn in turns {
        events.push(complete(
            format!("turn {}", turn.turn),
            turn.started_ns,
            turn.duration_ns,
            0,
        ));
        for stage in &turn.stages {
            events.push(complete(
                format!("{:?}", stage.stage),
                stage.start_ns,
                stage.duration_ns,
                0,
            ));
        }
    }
    for frame in frames {
        let start = frame.end_ns.saturating_sub(frame.work_ns);
        events.push(complete(
            "frame".to_owned(),
            start,
            frame.work_ns,
            frame.window,
        ));
    }
    json!({ "traceEvents": events }).to_string()
}

fn complete(name: String, start_ns: u64, duration_ns: u64, tid: u64) -> Value {
    json!({ "name": name, "ph": "X", "ts": micros(start_ns), "dur": micros(duration_ns), "pid": 1, "tid": tid })
}

fn micros(ns: u64) -> f64 {
    ns as f64 / 1000.0
}
