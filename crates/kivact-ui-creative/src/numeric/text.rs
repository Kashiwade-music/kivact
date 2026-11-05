//! Role: Numeric formatting, finite arithmetic parsing and displayed mixed values.
use kivact::i18n::message;
use kivact::state::{IntoValue, Value};
use std::rc::Rc;
pub(super) type TextParser = Rc<dyn Fn(&str, f64) -> Result<f64, String>>;
pub(super) fn format_scalar(value: f64, precision: usize, unit: &str, fps: Option<u32>) -> String {
    if let Some(fps) = fps {
        let f = value.max(0.0).floor() as u64;
        let fps = u64::from(fps);
        format!(
            "{:02}:{:02}:{:02}:{:02}",
            f / (fps * 3600),
            f / (fps * 60) % 60,
            f / fps % 60,
            f % fps
        )
    } else {
        format!("{value:.*}{unit}", precision)
    }
}
fn parse_scalar(s: &str, fps: Option<u32>, current: f64) -> Option<f64> {
    let s = s.trim();
    if let Some(fps) = fps {
        if s.contains(':') {
            let parts = s
                .split(':')
                .map(str::parse::<u64>)
                .collect::<std::result::Result<Vec<_>, _>>()
                .ok()?;
            if !(2..=4).contains(&parts.len()) {
                return None;
            }
            let mut fields = [0_u64; 4];
            fields[4 - parts.len()..].copy_from_slice(&parts);
            if fields[1] >= 60 || fields[2] >= 60 || fields[3] >= u64::from(fps) {
                return None;
            }
            let result = ((fields[0] as f64 * 60.0 + fields[1] as f64) * 60.0 + fields[2] as f64)
                * f64::from(fps)
                + fields[3] as f64;
            return result.is_finite().then_some(result);
        }
        if s.starts_with(['+', '-']) {
            return evaluate_expression(s).map(|delta| current + delta);
        }
    }
    evaluate_expression(s)
}
/// Evaluate finite scalar arithmetic (+, -, *, /, parentheses and pi).
pub fn evaluate_expression(s: &str) -> Option<f64> {
    let mut p = Parser {
        input: s.as_bytes(),
        at: 0,
    };
    let value = p.expression()?;
    p.spaces();
    (p.at == p.input.len() && value.is_finite()).then_some(value)
}

struct Parser<'a> {
    input: &'a [u8],
    at: usize,
}
impl Parser<'_> {
    fn spaces(&mut self) {
        while self.input.get(self.at).is_some_and(u8::is_ascii_whitespace) {
            self.at += 1;
        }
    }
    fn expression(&mut self) -> Option<f64> {
        let mut value = self.product()?;
        loop {
            self.spaces();
            match self.input.get(self.at) {
                Some(b'+') => {
                    self.at += 1;
                    value += self.product()?;
                }
                Some(b'-') => {
                    self.at += 1;
                    value -= self.product()?;
                }
                _ => return Some(value),
            }
        }
    }
    fn product(&mut self) -> Option<f64> {
        let mut value = self.atom()?;
        loop {
            self.spaces();
            match self.input.get(self.at) {
                Some(b'*') => {
                    self.at += 1;
                    value *= self.atom()?;
                }
                Some(b'/') => {
                    self.at += 1;
                    value /= self.atom()?;
                }
                _ => return Some(value),
            }
        }
    }
    fn atom(&mut self) -> Option<f64> {
        self.spaces();
        if self.input.get(self.at) == Some(&b'-') {
            self.at += 1;
            return Some(-self.atom()?);
        }
        if self.input.get(self.at) == Some(&b'+') {
            self.at += 1;
            return self.atom();
        }
        if self.input.get(self.at) == Some(&b'(') {
            self.at += 1;
            let v = self.expression()?;
            self.spaces();
            if self.input.get(self.at) != Some(&b')') {
                return None;
            }
            self.at += 1;
            return Some(v);
        }
        if self.input.get(self.at..self.at + 2) == Some(b"pi") {
            self.at += 2;
            return Some(std::f64::consts::PI);
        }
        let start = self.at;
        while self
            .input
            .get(self.at)
            .is_some_and(|c| c.is_ascii_digit() || *c == b'.')
        {
            self.at += 1;
        }
        std::str::from_utf8(self.input.get(start..self.at)?)
            .ok()?
            .parse()
            .ok()
    }
}

pub(super) fn scalar_parser(
    text_parser: Option<TextParser>,
    timecode: Option<Value<u32>>,
) -> TextParser {
    let not_finite = message("creative.numeric.not_finite", "Enter a finite number");
    // `from` is the value the text field opened with, which a relative
    // entry such as `+5` is added to.
    Rc::new(move |s: &str, from: f64| -> Result<f64, String> {
        match text_parser.as_ref() {
            Some(parser) => parser(s, from).and_then(|v| {
                if v.is_finite() {
                    Ok(v)
                } else {
                    Err(not_finite.get())
                }
            }),
            None => {
                parse_scalar(s, timecode.map(|fps| fps.get()), from).ok_or_else(|| not_finite.get())
            }
        }
    })
}

pub(super) fn display_value(
    mixed: Value<bool>,
    value: Value<f64>,
    min: Value<f64>,
    max: Value<f64>,
    precision: Value<usize>,
    unit: Value<String>,
    timecode: Option<Value<u32>>,
) -> Value<String> {
    let mixed_text = message("creative.numeric.mixed", "Mixed");
    let suffix = unit;
    (move || {
        if mixed.get() {
            mixed_text.get()
        } else {
            format_scalar(
                value.get().clamp(min.get(), max.get()),
                precision.get(),
                &suffix.get(),
                timecode.map(|fps| fps.get()),
            )
        }
    })
    .into_value()
}
