//! Role: Verify native drop delivery and caller-owned file validation.
use crate::support::mount;
use kivact::geometry::Point;
use kivact::interaction::FileInfo;
use kivact::prelude::*;
use kivact::test::raw::{RawDrop, RawInput};
use kivact::ui::dropzone;
use std::path::PathBuf;
#[test]
fn dropzone_accepts_dropped_files_and_rejects_invalid_ones() {
    let (screen, files) = mount(|| {
        let files = signal(Vec::<FileInfo>::new());
        (
            files,
            dropzone("Upload")
                .multiple()
                .validate(|f| {
                    f.path
                        .extension()
                        .filter(|ext| *ext == "txt")
                        .is_none()
                        .then(|| "Text files only".into())
                })
                .on_files(move |selected| files.set(selected))
                .style(style().width(240).height(120)),
        )
    });
    let rect = screen
        .get_by_role(Role::Group, "Upload")
        .rect()
        .expect("invariant: mounted drop target has geometry");
    let at = Some(Point {
        x: rect.x + 10.0,
        y: rect.y + 10.0,
    });
    screen.raw_input(RawInput::Drop(RawDrop::Hover {
        files: vec![PathBuf::from("a.txt")],
        at,
    }));
    screen.raw_input(RawInput::Drop(RawDrop::Drop));
    assert_eq!(files.get_untracked()[0].name(), "a.txt");
    screen.raw_input(RawInput::Drop(RawDrop::Hover {
        files: vec![PathBuf::from("b.exe")],
        at,
    }));
    screen.raw_input(RawInput::Drop(RawDrop::Drop));
    assert_eq!(files.get_untracked()[0].name(), "a.txt");
    assert!(screen.get_by_text("Text files only").is_visible());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn dropzone_picker_cancellation_and_failure_allow_retry() {
    let (screen, files) = mount(|| {
        let files = signal(Vec::<FileInfo>::new());
        (
            files,
            dropzone("Choose")
                .pick_on_press()
                .on_files(move |f| files.set(f))
                .style(style().width(240).height(120)),
        )
    });
    screen.script().push_file_dialog(None);
    screen.get_by_role(Role::Button, "Choose").click();
    assert!(files.get_untracked().is_empty());
    screen.fail_next_platform_request("Picker unavailable");
    screen.get_by_role(Role::Button, "Choose").click();
    assert!(
        screen
            .get_by_role(Role::Alert, "Picker unavailable")
            .text()
            .contains("Picker unavailable")
    );
    screen
        .script()
        .push_file_dialog(Some(PathBuf::from("retry.txt")));
    screen.get_by_role(Role::Button, "Choose").click();
    assert_eq!(files.get_untracked()[0].name(), "retry.txt");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
#[test]
fn dropzone_single_file_targets_do_not_publish_multiple_files() {
    let (screen, files) = mount(|| {
        let files = signal(Vec::<FileInfo>::new());
        (
            files,
            dropzone("Single")
                .on_files(move |f| files.set(f))
                .style(style().width(240).height(120)),
        )
    });
    let r = screen
        .get_by_role(Role::Group, "Single")
        .rect()
        .expect("invariant: drop target has geometry");
    screen.raw_input(RawInput::Drop(RawDrop::Hover {
        files: vec![PathBuf::from("a.txt"), PathBuf::from("b.txt")],
        at: Some(Point {
            x: r.x + 10.0,
            y: r.y + 10.0,
        }),
    }));
    screen.raw_input(RawInput::Drop(RawDrop::Drop));
    assert!(files.get_untracked().is_empty());
    assert!(screen.get_by_text("Choose one file.").is_visible());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
