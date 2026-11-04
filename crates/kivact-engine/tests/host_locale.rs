//! Role: Tests that the system locale of the window reaches the client.
//!
//! The system locale of the window reaches the client: in `HostInfo` at
//! start and in `EnvChanged` when it changes.

use std::cell::RefCell;
use std::rc::Rc;

use kivact_engine::api::Client;
use kivact_engine::testing::{ArvaneTestHost, TestHostOptions, WindowMode};

use kivact_engine::contract::{HostEvent, TurnOutput};

/// What the client was told about the locale.
#[derive(Default)]
struct Told {
    at_start: Option<Option<String>>,
    changes: Vec<Option<String>>,
}

fn start(locale: Option<&str>) -> (ArvaneTestHost, Rc<RefCell<Told>>) {
    let told = Rc::new(RefCell::new(Told::default()));
    let host = ArvaneTestHost::new(
        Client::new(
            {
                let told = told.clone();
                move |info, _, _| {
                    told.borrow_mut().at_start = Some(info.env.locale.clone());
                }
            },
            {
                let told = told.clone();
                move |events, _, _| {
                    for event in events {
                        if let HostEvent::EnvChanged {
                            environment: env, ..
                        } = event
                        {
                            told.borrow_mut().changes.push(env.locale.clone());
                        }
                    }
                    TurnOutput::default()
                }
            },
        ),
        TestHostOptions {
            window: WindowMode::Headless,
            locale: locale.map(str::to_owned),
            ..TestHostOptions::default()
        },
    )
    .expect("the test host starts");
    (host, told)
}

#[test]
fn the_windows_locale_is_in_host_info_at_start() {
    let (_host, told) = start(Some("ja-JP"));
    assert_eq!(told.borrow().at_start, Some(Some("ja-JP".to_owned())));
    let (_host, told) = start(None);
    assert_eq!(
        told.borrow().at_start,
        Some(None),
        "a window that names none"
    );
}

#[test]
fn a_change_of_the_locale_is_reported_once_and_only_when_it_changed() {
    let (mut host, told) = start(Some("en-US"));
    host.set_locale(Some("fr-FR"));
    host.wake();
    assert_eq!(told.borrow().changes, [Some("fr-FR".to_owned())]);

    // The same locale again changes nothing.
    host.set_locale(Some("fr-FR"));
    host.wake();
    assert_eq!(told.borrow().changes.len(), 1);

    host.set_locale(None);
    host.wake();
    assert_eq!(told.borrow().changes, [Some("fr-FR".to_owned()), None]);
}
