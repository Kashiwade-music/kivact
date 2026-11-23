//! Role: The messaging page: a conversation with a student, its quick replies and a
//! thread list.

use super::common::*;
use catalog_shell::{Example, Page};
use kivact::prelude::*;
use kivact::state::{Machine, MachineHandle, Step, machine};
use kivact::style::linear_gradient;
use kivact::view::Node;
use kivact_ui_pop as ui;
use std::time::Duration;
/// The page of the messaging components.
pub(crate) fn page() -> Page {
    Page::new(
        "messaging",
        "Messaging",
        "Components",
        include_str!("messaging.rs"),
    )
    .description("A conversation with quick replies, a live thread and a sidebar of threads.")
    .example(Example::new(
        "messaging",
        "Conversation",
        "Send a message: the reply arrives after a short typing indicator.",
        messaging_demo,
    ))
}

#[derive(Clone, PartialEq)]
struct ChatEntry {
    id: u64,
    mine: bool,
    content: ChatContent,
}

#[derive(Clone, PartialEq)]
enum ChatContent {
    Message(String),
    Typing,
}

/// The pending replies: none, or one being answered and the ones sent after it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
enum Reply {
    #[default]
    Idle,
    Busy {
        /// The sent message being answered.
        current: u64,
        /// Whether its typing placeholder is shown.
        typing: bool,
        /// Messages sent meanwhile, answered in order.
        queued: Vec<u64>,
    },
}

enum ReplyEvent {
    Sent { id: u64 },
    ShowTyping,
    Arrive,
}

/// What the reply machine asks the page to change in the message list.
enum ReplyEffect {
    ShowTyping { id: u64 },
    Arrived { id: u64 },
}

struct ReplyMachine;

impl Machine for ReplyMachine {
    type State = Reply;
    type Event = ReplyEvent;
    type Effect = ReplyEffect;
    fn transition(&self, state: &Reply, event: ReplyEvent) -> Step<Reply, ReplyEvent, ReplyEffect> {
        match (state, event) {
            (Reply::Idle, ReplyEvent::Sent { id }) => Step::to(Reply::Busy {
                current: id,
                typing: false,
                queued: Vec::new(),
            })
            .after(Duration::from_millis(400), ReplyEvent::ShowTyping),
            (
                Reply::Busy {
                    current,
                    typing,
                    queued,
                },
                ReplyEvent::Sent { id },
            ) => {
                let mut queued = queued.clone();
                queued.push(id);
                Step::to(Reply::Busy {
                    current: *current,
                    typing: *typing,
                    queued,
                })
            }
            (
                Reply::Busy {
                    current,
                    typing: false,
                    queued,
                },
                ReplyEvent::ShowTyping,
            ) => Step::to(Reply::Busy {
                current: *current,
                typing: true,
                queued: queued.clone(),
            })
            .with(ReplyEffect::ShowTyping { id: *current })
            .after(Duration::from_millis(1300), ReplyEvent::Arrive),
            (
                Reply::Busy {
                    current,
                    typing: true,
                    queued,
                },
                ReplyEvent::Arrive,
            ) => {
                let arrived = ReplyEffect::Arrived { id: *current };
                match queued.split_first() {
                    Some((next, rest)) => Step::to(Reply::Busy {
                        current: *next,
                        typing: false,
                        queued: rest.to_vec(),
                    })
                    .with(arrived)
                    .after(Duration::from_millis(400), ReplyEvent::ShowTyping),
                    None => Step::to(Reply::Idle).with(arrived),
                }
            }
            _ => Step::stay(),
        }
    }
}

const ANSWERS: [&str; 4] = [
    "Got it! I will check right away.",
    "Thank you, teacher ♪",
    "Heh... I will do my best!",
    "Understood. Is there anything else I can help with?",
];

fn messaging_demo() -> impl IntoView {
    // example: messaging
    let active = signal(0usize);
    let draft = signal(String::new());
    let messages = signal(Vec::<ChatEntry>::new());
    let unread = signal([2usize, 1, 0, 0]);
    let reply = machine(
        Reply::Idle,
        || ReplyMachine,
        move |_, effect| match effect {
            ReplyEffect::ShowTyping { id } => messages.update(|entries| {
                entries.push(ChatEntry {
                    id: id + 1,
                    mine: false,
                    content: ChatContent::Typing,
                })
            }),
            ReplyEffect::Arrived { id } => messages.update(|entries| {
                entries.retain(|entry| entry.id != id + 1);
                entries.push(ChatEntry {
                    id: id + 2,
                    mine: false,
                    content: ChatContent::Message(
                        ANSWERS[((id / 3) as usize) % ANSWERS.len()].into(),
                    ),
                })
            }),
        },
    );
    let chat_scroll = node_ref();
    let send = callback(move |s: String| send_message(s, reply, messages, draft));
    let list = thread_sidebar(active, unread);
    let body = chat_body(messages, chat_scroll, send);
    let main = chat_main(active, draft, send, body);
    section(
        "10",
        "Messaging",
        "Chat: conversations with students",
        kivact::ui::card(row((list, main)).style(style().height(560)))
            .style(style().padding(0).overflow(kivact::style::Overflow::Hidden)),
    )
    // end example
}

fn send_message(
    s: String,
    reply: MachineHandle<ReplyMachine>,
    messages: kivact::state::Signal<Vec<ChatEntry>>,
    draft: kivact::state::Signal<String>,
) {
    if !s.trim().is_empty() {
        let sent = messages
            .get_untracked()
            .iter()
            .filter(|entry| entry.mine)
            .count() as u64;
        let id = sent * 3;
        messages.update(|entries| {
            entries.push(ChatEntry {
                id,
                mine: true,
                content: ChatContent::Message(s),
            })
        });
        draft.set(String::new());
        reply.send(ReplyEvent::Sent { id });
    }
}

fn thread_sidebar(
    active: kivact::state::Signal<usize>,
    unread: kivact::state::Signal<[usize; 4]>,
) -> Node {
    let threads = [
        (
            "MN",
            "Minato Sorasaki",
            "I left the report on your desk!",
            2,
        ),
        ("HK", "Haruka Kasuga", "Please check the clinic shift", 1),
        (
            "YZ",
            "Yuzu Hanaoka",
            "Come hang out at the Game Dev Club",
            0,
        ),
        ("AR", "Arisa Shiraishi", "Sent a sticker", 0),
    ]
    .into_iter()
    .enumerate()
    .map(|(i, (initials, name, preview, _))| {
        ui::thread(initials, name)
            .preview(preview)
            .unread(move || unread.with(|counts| counts[i]))
            .with_node(|node| node.selected(move || active.get() == i))
            .portrait({
                decorated_avatar(
                    kivact::state::IntoText::into_text_value(initials),
                    AvatarLook::default()
                        .size(32)
                        .hue([205.0, 330.0, 45.0, 160.0][i]),
                )
            })
            .test_id(format!("pop-thread-{i}"))
            .on_press(move || {
                active.set(i);
                unread.update(|counts| counts[i] = 0);
            })
    })
    .collect::<Vec<_>>();
    column((
        row((
            text("Chats").style(
                style()
                    .flex_grow(1)
                    .font_size(16)
                    .font_weight(600)
                    .color(Color::WHITE),
            ),
            kivact::ui::button("New chat")
                .variant(kivact::ui::ButtonVariant::Ghost)
                .hide_label(true)
                .icon(kivact::view::icon("pencil"))
                .style(style().color(Color::WHITE)),
        ))
        .style(
            style()
                .height(56)
                .padding_x(18)
                .align_items(Align::Center)
                .background(linear_gradient(90).colors([rgb(0xf2559a), rgb(0xff7cb0)]))
                .color(Color::WHITE),
        ),
        column(threads).style(style().padding(8)),
    ))
    .style(
        style()
            .width(280)
            .flex_shrink(0)
            .background(color::SURFACE_ALT)
            .border_right(1, color::BORDER),
    )
}

fn live_messages(messages: kivact::state::Signal<Vec<ChatEntry>>) -> impl IntoView {
    for_each(
        messages,
        |entry| entry.id,
        move |entry| {
            node().child(dynamic(entry, |entry| match entry.content {
                ChatContent::Typing => {
                    let initials = kivact::state::IntoText::into_text_value("M");
                    kivact::ui::typing_indicator()
                        .avatar(decorated_avatar(initials, AvatarLook::default().size(24)))
                        .into_view()
                }
                ChatContent::Message(content) => {
                    let mine: kivact::state::Value<bool> =
                        kivact::state::IntoValue::into_value(entry.mine);
                    let initials = kivact::state::IntoText::into_text_value("M");
                    kivact::ui::chat_message(initials)
                        .sender(move || {
                            if mine.get() {
                                kivact::ui::ChatSender::User
                            } else {
                                kivact::ui::ChatSender::Assistant
                            }
                        })
                        .content(kivact::view::text(content))
                        .time(if entry.mine { "Read 09:16" } else { "09:16" })
                        .variant(ui::ChatBubbleVariant)
                        .avatar(decorated_avatar(initials, AvatarLook::default().size(24)))
                        .into_view()
                }
            }))
        },
    )
}

fn quick_replies(
    messages: kivact::state::Signal<Vec<ChatEntry>>,
    send: kivact::state::Callback<String>,
) -> impl IntoView {
    show(
        move || messages.with(|entries| !entries.iter().any(|entry| entry.mine)),
        move || {
            column((
                text("Choose a reply")
                    .style(style().font_size(12).font_weight(500).padding_left(4))
                    .style(style().color(ui::Hue::Pink.colors().1)),
                kivact::ui::button("Sure, let's eat together")
                    .variant(kivact::ui::ButtonVariant::Secondary)
                    .style(style().justify_content(Justify::Start))
                    .on_press(move || {
                        send.call("Sure, let's eat together".into());
                    }),
                kivact::ui::button("Sorry, I have a meeting")
                    .variant(kivact::ui::ButtonVariant::Secondary)
                    .style(style().justify_content(Justify::Start))
                    .on_press(move || {
                        send.call("Sorry, I have a meeting".into());
                    }),
            ))
            .test_id("pop-reply-card")
            .style(
                style()
                    .gap(6)
                    .padding(10)
                    .width(300)
                    .max_width(pct(90))
                    .align_self(Align::End)
                    .border_radius(8),
            )
            .style(move || {
                let t = &use_theme().get();
                style()
                    .background(ui::Hue::Pink.colors().0)
                    .border(1, ui::Hue::Pink.accent().mix(Color::TRANSPARENT, 0.7, t))
            })
        },
    )
}

fn morning_report() -> impl IntoView {
    {
        let mine: kivact::state::Value<bool> = kivact::state::IntoValue::into_value(false);
        let initials = kivact::state::IntoText::into_text_value("M");
        let time = kivact::state::IntoText::into_text_value("09:12");
        let paragraphs: kivact::state::Value<Vec<String>> =
            kivact::state::IntoValue::into_value(vec![
                "Good morning, teacher.".into(),
                "I left the chapter 3 report on your desk!".into(),
            ]);
        kivact::view::column(kivact::view::dynamic(paragraphs, move |paragraphs| {
            let count = paragraphs.len();
            paragraphs
                .into_iter()
                .enumerate()
                .map(move |(index, paragraph)| {
                    let group = if count == 1 {
                        kivact::ui::ChatBubbleGroup::Single
                    } else if index == 0 {
                        kivact::ui::ChatBubbleGroup::First
                    } else if index + 1 == count {
                        kivact::ui::ChatBubbleGroup::Last
                    } else {
                        kivact::ui::ChatBubbleGroup::Middle
                    };
                    let mut message = kivact::ui::chat_message(initials)
                        .sender(move || {
                            if mine.get() {
                                kivact::ui::ChatSender::User
                            } else {
                                kivact::ui::ChatSender::Assistant
                            }
                        })
                        .content(kivact::view::text(paragraph))
                        .group(group)
                        .variant(ui::ChatBubbleVariant);
                    if index == 0 {
                        message = message
                            .avatar(decorated_avatar(initials, AvatarLook::default().size(24)));
                    }
                    if index + 1 == count {
                        message = message.time(time);
                    }
                    message
                })
                .collect::<Vec<_>>()
        }))
        .style(style().gap(4))
    }
}

fn chat_body(
    messages: kivact::state::Signal<Vec<ChatEntry>>,
    chat_scroll: kivact::view::NodeRef,
    send: kivact::state::Callback<String>,
) -> kivact::ui::ChatThread {
    kivact::ui::chat_thread(
        column((
            text("Today").style(
                style()
                    .font_size(12)
                    .align_self(Align::Center)
                    .padding_x(10)
                    .padding_y(2)
                    .background(color::SURFACE_ALT)
                    .border_radius(999)
                    .color(color::TEXT_MUTED),
            ),
            morning_report(),
            {
                let mine: kivact::state::Value<bool> = kivact::state::IntoValue::into_value(true);
                let initials = kivact::state::IntoText::into_text_value("");
                kivact::ui::chat_message(initials)
                    .sender(move || {
                        if mine.get() {
                            kivact::ui::ChatSender::User
                        } else {
                            kivact::ui::ChatSender::Assistant
                        }
                    })
                    .content(kivact::view::text(
                        "Thanks, that helps. I will read it later.",
                    ))
                    .time("Read 09:14")
                    .variant(ui::ChatBubbleVariant)
                    .avatar(decorated_avatar(initials, AvatarLook::default().size(24)))
            },
            {
                let mine: kivact::state::Value<bool> = kivact::state::IntoValue::into_value(false);
                let initials = kivact::state::IntoText::into_text_value("M");
                kivact::ui::chat_message(initials)
                    .sender(move || {
                        if mine.get() {
                            kivact::ui::ChatSender::User
                        } else {
                            kivact::ui::ChatSender::Assistant
                        }
                    })
                    .content(kivact::view::text(
                        "Also, lunch today... do you have a little time?",
                    ))
                    .time("09:15")
                    .variant(ui::ChatBubbleVariant)
                    .avatar(decorated_avatar(initials, AvatarLook::default().size(24)))
            },
            quick_replies(messages, send),
            live_messages(messages),
        ))
        .style(style().gap(14).padding_x(20).padding_y(18)),
    )
    .viewport_ref(chat_scroll)
    .style(style().flex_grow(1).min_height(0))
    .draw(|d, s| {
        for x in (0..s.width as usize).step_by(18) {
            for y in (0..s.height as usize).step_by(18) {
                d.rounded_rect(Rect::new(x as f64, y as f64, 2.0, 2.0), 1.0, color::BORDER);
            }
        }
    })
}

fn chat_main(
    active: kivact::state::Signal<usize>,
    draft: kivact::state::Signal<String>,
    send: kivact::state::Callback<String>,
    body: impl IntoView,
) -> Node {
    let names = [
        "Minato Sorasaki",
        "Haruka Kasuga",
        "Yuzu Hanaoka",
        "Arisa Shiraishi",
    ];
    column((
        row((
            decorated_avatar(
                kivact::state::IntoText::into_text_value("MN"),
                AvatarLook::default().size(32).halo(true).online(true),
            ),
            column((
                text(move || names[active.get()].to_string()).style(style().font_weight(600)),
                text("Investigation Club · online")
                    .style(style().font_size(12).color(rgb(0x0f8a6a))),
            ))
            .style(style().flex_grow(1)),
            kivact::ui::button("Search chats")
                .variant(kivact::ui::ButtonVariant::Ghost)
                .hide_label(true)
                .icon(kivact::view::icon("search")),
            kivact::ui::button("Chat actions")
                .variant(kivact::ui::ButtonVariant::Ghost)
                .hide_label(true)
                .icon(kivact::view::icon("dots-vertical")),
        ))
        .style(
            style()
                .height(56)
                .padding_x(18)
                .gap(12)
                .align_items(Align::Center)
                .border_bottom(1, color::BORDER),
        ),
        body,
        kivact::ui::chat_composer(draft)
            .placeholder("Type a message…")
            .on_submit(move |message| send.call(message))
            .header_actions(
                kivact::ui::button("Attach")
                    .variant(kivact::ui::ButtonVariant::Ghost)
                    .hide_label(true)
                    .icon(kivact::view::icon("paperclip")),
            )
            .footer_actions(
                kivact::ui::button("Sticker")
                    .variant(kivact::ui::ButtonVariant::Ghost)
                    .hide_label(true)
                    .icon(kivact::view::icon("mood-smile"))
                    .on_press(move || send.call("★".into())),
            )
            .send_icon(kivact::view::icon("send")),
    ))
    .style(style().flex_grow(1).min_width(0))
}
