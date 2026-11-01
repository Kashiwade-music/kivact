// Application-side helpers used by the usage-example fixtures. Included by the
// fixtures after `use kivact::prelude::*;`. Nothing here is called from `main`.

#[derive(Clone, PartialEq, Debug, Default)]
pub struct Hit {
    pub id: u64,
    pub title: String,
}

pub fn search_index(q: &str) -> Result<Vec<Hit>, std::io::Error> {
    Ok(vec![Hit { id: 1, title: q.to_string() }])
}

pub mod http {
    #[derive(Clone, Debug, PartialEq)]
    pub struct Request {
        pub url: String,
    }

    #[derive(Clone, Debug, PartialEq)]
    pub struct Response {
        pub status: u16,
    }

    pub fn send(_req: Request) -> Result<Response, std::io::Error> {
        Ok(Response { status: 200 })
    }
}

pub use http::Request;

#[derive(Clone, PartialEq, Debug, Default)]
pub struct Form {
    pub url: String,
}

impl Form {
    pub fn into_request(self) -> http::Request {
        http::Request { url: self.url }
    }
}

pub mod storage {
    pub fn save<T>(_path: &std::path::Path, _value: &T) -> Result<(), std::io::Error> {
        Ok(())
    }
}

/// An external object with neither Clone nor PartialEq.
pub struct AudioPlayer {
    playing: bool,
    position: f64,
}

impl AudioPlayer {
    pub fn new() -> Self {
        AudioPlayer { playing: false, position: 0.0 }
    }
    pub fn toggle(&mut self) {
        self.playing = !self.playing;
    }
    pub fn is_playing(&self) -> bool {
        self.playing
    }
    pub fn position(&self) -> f64 {
        self.position
    }
}

pub fn format_time(seconds: f64) -> String {
    format!("{:02}:{:02}", (seconds / 60.0) as u64, (seconds % 60.0) as u64)
}

#[derive(Clone, Copy)]
pub struct KnobProps {
    pub label: &'static str,
    pub value: Value<f32>,
    pub on_change: Callback<f32>,
}

pub fn draw_knob(d: &mut Draw, size: Size, value: f32) {
    d.fill_rect(size.to_rect(), Color::token(ColorToken::SurfaceAlt));
    d.fill_rect(Rect::new(0.0, 0.0, size.width * value, 4.0), Color::token(ColorToken::Accent));
}

pub fn knob_drag(e: PointerEvent, drag: Var<Option<(PointerId, f32)>>, props: KnobProps) -> Handled {
    drag.set(Some((e.id, e.pos.y)));
    let _ = props.value.get_untracked();
    Handled::Yes
}

pub fn step(v: f32, delta: f32) -> f32 {
    (v + delta).clamp(0.0, 1.0)
}

pub fn editor_demo() -> impl IntoView {
    let status = kivact::signal(String::new());
    column((
        kivact::control::node().name("題名").part(kivact::interaction::use_text_edit(kivact::signal(String::new())).control()),
        { let label = ("保存").into_value(); node().role(Role::Button).focusable(true).name(label).child(text(label)) }.on_press(move || status.set("保存しました".into())),
        text(status).test_id("status"),
    ))
}
