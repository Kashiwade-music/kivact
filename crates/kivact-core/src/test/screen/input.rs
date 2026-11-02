//! Role: Input events and device handling.
//!
//! User input as real shell events at the host's entry, as the shell
//! delivers the winit events it maps: pointer moves and buttons go through
//! the host's hit testing, keys through its key mapping and text editing,
//! IME composition through its IME path, wheels through its scroll
//! routing, focus through the accessibility `Focus` action, touch and pen
//! contacts (`contact_*`, `tap`) as the platform's contacts.
//!
//! Element actions check actionability first, like Playwright: a click
//! lands at the centre of the element's box only if the element is
//! mounted, shown, has a size, is enabled, lies in the window and is what
//! the hit test finds there (not covered by another node or a modal's
//! backdrop); otherwise the test fails with the reason.
//! [`Element::force_click`] and the screen's pointer methods skip the
//! checks (a click that should do nothing, a press outside a popup).
//!
//! - `click` moves the pointer to the centre of the element's box (the
//!   point mapped by its transforms) and presses and releases there,
//!   through the host's hit test; `drag` takes points in the element's
//!   untransformed coordinates. `force_click`, `click_at` and `mouse_*`
//!   check nothing.
//! - `focus` is the accessibility `Focus` action (failing when it does not
//!   take; a node without a role cannot be focused by assistive
//!   technology). `type_text` is a sequence of keys (an IME commit while
//!   composing), `compose` a preedit, `commit` a commit, and `reconvert`
//!   the range an IME takes back (`ime`). `Screen::ime_text` is what the
//!   host answers the IME, `Screen::ime_cursor_area` where the window puts
//!   the candidate window, `Screen::cursor` the mouse cursor shown.
//! - `Element::scroll` moves to the element's centre and wheels;
//!   `Screen::scroll` wheels at the current pointer (unchecked). Tab and
//!   modifiers are keys with modifiers.
//! - Touch and pen: `Element::tap` and `tap_with(PointerKind)` check
//!   actionability; `Screen::contact_down(kind, x, y)`,
//!   `contact_move(contact, x, y)`, `contact_up(contact)` and
//!   `contact_cancel(contact)` name a contact by the [`Contact`] that
//!   `contact_down` returns, so several can be down at once. They go through
//!   the test window's `VirtualWindow::contact` as platform contacts and get
//!   pointer ids apart from the mouse's (from 2), as on a native window.
//! - Surfaces and the App's lifecycle: `Screen::lose_surface` and
//!   `restore_surface` (the surface destroyed, then its next generation),
//!   `Screen::suspend(destroys_surface)` and `resume`, `memory_warning`,
//!   `set_safe_area`, and the software keyboard read through
//!   `soft_keyboard_shown` and `soft_keyboard_requests`.

use kivact_engine::testing::{PointerId, PointerInput, PointerPhase, ShellEvent};
use kivact_wire::{self as p, NodeId};

use super::model::{Contact, Element, Screen};
use crate::view::input::{ButtonPress, Modifiers, MouseButton, PointerKind};
#[cfg(test)]
use crate::view::state::ViewState;
use crate::view::{A11yAction, Key, Role};
use kivact_style::draw::{Point, Rect};

/// The text a key types, as the platform reports it with the key.
fn key_text(key: Key) -> Option<String> {
    match key {
        Key::Char(c) => Some(c.to_string()),
        Key::Space => Some(" ".into()),
        _ => None,
    }
}

/// The press a touch or pen contact makes: the primary button, a single click
/// (the window counts taps).
const CONTACT_PRESS: ButtonPress = ButtonPress {
    button: MouseButton::Primary,
    clicks: 1,
};

impl Screen {
    pub(super) fn send(&self, event: ShellEvent) {
        self.host.borrow_mut().send(event);
    }

    /// Move the virtual window's mouse to the window point `at`.
    fn mouse_to(&self, at: Point) {
        self.host.borrow_mut().mouse(PointerPhase::Move, Some(at));
    }

    /// Press or release `button` where the virtual window's mouse is.
    fn mouse_button(&self, button: MouseButton, pressed: bool) {
        let press = ButtonPress { button, clicks: 1 };
        let phase = if pressed {
            PointerPhase::Down(press)
        } else {
            PointerPhase::Up(press)
        };
        self.host.borrow_mut().mouse(phase, None);
    }

    /// Send `key` with `modifiers` held (none: no modifier events).
    fn key_events(&self, key: Key, modifiers: Modifiers) {
        let held = modifiers != Modifiers::default();
        if held {
            self.send(ShellEvent::Modifiers(modifiers));
        }
        // A chord types no text (Ctrl+A selects, it does not type "a").
        let text = if modifiers.ctrl || modifiers.alt || modifiers.meta {
            None
        } else {
            key_text(key)
        };
        // The US layout: a letter key is its own physical key.
        let physical = match key {
            Key::Char(c) if c.is_ascii_alphabetic() => Some(c.to_ascii_lowercase()),
            _ => None,
        };
        self.send(ShellEvent::Key {
            key,
            physical,
            code: Default::default(),
            repeat: false,
            text,
        });
        self.send(ShellEvent::KeyUp {
            key,
            code: Default::default(),
        });
        if held {
            self.send(ShellEvent::Modifiers(Modifiers::default()));
        }
    }

    /// Type `text` key by key (a newline is Enter), or commit it when an
    /// IME composition is open (the IME commit replaces the composition).
    fn type_events(&self, text: &str) {
        if self.host.borrow().composing() {
            self.send(ShellEvent::Commit(text.into()));
            return;
        }
        for c in text.chars() {
            let key = match c {
                '\n' => Key::Enter,
                '\t' => Key::Tab,
                ' ' => Key::Space,
                c => Key::Char(c),
            };
            self.key_events(key, Modifiers::default());
        }
    }

    /// Press a key on whatever has the focus.
    pub fn press_key(&self, key: Key) {
        self.press_key_with(key, Modifiers::default());
    }

    /// Press a key with modifiers held (`Ctrl+A`, `Shift+Tab` ..) on
    /// whatever has the focus.
    pub fn press_key_with(&self, key: Key, modifiers: Modifiers) {
        self.log(format!("press {key:?} {modifiers:?}"));
        self.key_events(key, modifiers);
        self.run_until_idle();
    }

    /// Press Tab (Shift+Tab with `shift`): the host moves the focus.
    pub fn press_tab(&self, shift: bool) {
        self.press_key_with(
            Key::Tab,
            Modifiers {
                shift,
                ..Modifiers::default()
            },
        );
    }

    /// Type `text` into whatever has the focus.
    pub fn type_text(&self, text: &str) {
        self.log(format!("type {text:?}"));
        self.type_events(text);
        self.run_until_idle();
    }

    /// Wheel at the current pointer position (positive `dy` scrolls
    /// content forward, down), on whatever the hit test finds there (no
    /// actionability check). Move the pointer first with
    /// [`Self::mouse_move`]; [`Element::scroll`] moves it to an element.
    pub fn scroll(&self, dx: f32, dy: f32) {
        self.log(format!("scroll ({dx}, {dy}) at the pointer"));
        self.send(ShellEvent::Wheel { x: dx, y: dy });
        self.run_until_idle();
    }

    /// Move the pointer to the window point `(x, y)` (logical pixels).
    pub fn mouse_move(&self, x: f32, y: f32) {
        self.log(format!("move to ({x}, {y})"));
        self.mouse_to(Point::new(x, y));
        self.run_until_idle();
    }

    /// Press `button` where the pointer is. Panics when the pointer was never placed in the window (call
    /// [`Self::mouse_move`] or click or hover an element first).
    pub fn mouse_down(&self, button: MouseButton) {
        self.log(format!("{button:?} down"));
        self.mouse_button(button, true);
        self.run_until_idle();
    }

    /// Release `button` where the pointer is. Panics when the pointer was never placed in the window (call
    /// [`Self::mouse_move`] or click or hover an element first).
    pub fn mouse_up(&self, button: MouseButton) {
        self.log(format!("{button:?} up"));
        self.mouse_button(button, false);
        self.run_until_idle();
    }

    /// Click the primary button at the window point `(x, y)`, on whatever
    /// the hit test finds there (no actionability check).
    pub fn click_at(&self, x: f32, y: f32) {
        self.log(format!("click at ({x}, {y})"));
        self.pointer_click(Point::new(x, y), MouseButton::Primary);
        self.run_until_idle();
    }

    /// Put a finger (`PointerKind::Touch`) or a pen (`PointerKind::Pen`) on
    /// the window point `(x, y)` (logical pixels): a new contact, which
    /// [`Screen::contact_move`], [`Screen::contact_up`] and
    /// [`Screen::contact_cancel`] go on with. Contacts already down stay
    /// down, so several fingers can touch at once (the first of its kind is
    /// the primary pointer). `PointerKind::Mouse` is not a contact: use the
    /// `mouse_*` methods.
    ///
    /// ```ignore
    /// let a = screen.contact_down(PointerKind::Touch, 10.0, 10.0);
    /// let b = screen.contact_down(PointerKind::Touch, 50.0, 10.0);
    /// screen.contact_move(a, 0.0, 10.0);
    /// screen.contact_move(b, 60.0, 10.0);
    /// screen.contact_up(a);
    /// screen.contact_cancel(b);
    /// ```
    #[must_use = "the contact stays down until `contact_up` or `contact_cancel` names it"]
    #[track_caller]
    pub fn contact_down(&self, kind: PointerKind, x: f32, y: f32) -> Contact {
        if kind == PointerKind::Mouse {
            self.fail("contact_down: the mouse is not a contact (use the mouse_* methods)");
        }
        let id = self.last_contact.get() + 1;
        self.last_contact.set(id);
        let contact = Contact { id, kind };
        let at = Point::new(x, y);
        self.contacts.borrow_mut().push((contact, at));
        self.log(format!("{} down at ({x}, {y})", contact.name()));
        self.send_contact(PointerPhase::Down(CONTACT_PRESS), contact, at);
        self.run_until_idle();
        contact
    }

    /// Move `contact` to the window point `(x, y)`.
    #[track_caller]
    pub fn contact_move(&self, contact: Contact, x: f32, y: f32) {
        self.current_contact("contact_move", contact);
        let at = Point::new(x, y);
        if let Some(entry) = self
            .contacts
            .borrow_mut()
            .iter_mut()
            .find(|(c, _)| *c == contact)
        {
            entry.1 = at;
        }
        self.log(format!("{} moves to ({x}, {y})", contact.name()));
        self.send_contact(PointerPhase::Move, contact, at);
        self.run_until_idle();
    }

    /// Lift `contact` where it is.
    #[track_caller]
    pub fn contact_up(&self, contact: Contact) {
        let at = self.end_contact("contact_up", contact);
        self.log(format!("{} up", contact.name()));
        self.send_contact(PointerPhase::Up(CONTACT_PRESS), contact, at);
        self.run_until_idle();
    }

    /// The platform cancels `contact` (a system gesture took it over, a
    /// palm was rejected): it ends without a release. The other contacts
    /// stay down.
    #[track_caller]
    pub fn contact_cancel(&self, contact: Contact) {
        let at = self.end_contact("contact_cancel", contact);
        self.log(format!("{} canceled", contact.name()));
        self.send_contact(PointerPhase::Cancel, contact, at);
        self.run_until_idle();
    }

    /// Where `contact` is, failing the test if it is not down.
    #[track_caller]
    fn current_contact(&self, what: &str, contact: Contact) -> Point {
        let found = self
            .contacts
            .borrow()
            .iter()
            .find(|(c, _)| *c == contact)
            .map(|(_, at)| *at);
        match found {
            Some(at) => at,
            None => self.fail(&format!(
                "{what}: {} is not down (contact_down first; a lifted or canceled contact stays lifted)",
                contact.name()
            )),
        }
    }

    /// Forget `contact` (it ends), returning where it was.
    #[track_caller]
    fn end_contact(&self, what: &str, contact: Contact) -> Point {
        let at = self.current_contact(what, contact);
        self.contacts.borrow_mut().retain(|(c, _)| *c != contact);
        at
    }

    /// Report the contact as the platform does: the window maps the
    /// platform's contact to a pointer of its own (`VirtualWindow::contact`).
    fn send_contact(&self, phase: PointerPhase, contact: Contact, at: Point) {
        self.host.borrow_mut().contact(PointerInput {
            id: PointerId(contact.id),
            kind: contact.kind,
            phase,
            position: at,
            buttons: p::PointerButtons::NONE.with(
                MouseButton::Primary,
                matches!(phase, PointerPhase::Down(_) | PointerPhase::Move),
            ),
            pressure: None,
            tilt: None,
        });
    }

    /// The pointer leaves the window. Panics when the pointer was never placed in the window (call
    /// [`Self::mouse_move`] or click or hover an element first).
    pub fn mouse_leave(&self) {
        self.log("pointer leaves".into());
        self.host.borrow_mut().mouse(PointerPhase::Leave, None);
        self.run_until_idle();
    }

    fn pointer_click(&self, at: Point, button: MouseButton) {
        self.mouse_to(at);
        self.mouse_button(button, true);
        self.mouse_button(button, false);
    }

    /// The window's logical size (its root's box).
    fn window_rect(&self) -> Option<p::Rect> {
        let host = self.host.borrow();
        let root = *host.preorder().first()?;
        host.rect(root)
    }

    /// Whether `node` is `ancestor` or below it in the host's tree.
    fn in_subtree(&self, node: NodeId, ancestor: NodeId) -> bool {
        let host = self.host.borrow();
        let mut stack = vec![ancestor];
        while let Some(id) = stack.pop() {
            if id == node {
                return true;
            }
            stack.extend(host.children(id).unwrap_or_default());
        }
        false
    }
}

impl<'a> Element<'a> {
    /// The element's box (the region's for a region) in window
    /// coordinates, if it is mounted on the host.
    pub(super) fn host_rect(&self) -> Option<p::Rect> {
        let host = self.screen.host.borrow();
        match self.region {
            Some(r) => host.region_rect(self.node, r),
            None => host.rect(self.node),
        }
    }

    /// The centre of the element (its region's for a region) relative to
    /// its node's untransformed box: where actions land, mapped by the
    /// node's transforms.
    fn local_center(&self) -> Point {
        let host = self.screen.host.borrow();
        let r = match self.region {
            Some(region) => host.region_local(self.node, region),
            None => host.size(self.node).map(|s| p::Rect {
                x: 0.0,
                y: 0.0,
                width: s.width,
                height: s.height,
            }),
        }
        .unwrap_or_default();
        Point::new(r.x + r.width / 2.0, r.y + r.height / 2.0)
    }

    /// `node <id>` (and the region) for the action log.
    pub(super) fn short(&self) -> String {
        let region = self
            .region
            .map(|r| format!(" region {}", r.0))
            .unwrap_or_default();
        format!("node {}{region}", self.node.0)
    }

    fn describe(&self) -> String {
        let region = self
            .region
            .map(|r| format!(" region {}", r.0))
            .unwrap_or_default();
        format!("{}{region}", self.screen.describe(self.node))
    }

    /// Where `what` lands on the element: the point at the offset `local`
    /// in its node's box (default: the centre of the element's box, the
    /// region's for a region), after the actionability checks (`enabled`:
    /// the element must not be disabled).
    #[track_caller]
    fn action_point(&self, what: &str, local: Option<Point>, enabled: bool) -> Point {
        let screen = self.screen;
        screen.run_until_idle();
        let fail = |why: String| -> ! {
            screen.fail(&screen.report(&format!("{what}: {} {why}", self.describe()), None, &[]))
        };
        let Some(r) = self.host_rect() else {
            fail("is not mounted".into());
        };
        if !screen.host.borrow().shown(self.node) {
            fail("is hidden".into());
        }
        if r.width <= 0.0 || r.height <= 0.0 {
            fail(format!("has no size ({} x {})", r.width, r.height));
        }
        if enabled && self.is_disabled() {
            fail("is disabled".into());
        }
        // Local to the node's untransformed box (a region's too, like
        // `drag`), as drawn: through the node's transforms.
        let local = local.unwrap_or_else(|| self.local_center());
        let at = { screen.host.borrow().window_point(self.node, local) };
        if let Some(w) = screen.window_rect()
            && !(at.x >= w.x && at.y >= w.y && at.x < w.x + w.width && at.y < w.y + w.height)
        {
            fail(format!(
                "is outside the window at ({}, {}) (window {} x {}); scroll it into view first",
                at.x, at.y, w.width, w.height
            ));
        }
        let hit = screen.host.borrow().hit(at.x, at.y);
        let modal = screen.host.borrow().modal();
        match hit {
            Some(h) if screen.in_subtree(h, self.node) => at,
            Some(h) => fail(format!(
                "is covered at ({}, {}) by {}",
                at.x,
                at.y,
                screen.describe(h)
            )),
            None => match modal {
                Some(m) => fail(format!(
                    "is behind the backdrop of the modal {}",
                    screen.describe(m)
                )),
                None => fail(format!(
                    "is not hit at ({}, {}) (clipped out of view?)",
                    at.x, at.y
                )),
            },
        }
    }

    /// Click the centre of the element with the primary button (pointer
    /// move, press, release), after the actionability checks.
    #[track_caller]
    pub fn click(&self) {
        self.click_with(MouseButton::Primary);
    }

    /// Click the centre of the element with `button`.
    #[track_caller]
    pub fn click_with(&self, button: MouseButton) {
        let at = self.action_point("click", None, true);
        self.screen.log(format!("click {}", self.short()));
        self.screen.pointer_click(at, button);
        self.screen.run_until_idle();
    }

    /// Click the centre of the element's box without the actionability
    /// checks: the press lands on whatever the hit test finds there (a
    /// disabled control, a modal's backdrop ..).
    pub fn force_click(&self) {
        if self.host_rect().is_none() {
            self.screen.fail(&format!(
                "force_click: {} is not mounted",
                self.screen.describe(self.node)
            ));
        }
        self.screen.log(format!("force click {}", self.short()));
        let c = self.local_center();
        let at = { self.screen.host.borrow().window_point(self.node, c) };
        self.screen.pointer_click(at, MouseButton::Primary);
        self.screen.run_until_idle();
    }

    /// Tap the centre of the element with a finger (touch down and up),
    /// after the actionability checks.
    #[track_caller]
    pub fn tap(&self) {
        self.tap_with(PointerKind::Touch);
    }

    /// Tap the centre of the element with a finger or a pen
    /// (`PointerKind::Mouse` clicks instead), after the actionability
    /// checks.
    #[track_caller]
    pub fn tap_with(&self, kind: PointerKind) {
        if kind == PointerKind::Mouse {
            return self.click();
        }
        let at = self.action_point("tap", None, true);
        self.screen.log(format!("{kind:?} tap {}", self.short()));
        let contact = self.screen.contact_down(kind, at.x, at.y);
        self.screen.contact_up(contact);
    }

    /// Two clicks in a row at the same point (a double click: the host
    /// counts them).
    #[track_caller]
    pub fn double_click(&self) {
        self.click();
        self.click();
    }

    /// Move the pointer onto the centre of the element.
    #[track_caller]
    pub fn hover(&self) {
        let at = self.action_point("hover", None, false);
        self.screen.log(format!("hover {}", self.short()));
        self.screen.mouse_to(at);
        self.screen.run_until_idle();
    }

    /// Give the element the focus through the accessibility `Focus` action
    /// (as assistive technology or a script does); fails if it does not
    /// take it.
    #[track_caller]
    pub fn focus(&self) {
        self.screen.log(format!("focus {}", self.short()));
        self.focus_now();
        self.screen.run_until_idle();
    }

    /// Focus the element unless it has the focus.
    #[track_caller]
    pub(super) fn ensure_focus(&self) {
        if !self.is_focused() {
            self.focus_now();
        }
    }

    #[track_caller]
    fn focus_now(&self) {
        let screen = self.screen;
        screen.run_until_idle();
        let published = self.published_any();
        let why = match &published {
            None => Some("is not in the accessibility tree".to_string()),
            Some(n) if n.role == p::Role::Generic => Some(
                "is not exposed to assistive technology (it has no role), so nothing can focus it \
                 but a click or Tab; give it a role"
                    .to_string(),
            ),
            Some(n) if n.hidden => Some("is hidden".into()),
            Some(n) if n.disabled => Some("is disabled".into()),
            Some(n) if !n.focusable => {
                Some("is not focusable (not a tab stop, or outside the open modal)".to_string())
            }
            Some(_) => None,
        };
        if let Some(why) = why {
            screen.fail(&screen.report(&format!("focus: {} {why}", self.describe()), None, &[]));
        }
        let id = screen
            .host
            .borrow()
            .semantics_id(self.node, self.region)
            .expect("invariant: focusable element has accessibility identity");
        screen.send(ShellEvent::Accessibility {
            node: id,
            request: p::A11yRequest::Focus,
        });
        screen.run_until_idle();
        if !self.is_focused() {
            screen.fail(&screen.report(
                &format!("focus: {} did not take the focus", self.describe()),
                None,
                &[],
            ));
        }
    }

    /// Focus the element (unless focused) and type `text` key by key, as a
    /// keyboard does (a newline is Enter); while an IME composition is open
    /// the text is committed instead.
    #[track_caller]
    pub fn type_text(&self, text: &str) {
        self.ensure_focus();
        self.screen
            .log(format!("type {text:?} into {}", self.short()));
        self.screen.type_events(text);
        self.screen.run_until_idle();
    }

    /// Focus the element (unless focused) and set the IME composition
    /// (preedit) text, caret at its end.
    #[track_caller]
    pub fn compose(&self, preedit: &str) {
        self.ensure_focus();
        self.screen
            .log(format!("compose {preedit:?} in {}", self.short()));
        self.screen.send(ShellEvent::Preedit {
            text: preedit.into(),
            selection: Some((preedit.len(), preedit.len())),
        });
        self.screen.run_until_idle();
    }

    /// Focus the element (unless focused) and commit `text` from the IME
    /// (it replaces an open composition).
    #[track_caller]
    pub fn commit(&self, text: &str) {
        self.ensure_focus();
        self.screen
            .log(format!("commit {text:?} in {}", self.short()));
        self.screen.send(ShellEvent::Commit(text.into()));
        self.screen.run_until_idle();
    }

    /// Focus the element (unless focused) and press `key`.
    #[track_caller]
    pub fn press_key(&self, key: Key) {
        self.press_key_with(key, Modifiers::default());
    }

    /// Focus the element (unless focused) and press `key` with modifiers.
    #[track_caller]
    pub fn press_key_with(&self, key: Key, modifiers: Modifiers) {
        self.ensure_focus();
        self.screen
            .log(format!("press {key:?} {modifiers:?} on {}", self.short()));
        self.screen.key_events(key, modifiers);
        self.screen.run_until_idle();
    }

    /// Drag with the primary button from `from` to `to` (local to the
    /// element's node box, also for a scene region): press at `from`, move
    /// through the midpoint to `to`, release there. The press must land on
    /// the element.
    #[track_caller]
    pub fn drag(&self, from: Point, to: Point) {
        let start = self.action_point("drag", Some(from), true);
        self.screen.log(format!("drag {}", self.short()));
        let node = self.node;
        let host = &self.screen.host;
        let at = |l: Point| host.borrow().window_point(node, l);
        let s = self.screen;
        s.mouse_to(start);
        s.mouse_button(MouseButton::Primary, true);
        s.mouse_to(at(Point::new((from.x + to.x) / 2.0, (from.y + to.y) / 2.0)));
        s.mouse_to(at(to));
        s.mouse_button(MouseButton::Primary, false);
        s.run_until_idle();
    }

    /// Wheel over the centre of the element (positive `dy` scrolls content
    /// forward, down).
    #[track_caller]
    pub fn scroll(&self, dx: f32, dy: f32) {
        let at = self.action_point("scroll", None, false);
        self.screen
            .log(format!("scroll ({dx}, {dy}) over {}", self.short()));
        self.screen.mouse_to(at);
        self.screen.send(ShellEvent::Wheel { x: dx, y: dy });
        self.screen.run_until_idle();
    }

    /// Click the element (a check box, switch ..).
    #[track_caller]
    pub fn toggle(&self) {
        self.click();
    }

    /// Open a select and choose the option named `name` (two clicks).
    #[track_caller]
    pub fn select_option(&self, name: &str) {
        self.click();
        let opt = self.screen.get_by_role(Role::Option, name);
        opt.click();
    }

    /// Request an accessibility action, as assistive technology does (the
    /// host carries out only what the element offers now). A `Dismiss` is
    /// `DismissReason::Accessibility`. `SetValue` carries a value: use
    /// [`a11y_set_value`](Self::a11y_set_value).
    #[track_caller]
    pub fn a11y(&self, action: A11yAction) {
        let Some(request) = p::A11yRequest::from_action(action) else {
            self.screen.fail(&format!(
                "a11y: SetValue carries a value; use `a11y_set_value(..)` on {}",
                self.describe()
            ));
        };
        self.a11y_request(request);
    }

    /// Ask assistive technology's `SetValue` with `value`.
    #[track_caller]
    pub fn a11y_set_value(&self, value: &str) {
        self.a11y_request(p::A11yRequest::SetValue(value.to_string()));
    }

    #[track_caller]
    fn a11y_request(&self, request: p::A11yRequest) {
        self.screen
            .log(format!("a11y {request:?} on {}", self.short()));
        let id = {
            let host = self.screen.host.borrow();
            host.semantics_id(self.node, self.region)
        };
        let Some(id) = id else {
            self.screen.fail(&format!(
                "a11y: {} is not in the accessibility tree",
                self.describe()
            ));
        };
        self.screen
            .send(ShellEvent::Accessibility { node: id, request });
        self.screen.run_until_idle();
    }

    /// The element's box in window coordinates (the region's for a scene
    /// region) as drawn: its layout box mapped by its transforms (their
    /// bounds), like `NodeRef::rect`.
    pub fn rect(&self) -> Option<Rect> {
        let r = self.host_rect()?;
        Some(r)
    }
}

/// Input queued without settling (internal tests): the host applies it at
/// once and the client's turn for all of it runs at the next settle, as
/// for input arriving between two turns of a running App.
#[cfg(test)]
impl Screen {
    /// Press and release the primary button at the centre of `node`.
    pub(crate) fn queue_click(&self, node: NodeId) {
        let r = self
            .host
            .borrow()
            .rect(node)
            .expect("invariant: the node is laid out");
        self.pointer_click(
            Point::new(r.x + r.width / 2.0, r.y + r.height / 2.0),
            MouseButton::Primary,
        );
    }

    /// Type `text` key by key into whatever has the focus.
    pub(crate) fn queue_text(&self, text: &str) {
        self.type_events(text);
    }

    /// The host reports a failure after the last revision it applied.
    pub(crate) fn queue_host_failure(&self, reason: &str) {
        let revision = kivact_wire::Revision(ViewState::tree(&self.rt).engine().stats().revision);
        self.host
            .borrow_mut()
            .inject(kivact_wire::HostEvent::HostFailure {
                revision,
                reason: reason.into(),
            });
    }
}
