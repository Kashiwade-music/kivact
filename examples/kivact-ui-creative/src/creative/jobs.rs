//! Role: Job and task management for creative application.

use super::common::{TextRow, bind, col, line, note, spec, specs, text_table};
use kivact::draw::{LineCap, LineJoin, Stroke};
use kivact::geometry::Point;
use kivact::geometry::Rect;
use kivact::interaction::toaster;
use kivact::motion::FrameRate;
use kivact::motion::frame_loop;
use kivact::prelude::{
    IntoValue, IntoView, Node, NodeBuilder, Signal, effect, node, row, signal, spacer, text,
};
use kivact::style::GridPlacement;
use kivact::style::style;
use kivact_ui_creative as ui;

#[derive(Clone, PartialEq)]
struct LogLine {
    level: usize,
    text: String,
}

pub(super) fn jobs(status: Signal<String>) -> impl IntoView {
    let running = signal(false);
    let progress = signal(vec![1.0_f64, 0.0, 0.0, 0.0]);
    let rows = signal(vec![
        TextRow::new(
            1,
            [
                "1",
                "▧ Shot_01…",
                "ProRes 422 HQ",
                "240",
                "Done",
                "100%",
                "×",
            ],
        ),
        TextRow::new(
            2,
            [
                "2",
                "▧ Shot_02…",
                "EXR Seq (DWAA)",
                "120",
                "Queued",
                "0%",
                "×",
            ],
        ),
        TextRow::new(
            3,
            [
                "3",
                "▧ Trailer…",
                "H.265 10-bit",
                "1440",
                "Queued",
                "0%",
                "×",
            ],
        ),
        TextRow::new(
            4,
            ["4", "♫ Stems…", "WAV 48k / 24", "1", "Queued", "0%", "×"],
        ),
    ]);
    let log=signal("04:00:49  ›  Accent: lime\n04:00:50  ›  Compact rows (20 px)\n04:01:02  ›  Accent: iris\n04:01:03  ›  Regular rows (24 px)\n04:01:22  ›  Accent: lime\n04:01:23  ›  Compact rows (20 px)\n04:01:25  ›  Accent: iris\n04:01:26  ›  Regular rows (24 px)".split('\n').map(|text|LogLine{level:0,text:text.to_owned()}).collect::<Vec<_>>());
    let severity = signal(vec![true; 4]);
    let amount = signal(0.0_f64);
    let clock = frame_loop(move |tick| {
        amount.update(|v| *v = (*v + tick.delta.as_secs_f64() * 0.12).min(1.0));
        progress.update(|p| {
            if let Some((_, v)) = p.iter_mut().enumerate().find(|(index, value)| {
                **value < 1.0
                    && rows
                        .with_untracked(|rows| rows.iter().any(|row| row.id == *index as u64 + 1))
            }) {
                *v = (*v + tick.delta.as_secs_f64() * 0.2).min(1.0);
            }
        });
        if rows.with_untracked(|rows| {
            rows.iter()
                .all(|row| progress.with_untracked(|p| p[row.id as usize - 1] >= 1.0))
        }) {
            running.set(false);
            toaster().push(
                kivact::interaction::Toast::new("Catalog report")
                    .tone(kivact::interaction::Tone::Info)
                    .body("UI operation completed."),
            );
            status.set("Render complete · four catalog jobs".into());
            log.update(|v| {
                v.push(LogLine {
                    level: 1,
                    text: "04:01:30  INFO  Render complete".into(),
                })
            });
        }
    });
    effect(move || {
        if running.get() {
            clock.start(FrameRate::Hz(24.0));
        } else {
            clock.stop();
        }
    });
    specs((
        col((
            row((text("Render Queue").style(style().weight(600)),kivact::ui::button("Render All").icon(kivact::view::icon("play")).variant(kivact::ui::ButtonVariant::Primary).size(kivact::ui::ControlSize::Small).test_id("creative-start-job").on_press(move||running.set(true)),kivact::ui::button("Stop").variant(kivact::ui::ButtonVariant::Secondary).icon(kivact::view::icon("stop")).size(kivact::ui::ControlSize::Small).on_press(move||{running.set(false);status.set("Render cancelled".into());}),spacer(),note(move||format!("{}/{} done",rows.with(|rows|rows.iter().filter(|row|progress.with(|p|p[row.id as usize-1]>=1.0)).count()),rows.with(Vec::len))))).style(style().height(40).padding_x(10).gap(8)),
            text_table(&[("#",30.0),("Job",94.0),("Preset",145.0),("Frames",58.0),("Status",92.0),("Progress",165.0),("",24.0)],rows,signal(Vec::new()),signal(None)).row_height(30.0).cell_controls(move|item, col|{
                let id=item.get_untracked().id;let i=id as usize-1;
                if col==4{return text(move||(if progress.get()[i]>=1.0 {"Done"}else if running.get()&&progress.get()[i]>0.0 {"Rendering"}else{"Queued"}).to_owned()).style(style().font_size(9.5).weight(600)).style( style().color(ui::ColorName::Success)).into_view();}
                if col==5{return kivact::ui::progress(move||Some(progress.get()[i])).name(format!("Job {id} progress")).style(style().height(18)).into_view();}
                if col==6{return kivact::ui::button(format!("Remove job {id}")).variant(kivact::ui::ButtonVariant::Secondary).hide_label(true).icon(kivact::view::icon("x")).style(style().size(18,18).padding(0)).on_press(move||rows.update(|v|v.retain(|r|r.id!=id))).into_view();}
                text(move||item.get().cells[col].clone()).style(style().font_size(10.5).font_family(if col==3{"Geist Mono"}else{"Geist"})).into_view()
            }),
        )).style(style().grid_column(GridPlacement::span(2)).height(210).gap(0)),
        spec("Progress","進捗",col((
            kivact::ui::progress(Some(0.64)).name("Render progress"),kivact::ui::progress(Some(0.32)).name("Proxy progress"),
            kivact::ui::progress(move||Some(amount.get())).name("Determinate progress").style(style().height(18)),
            kivact::ui::progress(move||Some(if running.get(){amount.get()}else{0.21})).name("Background task").style(style().height(18)),
            node().style(style().height(22)).draw(move|d,s|{let p=d.theme().ext::<ui::Tokens>();for i in 0..56 {d.rounded_rect(Rect::new(i as f64*s.width/56.0,5.0,3.0,12.0),1.5,if i<14{p.success}else{p.strong});}}),
            row((job_ring(0.24),job_ring(0.0),job_ring(amount))).style(style().gap(12)),
        ))).style(style().height(210)),
        col((
            row((text("Info").style(style().weight(600)),row((0..4).map(move|i|kivact::ui::toggle_button(["Operator","Info","Warning","Error"][i], bind(move||severity.get()[i],move|on|severity.update(|v|v[i]=on))).test_id(format!("creative-log-filter-{i}")).style(style().height(20).padding_x(8).font_size(10))).collect::<Vec<_>>()).style(style().gap(2)),spacer(),kivact::ui::button("Clear").variant(kivact::ui::ButtonVariant::Secondary).icon(kivact::view::icon("trash")).size(kivact::ui::ControlSize::Small).on_press(move||log.set(Vec::new())))).style(style().height(40).padding_x(10).gap(8)),
            kivact::ui::text_area("Job log", bind(move||{log.with(|lines|lines.iter().filter(|line|severity.get()[line.level]).map(|line|line.text.as_str()).collect::<Vec<_>>().join("\n"))},|_|{})).read_only(true).style(style().height(170)),
        )).style(style().grid_column(GridPlacement::span(2)).height(210).gap(0)),
        spec("Toast / Report","通知",col((
            line([("Info",kivact::interaction::Tone::Info),("Success",kivact::interaction::Tone::Success),("Warning",kivact::interaction::Tone::Warning),("Error",kivact::interaction::Tone::Danger)].into_iter().map(move|(label,tone)|kivact::ui::button(label).variant(kivact::ui::ButtonVariant::Secondary).test_id(format!("creative-log-report-{label}")).icon(kivact::view::icon("info")).on_press(move||{toaster().push(kivact::interaction::Toast::new("Catalog report").tone(tone).body("UI operation completed."));status.set(format!("{label} · Catalog report"));log.update(|lines|lines.push(LogLine{level:match label{"Info"=>1,"Warning"=>2,"Error"=>3,_=>0},text:format!("04:01:30  {}  Catalog report",label.to_uppercase())}));})).collect::<Vec<_>>()),
            note("トーストと同時にステータスバーにもレポートを表示し、インフォログへ記録します。"),
        ))).style(style().height(162)),
    ))
}
fn job_ring(value: impl IntoValue<f64>) -> Node {
    let value = value.into_value();
    node().style(style().size(38, 38)).draw(move |d, s| {
        let p = d.theme().ext::<ui::Tokens>();
        let center = Point::new(s.width / 2.0, s.height / 2.0);
        let mut path = kivact::draw::Path::new();
        for i in 0..=50 {
            let a = i as f64 / 50.0 * std::f64::consts::TAU;
            let pt = Point::new(center.x + a.sin() * 16.0, center.y - a.cos() * 16.0);
            path = if i == 0 {
                path.move_to(pt)
            } else {
                path.line_to(pt)
            };
        }
        d.stroke_path(
            &path,
            &Stroke::new(3.0).cap(LineCap::Round).join(LineJoin::Round),
            p.strong,
        );
        let v = value.get();
        let mut path = kivact::draw::Path::new();
        for i in 0..=50 {
            let a = i as f64 / 50.0 * std::f64::consts::TAU * v;
            let pt = Point::new(center.x + a.sin() * 16.0, center.y - a.cos() * 16.0);
            path = if i == 0 {
                path.move_to(pt)
            } else {
                path.line_to(pt)
            };
        }
        d.stroke_path(
            &path,
            &Stroke::new(3.0).cap(LineCap::Round).join(LineJoin::Round),
            p.accent,
        );
    })
}
