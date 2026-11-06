//! Role: Frame clocks, frame requests, smooth scrolls and drawability.
//!
//! The frame group: the host clock, the frame ticks kivact and the model
//! asked for, the programmatic scrolls animating, the display period and
//! whether there is a surface to draw on.
//!
//! Invariants:
//! - At most one tick per display period: a request due at or after a
//!   quarter period before the period's end is ticked at its own time, an
//!   earlier one at the period's end ([`Model::frame_deadline`]). The
//!   period is the window's display period; a change of it reaches the
//!   core's frame planning as an environment change.
//! - Without a surface no tick is delivered, and state transitions keep
//!   their place in time: the pause is skipped on resume. The same stop is
//!   the client's: every tick carries the engine's one `MotionClock` (the
//!   time stood still so far, when the last stop ended), a snapshot taken
//!   when the tick is queued. The stop is `Clock::Stopped`, written only by
//!   [`Model::start_env`] and [`Model::update_env`].
//! - A wake less than a quarter period late stamps the tick with its due
//!   time, so a display-rate loop does not add the shell's wake latency to
//!   every period. Missed ticks are never made up: a later wake (after a
//!   pause too) delivers one tick stamped now, and the schedule starts
//!   again from it ([`Model::tick`]).
//! - The environment the client sees (`Environment`: the display period,
//!   reduced motion, drawability, and the window's scale, size, fullscreen,
//!   theme, safe area and locale) lives here, once. [`Model::update_env`]
//!   queues one `EnvChanged` per change of any of it, and none when the
//!   value did not change.
//! - A smooth scroll allocates its state only while it animates; reduced
//!   motion makes it a jump. It uses the requested easing over 240 ms; touch flings decay per platform settings.

mod scroll_snap;
pub(crate) mod state;
mod ticks;
