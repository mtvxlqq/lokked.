//! The active study session: starting it, reading it, and writing it down.
//!
//! Exactly one session can be active, and it lives in Tauri's managed state
//! rather than in the frontend — the timer is domain logic, and the screen is
//! only a view of it. The frontend polls [`current`] a few times a second and
//! renders whatever comes back; every decision, including whether a finished
//! Pomodoro phase rolls over on its own, is made here.
//!
//! Rows are written when a phase *ends*, not while it runs: a phase becomes
//! one row per study day it touched (see [`crate::core::session::slice_phase`]),
//! and the write happens before the timer is mutated, so a failing database
//! leaves the session exactly as it was and the student can try again.

use std::sync::Mutex;

pub mod actions;
pub mod desktop;

use chrono::{DateTime, Local, TimeDelta, Utc};
use serde::Serialize;

use crate::core::clock::Clock;
use crate::core::session::slice_phase;
use crate::core::timer::{Mode, Pause, SessionPhase, Timer};
use crate::db::sessions::{NewSession, SessionRepo};
use crate::db::Database;
use crate::platform::SharedPlatform;

use super::settings::day_start;
use super::{CommandError, ErrorKind};

/// The session the app is running right now.
pub struct ActiveSession {
    subject_id: String,
    /// Cached so the frontend's poll does not hit the database four times a
    /// second for a name that cannot change mid-session.
    subject_name: String,
    subject_color: Option<String>,
    preset_id: Option<String>,
    /// `'countup' | 'countdown' | 'pomodoro'`, as stored in `sessions.mode`.
    mode_label: &'static str,
    timer: Timer,
    /// Studied time from the work phases that are already over, in seconds.
    /// The current phase is not in here — it is still moving, and adding it
    /// would mean accumulating time instead of deriving it.
    pub studied_seconds: i64,
}

/// Managed state: the active session, or nothing.
#[derive(Default)]
pub struct SessionState(Mutex<Option<ActiveSession>>);

impl SessionState {
    fn lock(&self) -> std::sync::MutexGuard<'_, Option<ActiveSession>> {
        self.0.lock().expect("session mutex poisoned")
    }
}

/// What the timer screen draws.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SessionSnapshot {
    pub subject_id: String,
    pub subject_name: String,
    pub subject_color: Option<String>,
    pub preset_id: Option<String>,
    pub mode: String,
    /// `'work' | 'break' | 'long_break'`.
    pub phase: String,
    /// `'running' | 'paused'`.
    pub status: String,
    /// 1-based work phase within the current group; always `1` outside Pomodoro.
    pub cycle: u32,
    /// How many work phases make up a group, for the «работа 2/4» caption.
    pub cycles_before_long: Option<u32>,
    pub elapsed_seconds: i64,
    /// Studied time since «Старт», across every work phase and excluding
    /// breaks and pauses. This is what the black screen shows.
    pub session_seconds: i64,
    /// `null` for a stopwatch, which has nothing to count down to.
    pub remaining_seconds: Option<i64>,
    pub target_seconds: Option<i64>,
    /// Whether this phase has reached its target and is waiting to be moved on.
    pub phase_finished: bool,
    pub interruptions: u32,
    pub auto_start_next: bool,
}

fn phase_label(phase: SessionPhase) -> &'static str {
    match phase {
        SessionPhase::Work => "work",
        SessionPhase::Break => "break",
        SessionPhase::LongBreak => "long_break",
    }
}

fn cycles_before_long(mode: Mode) -> Option<u32> {
    match mode {
        Mode::Pomodoro {
            cycles_before_long_break,
            ..
        } => Some(cycles_before_long_break),
        Mode::CountUp | Mode::CountDown { .. } => None,
    }
}

fn snapshot(session: &ActiveSession, clock: &dyn Clock) -> SessionSnapshot {
    let state = session.timer.state_at(clock);
    let elapsed_seconds = state.elapsed.num_seconds();
    let remaining_seconds = state.remaining.map(|left| left.num_seconds());
    // Only a work phase is study time; during a break the total stands still
    // at what the finished phases add up to.
    let session_seconds = session.studied_seconds
        + if state.phase == SessionPhase::Work {
            elapsed_seconds
        } else {
            0
        };

    SessionSnapshot {
        subject_id: session.subject_id.clone(),
        subject_name: session.subject_name.clone(),
        subject_color: session.subject_color.clone(),
        preset_id: session.preset_id.clone(),
        mode: session.mode_label.to_string(),
        phase: phase_label(state.phase).to_string(),
        status: if session.timer.is_paused() {
            "paused".to_string()
        } else {
            "running".to_string()
        },
        cycle: state.cycle,
        cycles_before_long: cycles_before_long(session.timer.mode()),
        elapsed_seconds,
        session_seconds,
        remaining_seconds,
        target_seconds: remaining_seconds.map(|left| elapsed_seconds + left),
        phase_finished: state.finished,
        interruptions: session.timer.interruptions(),
        auto_start_next: session.timer.auto_start_next(),
    }
}

/// Keeps the screen awake exactly while work is being counted.
fn sync_wakelock(platform: &SharedPlatform, session: Option<&ActiveSession>) {
    let awake = session.is_some_and(|session| {
        session.timer.is_running() && session.timer.phase() == SessionPhase::Work
    });
    platform.keep_awake(awake);
}

/// Writes the phase that is ending: one `sessions` row per study day it
/// touched. Called before the timer moves on, so a database failure leaves
/// the session intact.
///
/// Returns the studied seconds it wrote down, so the caller can add them to
/// the session's running total.
#[allow(clippy::too_many_arguments)]
fn persist_phase(
    db: &Database,
    session: &ActiveSession,
    ended_at: DateTime<Utc>,
    pauses: &[Pause],
    completed: bool,
    planned_seconds: Option<i64>,
    interruptions: u32,
    day_start: TimeDelta,
) -> Result<i64, CommandError> {
    let repo = SessionRepo::new(db);
    let mut active_seconds = 0;

    for (index, slice) in slice_phase(
        session.timer.phase_started_at(),
        ended_at,
        pauses,
        &Local,
        day_start,
    )
    .into_iter()
    .enumerate()
    {
        repo.create(NewSession {
            subject_id: &session.subject_id,
            preset_id: session.preset_id.as_deref(),
            mode: session.mode_label,
            phase: phase_label(session.timer.phase()),
            started_at: slice.started_at,
            ended_at: slice.ended_at,
            day_key: &slice.day_key,
            active_seconds: slice.active_seconds,
            paused_seconds: slice.paused_seconds,
            planned_seconds,
            completed,
            // Interruptions belong to the phase, not to a day: crediting each
            // to the slice it was marked in would need timestamps the timer
            // deliberately does not keep, so the first slice carries them all
            // and any later one carries none. Summing the column over a phase
            // still gives the right total.
            interruptions: if index == 0 { interruptions.into() } else { 0 },
            device_id: None,
        })?;
        active_seconds += slice.active_seconds;
    }

    Ok(active_seconds)
}

/// Writes the phase that is ending, reading everything it needs from the
/// timer first — `skip_phase` resets the pauses and the interruption count,
/// so both have to be captured before it runs.
fn persist_current_phase(
    db: &Database,
    session: &mut ActiveSession,
    ended_at: DateTime<Utc>,
    clock: &dyn Clock,
) -> Result<(), CommandError> {
    let state = session.timer.state_at(clock);
    let planned = state
        .remaining
        .map(|left| (state.elapsed + left).num_seconds());

    let studied = persist_phase(
        db,
        session,
        ended_at,
        &session.timer.pauses_at(clock),
        state.finished,
        planned,
        session.timer.interruptions(),
        day_start(db)?,
    )?;

    // Breaks are not studied time and never count toward the session total.
    if session.timer.phase() == SessionPhase::Work {
        session.studied_seconds += studied;
    }

    Ok(())
}

fn conflict(message: &str) -> CommandError {
    CommandError {
        kind: ErrorKind::Conflict,
        message: message.to_string(),
    }
}

/// How much of the study day `day` the current phase has already earned,
/// before it is written down.
///
/// Rows appear in `sessions` only when a phase ends, so without this the
/// subject list would show nothing for a session that has been running for
/// half an hour. Returns `(subject_id, seconds)`, and nothing at all during a
/// break — that is not study time — or when the phase has yet to earn a
/// second of the day.
pub fn work_in_progress(
    state: &SessionState,
    clock: &dyn Clock,
    day_start: TimeDelta,
    day: &str,
) -> Option<(String, i64)> {
    let active = state.lock();
    let session = active.as_ref()?;

    if session.timer.phase() != SessionPhase::Work {
        return None;
    }

    let seconds: i64 = slice_phase(
        session.timer.phase_started_at(),
        clock.now(),
        &session.timer.pauses_at(clock),
        &Local,
        day_start,
    )
    .into_iter()
    .filter(|slice| slice.day_key == day)
    .map(|slice| slice.active_seconds)
    .sum();

    (seconds > 0).then(|| (session.subject_id.clone(), seconds))
}
