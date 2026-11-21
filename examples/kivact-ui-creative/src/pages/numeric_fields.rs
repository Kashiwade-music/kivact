//! Role: The page of the Creative catalog.

use super::common::*;
use super::controls::{num, slide};
use super::numeric_samples::*;
use catalog_shell::{Example, Page};
use kivact::prelude::{Element, IntoView, column, row, signal};
use kivact::style::{pct, style};
use kivact_ui_creative as ui;

pub(crate) fn page() -> Page {
    Page::new(
        "numeric-fields",
        "Numeric Fields",
        "Controls",
        include_str!("numeric_fields.rs"),
    )
    .description("Drag values, sliders, vectors, timecode and keyframe state.")
    .example(Example::new("numeric-fields", "Numeric Fields", "", demo))
}

// example: numeric-fields
fn demo() -> impl IntoView {
    let transport = transport();
    specs((
        spec("Drag Number", "Drag to edit", col((
            num("Focal Length", 50.0, 1.0, 5000.0, 0, " mm"),
            num("Exposure", 0.0, -10.0, 10.0, 3, " EV"),
            num("Samples", 128.0, 1.0, 4096.0, 0, ""),
            num("Frame Rate", 24.0, 1.0, 240.0, 3, " fps").label_visible(false).step(0.001),
            note("Drag left or right; Ctrl for fine, Shift for coarse; click to type (expressions allowed: 50*2); Backspace while hovering restores the default; Ctrl+wheel steps."),
        ))).style(style().min_height(258)),
        spec("Slider Field", "Sliders", col((
            ui::value_slider("Opacity", signal(100.0)).range(0.0..=100.0).precision(1).step(0.1).unit(" %").reset_to(100.0).style(style().width(pct(100))),
            slide("Roughness", 0.5).step(0.001),
            slide("Strength", 0.82).step(0.001).reset_to(1.0),
            slide("Disabled", 0.3).precision(2).disabled(true),
        ))),
        spec("Vector (Column)", "Vector Input", col((
            ui::property_row("Location X", column((
                axis_num("X",0.0," m",3,0xff5c6c).label_visible(false),
                axis_num("Y",1.25," m",3,0x34d399),
                axis_num("Z",0.5," m",3,0x5aa9ff),
            )).style(style().gap(1))),
            ui::property_row("Rotation", row((
                axis_num("Rotation X",90.0,"°",1,0xff5c6c).label_visible(false).style(style().width(0).flex_grow(1)),
                axis_num("Rotation Y",0.0,"°",1,0x34d399).label_visible(false).style(style().width(0).flex_grow(1)),
                axis_num("Rotation Z",-35.0,"°",1,0x5aa9ff).label_visible(false).style(style().width(0).flex_grow(1)),
            )).style(style().gap(1))),
        )).role(kivact::view::Role::List).style(style().gap(3))),
        spec("Animatable Property", "Keyframe State", col((
            animation_property("Scale",1.0,ui::PropertyState::Plain),
            animation_property("Keyed",0.75,ui::PropertyState::Keyed),
            animation_property("Animated",0.4,ui::PropertyState::Animated),
            animation_property("Driven",2.5,ui::PropertyState::Driven),
            note("Click ◇ to insert a key (yellow); click again to remove it. Unkeyed frames are green; driver-driven ones are purple."),
        )).style(style().gap(3))),
        spec("Timecode Field", "Timecode", col((
            ui::property_row("Start", tc("Start timecode", signal(0.0))),
            ui::property_row("End", tc("End timecode", signal(5760.0))),
            ui::property_row("Duration", tc("Duration timecode", signal(86.0))),
            ui::property_row("Current", tc("Current timecode", bind(move || transport.get().frame, move |frame| transport.update(|state| state.frame = frame)))),
            note("Drag to scrub by frame; click to type HH:MM:SS:FF. Relative input such as +12 also works."),
        )).role(kivact::view::Role::List).style(style().gap(3))),
        spec("Inline Pairs", "Side by Side", col((
            ui::property_row("Resolution", column((
                num("X",1920.0,4.0,65536.0,0," px").style(style().border_radius(3)),
                num("Y",1080.0,4.0,65536.0,0," px").style(style().border_radius(3)),
                ui::value_slider("%", signal(100.0)).range(1.0..=100.0).precision(0).step(1.0).unit(" %").style(style().width(pct(100)).border_radius(3)),
            )).style(style().gap(1))),
            ui::property_row("Frame Range", row((
                num("Range start",1.0,0.0,65536.0,0,"").label_visible(false).style(style().width(0).flex_grow(1)),
                num("Range end",250.0,0.0,65536.0,0,"").label_visible(false).style(style().width(0).flex_grow(1)),
            )).style(style().gap(1))),
            ui::property_row("Tempo", num("BPM",120.0,20.0,300.0,3,"")),
        )).role(kivact::view::Role::List).style(style().gap(3))).style(style().min_height(190)),
    ))
}
// end example
