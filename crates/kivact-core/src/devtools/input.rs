//! Role: Input injection and the input log of the DevTools target.

use kivact_devtools_agent::Responder;
use kivact_devtools_protocol as p;
use kivact_engine::contract::{
    HostRequest, ImeStateInfo, InjectedInput, PlatformKey, RawInput, RawKey, RawKeyPhase,
};
use kivact_runtime::Runtime;
use kivact_types::{InputPurpose, Key, KeyCode, Modifiers, NodeId, Point, RequestId, WindowId};

use super::convert;
use crate::view::devtools_state::{DevtoolsState, Pending};
use crate::view::state::ViewState;

fn window_not_found(raw: p::WindowId) -> p::RpcError {
    p::RpcError::new(
        p::code::WINDOW_NOT_FOUND,
        format!("window {raw} does not exist"),
    )
}

fn node_not_found(raw: p::NodeId) -> p::RpcError {
    p::RpcError::new(
        p::code::NODE_NOT_FOUND,
        format!("node {raw} does not exist"),
    )
}

fn invalid_chord() -> p::RpcError {
    p::RpcError::new(
        p::code::INVALID_PARAMS,
        "key must be one of \"Enter\", \"Ctrl+Enter\", \"Shift+Tab\", \"F5\", \"F1\"..\"F24\", or a single character",
    )
}

fn raw_key(key: Key, pressed: bool, text: Option<String>) -> InjectedInput {
    let phase = if pressed {
        RawKeyPhase::Down {
            repeat: Some(false),
            text,
        }
    } else {
        RawKeyPhase::Up
    };
    InjectedInput::Raw(RawInput::Key(RawKey {
        key: PlatformKey::Code {
            key,
            code: KeyCode::Unidentified,
        },
        phase,
    }))
}

fn key_text(key: Key, modifiers: Modifiers) -> Option<String> {
    if modifiers.ctrl || modifiers.alt || modifiers.meta {
        return None;
    }
    match key {
        Key::Char(character) => Some(character.to_string()),
        Key::Space => Some(" ".to_owned()),
        _ => None,
    }
}

fn key_inputs(text: &str) -> Result<Vec<InjectedInput>, p::RpcError> {
    let chord = convert::parse_chord(text).ok_or_else(invalid_chord)?;
    let typed = key_text(chord.key, chord.modifiers);
    Ok(vec![
        InjectedInput::Modifiers(chord.modifiers),
        raw_key(chord.key, true, typed),
        raw_key(chord.key, false, None),
        InjectedInput::Modifiers(Modifiers::default()),
    ])
}

fn text_inputs(text: &str) -> Vec<InjectedInput> {
    let mut inputs = Vec::new();
    for character in text.chars() {
        let key = match character {
            '\n' => Key::Enter,
            '\t' => Key::Tab,
            ' ' => Key::Space,
            character => Key::Char(character),
        };
        let typed = key_text(key, Modifiers::default());
        inputs.push(raw_key(key, true, typed));
        inputs.push(raw_key(key, false, None));
    }
    inputs
}

fn click_inputs(
    rt: &Runtime,
    window: WindowId,
    raw: p::NodeId,
) -> Result<Vec<InjectedInput>, p::RpcError> {
    let tree = ViewState::tree(rt);
    let reader = tree.reader();
    let node = NodeId(raw);
    if tree.get(node).is_none() || !tree.is_visible(node) || reader.window_of(node) != Some(window)
    {
        return Err(node_not_found(raw));
    }
    let Some(model) = reader.box_model(node) else {
        return Err(node_not_found(raw));
    };
    let border = model.border;
    if border.width <= 0.0 || border.height <= 0.0 {
        return Err(node_not_found(raw));
    }
    let center = Point::new(
        border.x + border.width / 2.0,
        border.y + border.height / 2.0,
    );
    if let Some(record) = ViewState::of(rt).window_record(window) {
        let size = record.env.environment().window_size;
        if center.x < 0.0 || center.y < 0.0 || center.x > size.width || center.y > size.height {
            return Err(node_not_found(raw));
        }
    }
    Ok(vec![
        InjectedInput::MouseMove(center),
        InjectedInput::MouseDown(kivact_types::MouseButton::Primary),
        InjectedInput::MouseUp(kivact_types::MouseButton::Primary),
    ])
}

fn injected_inputs(
    rt: &Runtime,
    window: WindowId,
    steps: Vec<p::InputStep>,
) -> Result<Vec<InjectedInput>, p::RpcError> {
    let mut inputs = Vec::new();
    for step in steps {
        match step {
            p::InputStep::Click { node } => inputs.extend(click_inputs(rt, window, node)?),
            p::InputStep::MouseMove { x, y } => {
                inputs.push(InjectedInput::MouseMove(Point::new(x, y)));
            }
            p::InputStep::MouseDown { button } => {
                inputs.push(InjectedInput::MouseDown(convert::mouse_button(button)));
            }
            p::InputStep::MouseUp { button } => {
                inputs.push(InjectedInput::MouseUp(convert::mouse_button(button)));
            }
            p::InputStep::Wheel { lines_x, lines_y } => {
                inputs.push(InjectedInput::Wheel { lines_x, lines_y });
            }
            p::InputStep::Key { key } => inputs.extend(key_inputs(&key)?),
            p::InputStep::Text { text } => inputs.extend(text_inputs(&text)),
            p::InputStep::A11y {
                node,
                action,
                value,
            } => inputs.push(InjectedInput::Accessibility {
                node,
                request: convert::a11y_request(action, value)?,
            }),
        }
    }
    Ok(inputs)
}

pub(super) fn inject_input(rt: &Runtime, params: p::InjectInputParams, reply: Responder) {
    let i = DevtoolsState::inspector(rt);
    if !i.can_inject {
        reply.send(Err(p::RpcError::new(
            p::code::UNSUPPORTED,
            "input injection is not supported by this target",
        )));
        return;
    }
    let reader = ViewState::tree(rt).reader();
    let Some(window) =
        convert::window_from(params.window).filter(|window| reader.windows().contains(window))
    else {
        reply.send(Err(window_not_found(params.window)));
        return;
    };
    let inputs = match injected_inputs(rt, window, params.steps) {
        Ok(inputs) => inputs,
        Err(error) => {
            reply.send(Err(error));
            return;
        }
    };
    let request = RequestId(ViewState::of(rt).requests.borrow_mut().request_id());
    ViewState::tree(rt)
        .engine()
        .effect(request, HostRequest::Inject { window, inputs });
    i.session.pending.borrow_mut().push(Pending::Inject {
        window,
        request,
        reply,
    });
}

pub(super) fn list_inputs(
    rt: &Runtime,
    params: p::ListInputsParams,
) -> Result<p::InputLog, p::RpcError> {
    let i = DevtoolsState::inspector(rt);
    let log = i.hooks.input_log.borrow();
    let mut entries: Vec<p::InputLogEntry> = log
        .iter()
        .filter(|entry| params.since.is_none_or(|since| entry.seq > since))
        .cloned()
        .collect();
    if let Some(limit) = params.limit {
        let keep = limit as usize;
        if entries.len() > keep {
            let extra = entries.len() - keep;
            entries.drain(..extra);
        }
    }
    Ok(p::InputLog { entries })
}

fn purpose_name(purpose: InputPurpose) -> String {
    match purpose {
        InputPurpose::Text => "text",
        InputPurpose::Number => "number",
        InputPurpose::Decimal => "decimal",
        InputPurpose::Email => "email",
        InputPurpose::Url => "url",
        InputPurpose::Phone => "phone",
        InputPurpose::Password => "password",
        InputPurpose::Search => "search",
    }
    .to_owned()
}

pub(super) fn get_ime_state(
    rt: &Runtime,
    params: p::WindowParams,
) -> Result<p::ImeState, p::RpcError> {
    let reader = ViewState::tree(rt).reader();
    let Some(window) =
        convert::window_from(params.window).filter(|window| reader.windows().contains(window))
    else {
        return Err(window_not_found(params.window));
    };
    let state = reader.ime_state(window);
    let (node, composing, preedit, cursor_area, purpose) = match state {
        ImeStateInfo::Inactive => (None, false, None, None, InputPurpose::Text),
        ImeStateInfo::Focused {
            node,
            purpose,
            cursor_area,
            composition,
        } => (
            Some(node.0),
            composition.is_some(),
            composition,
            cursor_area.map(convert::rect),
            purpose,
        ),
    };
    Ok(p::ImeState {
        window: params.window,
        node,
        composing,
        preedit,
        cursor_area,
        purpose: purpose_name(purpose),
    })
}

/// Answer the `inject_input` call whose inputs were delivered.
pub(super) fn delivered(rt: &Runtime, request: RequestId) {
    let i = DevtoolsState::inspector(rt);
    let mut pending = i.session.pending.borrow_mut();
    let Some(index) = pending.iter().position(
        |item| matches!(item, Pending::Inject { request: queued, .. } if *queued == request),
    ) else {
        return;
    };
    let Pending::Inject { reply, .. } = pending.remove(index) else {
        unreachable!("invariant: the pending item was matched as an injection")
    };
    drop(pending);
    reply.send(Ok(p::Reply::InjectInput(p::Empty {})));
}

pub(super) fn route(rt: &Runtime, call: p::Call, reply: Responder) -> Option<(p::Call, Responder)> {
    match call {
        p::Call::InjectInput(params) => inject_input(rt, params, reply),
        p::Call::ListInputs(params) => {
            reply.send(list_inputs(rt, params).map(p::Reply::ListInputs))
        }
        p::Call::GetImeState(params) => {
            reply.send(get_ime_state(rt, params).map(p::Reply::GetImeState))
        }
        other => return Some((other, reply)),
    }
    None
}
