//! Session actions that come from the desktop rather than from a screen: a
//! global hotkey, and the machine going to sleep.
//!
//! Neither of these can assume there is a session at all — a hotkey is
//! pressed whenever the student feels like it, and a laptop lid closes on an
//! idle app just as often as on a running timer. So nothing here is an
//! error: «нечего делать» is a normal answer and comes back as `None` or
//! `false`.

use crate::core::clock::Clock;
use crate::platform::SharedPlatform;

use super::actions::{pause, resume};
use super::{SessionSnapshot, SessionState};
use crate::commands::CommandError;

/// Pauses a running session, resumes a paused one.
///
/// What `lokked --toggle` does, and what a global hotkey is usually bound
/// to. A finished phase waiting to be moved on is left alone: it is neither
/// running nor paused, and «пауза» over it would mean nothing.
pub fn toggle(
    state: &SessionState,
    platform: &SharedPlatform,
    clock: &dyn Clock,
) -> Result<Option<SessionSnapshot>, CommandError> {
    let running = {
        let active = state.lock();
        active
            .as_ref()
            .map(|session| (session.timer.is_running(), session.timer.is_paused()))
    };

    match running {
        Some((true, _)) => pause(state, platform, clock).map(Some),
        Some((_, true)) => resume(state, platform, clock).map(Some),
        _ => Ok(None),
    }
}

/// Puts a running session on pause because the machine is suspending.
///
/// Returns whether anything was paused, which is what decides if the student
/// is asked about it after the machine comes back.
///
/// The time asleep is not counted as study and not offered as a choice: a
/// closed lid is not «отвлёкся на минуту», it is the machine being off. The
/// question after waking is only whether to carry on.
pub fn pause_for_sleep(state: &SessionState, platform: &SharedPlatform, clock: &dyn Clock) -> bool {
    let running = {
        let active = state.lock();
        active
            .as_ref()
            .is_some_and(|session| session.timer.is_running())
    };

    running && pause(state, platform, clock).is_ok()
}
