//! The verbs of a session: start, pause, resume, skip, stop — plus what to do
//! about time that passed while the app was not on screen.
//!
//! Each action is a plain function over the managed state so it can be tested
//! with a [`FakeClock`](crate::core::clock::FakeClock) and an in-memory
//! database; the `#[tauri::command]` wrappers at the bottom only unwrap
//! `State` and supply the real clock.

use chrono::{DateTime, Utc};
use tauri::State;

use crate::core::clock::Clock;
use crate::core::preset::{select_preset, validate, PresetChoice, PresetDraft};
use crate::core::session::{away_report, AwayReport};
use crate::core::timer::{Mode, Timer, TimerError};
use crate::db::presets::PresetRepo;
use crate::db::subjects::SubjectRepo;
use crate::db::Database;
use crate::platform::clock::SystemClock;
use crate::platform::SharedPlatform;

use super::super::CommandError;
use super::{
    conflict, persist_current_phase, snapshot, sync_wakelock, ActiveSession, SessionSnapshot,
    SessionState,
};

/// Timer transitions in the student's language. `TimerError`'s own `Display`
/// is developer-facing English; these are the strings a dialog shows.
fn describe(error: TimerError) -> CommandError {
    conflict(match error {
        TimerError::AlreadyPaused => "сессия уже на паузе",
        TimerError::NotPaused => "сессия не на паузе",
        TimerError::AlreadyFinished => "сессия уже завершена",
        TimerError::NoNextPhase => "у этого режима нет следующей фазы",
    })
}

fn no_session() -> CommandError {
    conflict("сейчас нет активной сессии")
}

/// Starts a session for `subject_id` with the preset that applies to it.
///
/// With no preset at all, the session is a plain stopwatch: a student who
/// never opened the preset dialog should still be able to press «Старт».
pub fn start(
    db: &Database,
    state: &SessionState,
    platform: &SharedPlatform,
    clock: &dyn Clock,
    subject_id: &str,
) -> Result<SessionSnapshot, CommandError> {
    let mut active = state.lock();
    if active.is_some() {
        return Err(conflict("сессия уже идёт"));
    }

    let subject = SubjectRepo::new(db)
        .get(subject_id)?
        .filter(|subject| subject.deleted_at.is_none())
        .ok_or_else(|| CommandError::not_found("предмет"))?;

    let presets = PresetRepo::new(db).list()?;
    let choices: Vec<PresetChoice<'_>> = presets
        .iter()
        .map(|preset| PresetChoice {
            id: &preset.id,
            subject_id: preset.subject_id.as_deref(),
            is_default: preset.is_default,
        })
        .collect();

    let chosen = select_preset(&choices, subject_id)
        .and_then(|id| presets.iter().find(|preset| preset.id == id));

    let (mode, mode_label, preset_id) = match chosen {
        Some(preset) => {
            let valid = validate(PresetDraft {
                name: &preset.name,
                mode: &preset.mode,
                work_seconds: preset.work_seconds,
                break_seconds: preset.break_seconds,
                long_break_seconds: preset.long_break_seconds,
                cycles_before_long: preset.cycles_before_long,
                auto_start_next: preset.auto_start_next,
            })?;

            (
                valid.to_mode(),
                valid.kind.as_str(),
                Some(preset.id.clone()),
            )
        }
        None => (Mode::CountUp, "countup", None),
    };

    let session = ActiveSession {
        subject_id: subject.id,
        subject_name: subject.name,
        subject_color: subject.color,
        preset_id,
        mode_label,
        timer: Timer::start(mode, clock),
        studied_seconds: 0,
    };

    let view = snapshot(&session, clock);
    *active = Some(session);
    sync_wakelock(platform, active.as_ref());

    Ok(view)
}

/// The session as it stands right now, or `None` if there is none.
///
/// Also the place a Pomodoro rolls over: when a phase with `auto_start_next`
/// has run out, this writes it down and moves to the next one. One transition
/// per call is enough — the screen polls several times a second — and a
/// student who was away for an hour gets the «засчитать или отбросить»
/// dialog rather than a silent replay of the phases they missed.
pub fn current(
    db: &Database,
    state: &SessionState,
    platform: &SharedPlatform,
    clock: &dyn Clock,
) -> Result<Option<SessionSnapshot>, CommandError> {
    let mut active = state.lock();
    let Some(session) = active.as_mut() else {
        return Ok(None);
    };

    let view = snapshot(session, clock);
    if !(view.phase_finished && view.auto_start_next) {
        return Ok(Some(view));
    }

    persist_current_phase(db, session, clock.now(), clock)?;
    session.timer.skip_phase(clock).map_err(describe)?;

    let view = snapshot(session, clock);
    sync_wakelock(platform, active.as_ref());
    Ok(Some(view))
}

pub fn pause(
    state: &SessionState,
    platform: &SharedPlatform,
    clock: &dyn Clock,
) -> Result<SessionSnapshot, CommandError> {
    let mut active = state.lock();
    let session = active.as_mut().ok_or_else(no_session)?;

    session.timer.pause(clock).map_err(describe)?;

    let view = snapshot(session, clock);
    sync_wakelock(platform, active.as_ref());
    Ok(view)
}

pub fn resume(
    state: &SessionState,
    platform: &SharedPlatform,
    clock: &dyn Clock,
) -> Result<SessionSnapshot, CommandError> {
    let mut active = state.lock();
    let session = active.as_mut().ok_or_else(no_session)?;

    session.timer.resume(clock).map_err(describe)?;

    let view = snapshot(session, clock);
    sync_wakelock(platform, active.as_ref());
    Ok(view)
}

/// Records «отвлёкся» without stopping the clock: the time still counts, but
/// the session remembers it was not clean.
pub fn mark_interruption(
    state: &SessionState,
    clock: &dyn Clock,
) -> Result<SessionSnapshot, CommandError> {
    let mut active = state.lock();
    let session = active.as_mut().ok_or_else(no_session)?;

    session.timer.mark_interruption().map_err(describe)?;
    Ok(snapshot(session, clock))
}

/// Moves to the next Pomodoro phase by hand, writing down the one being left.
pub fn skip_phase(
    db: &Database,
    state: &SessionState,
    platform: &SharedPlatform,
    clock: &dyn Clock,
) -> Result<SessionSnapshot, CommandError> {
    let mut active = state.lock();
    let session = active.as_mut().ok_or_else(no_session)?;

    persist_current_phase(db, session, clock.now(), clock)?;
    session.timer.skip_phase(clock).map_err(describe)?;

    let view = snapshot(session, clock);
    sync_wakelock(platform, active.as_ref());
    Ok(view)
}

/// Ends the session, writing down the phase that was running.
pub fn stop(
    db: &Database,
    state: &SessionState,
    platform: &SharedPlatform,
    clock: &dyn Clock,
) -> Result<(), CommandError> {
    let mut active = state.lock();
    let session = active.as_mut().ok_or_else(no_session)?;

    // Written before the timer is touched: if this fails, the session is
    // still there to stop again rather than lost.
    persist_current_phase(db, session, clock.now(), clock)?;
    session.timer.finish(clock).map_err(describe)?;

    *active = None;
    sync_wakelock(platform, None);
    Ok(())
}

/// How long the app was away, and whether that is long enough to ask about.
///
/// Without an active session there is nothing to decide: the time was not
/// being counted in the first place.
pub fn report_return(
    state: &SessionState,
    clock: &dyn Clock,
    last_seen: DateTime<Utc>,
) -> AwayReport {
    let report = away_report(last_seen, clock.now());

    if state.lock().is_none() {
        return AwayReport {
            needs_decision: false,
            ..report
        };
    }
    report
}

/// Throws away the time between `since` and now — the student said they were
/// not studying while the app was out of sight.
pub fn discard_away(
    state: &SessionState,
    clock: &dyn Clock,
    since: DateTime<Utc>,
) -> Result<SessionSnapshot, CommandError> {
    let mut active = state.lock();
    let session = active.as_mut().ok_or_else(no_session)?;

    session
        .timer
        .discard_span(since, clock.now(), clock)
        .map_err(describe)?;

    Ok(snapshot(session, clock))
}

// The `#[tauri::command]` wrappers — see the note in [`super::super::subjects`].

#[tauri::command]
pub fn start_session(
    db: State<'_, Database>,
    state: State<'_, SessionState>,
    platform: State<'_, SharedPlatform>,
    subject_id: String,
) -> Result<SessionSnapshot, CommandError> {
    start(&db, &state, &platform, &SystemClock, &subject_id)
}

#[tauri::command]
pub fn session_snapshot(
    db: State<'_, Database>,
    state: State<'_, SessionState>,
    platform: State<'_, SharedPlatform>,
) -> Result<Option<SessionSnapshot>, CommandError> {
    current(&db, &state, &platform, &SystemClock)
}

#[tauri::command]
pub fn pause_session(
    state: State<'_, SessionState>,
    platform: State<'_, SharedPlatform>,
) -> Result<SessionSnapshot, CommandError> {
    pause(&state, &platform, &SystemClock)
}

#[tauri::command]
pub fn resume_session(
    state: State<'_, SessionState>,
    platform: State<'_, SharedPlatform>,
) -> Result<SessionSnapshot, CommandError> {
    resume(&state, &platform, &SystemClock)
}

#[tauri::command]
pub fn session_mark_interruption(
    state: State<'_, SessionState>,
) -> Result<SessionSnapshot, CommandError> {
    mark_interruption(&state, &SystemClock)
}

#[tauri::command]
pub fn session_skip_phase(
    db: State<'_, Database>,
    state: State<'_, SessionState>,
    platform: State<'_, SharedPlatform>,
) -> Result<SessionSnapshot, CommandError> {
    skip_phase(&db, &state, &platform, &SystemClock)
}

#[tauri::command]
pub fn stop_session(
    db: State<'_, Database>,
    state: State<'_, SessionState>,
    platform: State<'_, SharedPlatform>,
) -> Result<(), CommandError> {
    stop(&db, &state, &platform, &SystemClock)
}

#[tauri::command]
pub fn session_report_return(
    state: State<'_, SessionState>,
    last_seen: DateTime<Utc>,
) -> AwayReport {
    report_return(&state, &SystemClock, last_seen)
}

#[tauri::command]
pub fn session_discard_away(
    state: State<'_, SessionState>,
    since: DateTime<Utc>,
) -> Result<SessionSnapshot, CommandError> {
    discard_away(&state, &SystemClock, since)
}
