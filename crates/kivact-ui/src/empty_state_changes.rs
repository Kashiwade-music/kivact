//! Role: Observe actual empty-message eligibility and publish accepted changes.
use kivact_core::custom::NodeRefExt;
use kivact_core::state::{Machine, Step, Value, effect, machine, on_mount};
use kivact_core::view::{IntoView, LiveRegion, Node, View, node_ref};

#[derive(Clone, PartialEq)]
enum Sent {
    Empty,
    Message(String),
}
#[derive(Clone, PartialEq)]
enum Phase {
    Mounting,
    Observing { seen: Option<String>, sent: Sent },
}
enum Observation {
    Hidden,
    Suppressed,
    Eligible(String),
}
enum Event {
    Mounted(bool, Observation),
    Observe(Observation),
}
struct Controller;
impl Machine for Controller {
    type State = Phase;
    type Event = Event;
    type Effect = String;
    fn transition(&self, state: &Phase, event: Event) -> Step<Phase, Event, String> {
        let (initial, observation, seen, sent) = match (state, event) {
            (Phase::Mounting, Event::Mounted(initial, observation)) => {
                (initial, observation, None, Sent::Empty)
            }
            (Phase::Observing { seen, sent }, Event::Observe(observation)) => {
                (false, observation, seen.clone(), sent.clone())
            }
            _ => return Step::stay(),
        };
        let (seen, message) = match observation {
            Observation::Hidden => (None, None),
            Observation::Suppressed => (seen, None),
            Observation::Eligible(message) => (seen, (!message.is_empty()).then_some(message)),
        };
        let Some(message) = message else {
            let next = Phase::Observing {
                seen,
                sent: Sent::Empty,
            };
            return if sent == Sent::Empty {
                Step::to(next)
            } else {
                Step::to(next).with(String::new())
            };
        };
        if seen.as_ref() == Some(&message) {
            return Step::stay();
        }
        if initial {
            Step::to(Phase::Observing {
                seen: Some(message),
                sent,
            })
        } else {
            let next = Phase::Observing {
                seen: Some(message.clone()),
                sent: Sent::Message(message.clone()),
            };
            if sent == Sent::Message(message.clone()) {
                Step::to(next)
            } else {
                Step::to(next).with(message)
            }
        }
    }
}
/// Attach one observer to the actual message subtree, leaving its outlet mounted.
pub(super) fn observe(
    title: Value<String>,
    description: Option<Value<String>>,
    content: Node,
) -> View {
    let notices = kivact_interaction::announcer();
    let controller = machine(
        Phase::Mounting,
        || Controller,
        move |_, message| {
            notices.announce(message, LiveRegion::Polite);
        },
    );
    let source = node_ref();
    let observation = move || {
        if !source.is_visible() {
            Observation::Hidden
        } else if source.is_in_live_region() {
            Observation::Suppressed
        } else {
            Observation::Eligible(announcement_text(
                &title.get(),
                description.map(|text| text.get()).as_deref(),
            ))
        }
    };
    let content = content.node_ref(source).on_attach(move |_| {
        let initial = kivact_core::platform::is_initial_render().get();
        on_mount(move || controller.send(Event::Mounted(initial, observation())));
        effect(move || controller.send(Event::Observe(observation())));
    });
    (content, notices.outlet()).into_view()
}
fn announcement_text(title: &str, description: Option<&str>) -> String {
    let normalize = |text: &str| text.split_whitespace().collect::<Vec<_>>().join(" ");
    let title = normalize(title);
    let description = description.map(normalize).unwrap_or_default();
    if title.is_empty() {
        description
    } else if description.is_empty() {
        title
    } else if title.ends_with(['.', '!', '?']) {
        format!("{title} {description}")
    } else {
        format!("{title}. {description}")
    }
}
