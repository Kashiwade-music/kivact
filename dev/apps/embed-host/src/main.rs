//! Role: Host for manual embedded editor input and accessibility checks.
#[cfg(any(windows, target_os = "linux"))]
use kivact::prelude::*;
#[cfg(any(windows, target_os = "linux"))]
use std::time::{Duration, Instant};

#[cfg(any(windows, target_os = "linux"))]
fn editor() -> impl IntoView {
    let name = signal(String::new());
    let clicks = signal(0);
    let gain = signal(0.5);
    column((
        text_input("名前 / Name", name),
        button("増やす / Increment").on_press(move || clicks.update(|n| *n += 1)),
        text(move || format!("Count: {}", clicks.get())),
        slider("音量 / Gain", gain).range(0.0..=1.0),
    ))
    .style(style().padding(16).gap(12))
}

#[cfg(any(windows, target_os = "linux"))]
fn main() -> std::result::Result<(), String> {
    let mut scale = 1.0f64;
    let mut two = false;
    let mut smoke = false;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--scale" => {
                scale = args
                    .next()
                    .ok_or("--scale needs a number")?
                    .parse()
                    .map_err(|e: std::num::ParseFloatError| e.to_string())?
            }
            "--two" => two = true,
            "--smoke" => smoke = true,
            _ => return Err(format!("unknown argument: {arg}")),
        }
    }
    if !scale.is_finite() || scale <= 0.0 || !(scale as f32).is_finite() || scale as f32 == 0.0 {
        return Err("--scale must be finite and positive in f32".into());
    }
    let host = kivact::test::TestParent::open(800, 600).map_err(|e| e.to_string())?;
    let width = if two { 400 } else { 800 };
    let mut editors = Vec::new();
    for index in 0..if two { 2 } else { 1 } {
        let parent = if two {
            host.parent_in(index * width, 0, width, 600)
                .map_err(|e| e.to_string())?
        } else {
            host.parent()
        };
        editors.push(
            App::new("Embedded editor")
                .window_size(f64::from(width) / scale, 600.0 / scale)
                .embed(parent.scale(scale), editor)
                .map_err(|e| e.to_string())?,
        );
    }
    let started = Instant::now();
    while !host.is_closed() {
        host.pump(Duration::from_millis(16));
        if smoke && started.elapsed() >= Duration::from_secs(5) {
            host.close();
        }
    }
    drop(editors);
    Ok(())
}

#[cfg(not(any(windows, target_os = "linux")))]
fn main() -> std::result::Result<(), String> {
    Err("the embedded editor host supports Windows and Linux X11".into())
}
