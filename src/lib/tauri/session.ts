/**
 * Активная сессия: старт, пауза, фазы, возвращение из фона.
 */

import { invoke } from "@tauri-apps/api/core";
import type { PresetMode } from "@/lib/tauri/presets";

export type SessionPhase = "work" | "break" | "long_break";

export type SessionSnapshot = {
  subject_id: string;
  subject_name: string;
  subject_color: string | null;
  preset_id: string | null;
  mode: PresetMode;
  phase: SessionPhase;
  status: "running" | "paused";
  /** Номер рабочей фазы в текущем круге; вне помодоро всегда 1. */
  cycle: number;
  /** Сколько рабочих фаз в круге — для подписи «работа 2/4». */
  cycles_before_long: number | null;
  elapsed_seconds: number;
  /** Время учёбы с начала сессии: все рабочие фазы без перерывов и пауз. */
  session_seconds: number;
  /** `null` у секундомера: ему не к чему идти. */
  remaining_seconds: number | null;
  target_seconds: number | null;
  /** Фаза добрала свою длительность и ждёт перехода. */
  phase_finished: boolean;
  interruptions: number;
  auto_start_next: boolean;
};

/** Ответ на возвращение приложения из фона. */
export type AwayReport = {
  away_seconds: number;
  /** Отсутствие достаточно долгое, чтобы спросить: засчитать или отбросить. */
  needs_decision: boolean;
};

export function startSession(subjectId: string): Promise<SessionSnapshot> {
  return invoke<SessionSnapshot>("start_session", { subjectId });
}

export function sessionSnapshot(): Promise<SessionSnapshot | null> {
  return invoke<SessionSnapshot | null>("session_snapshot");
}

export function pauseSession(): Promise<SessionSnapshot> {
  return invoke<SessionSnapshot>("pause_session");
}

export function resumeSession(): Promise<SessionSnapshot> {
  return invoke<SessionSnapshot>("resume_session");
}

export function markInterruption(): Promise<SessionSnapshot> {
  return invoke<SessionSnapshot>("session_mark_interruption");
}

export function skipPhase(): Promise<SessionSnapshot> {
  return invoke<SessionSnapshot>("session_skip_phase");
}

export function stopSession(): Promise<void> {
  return invoke<void>("stop_session");
}

/** Сообщает бэкенду, когда экран видели в последний раз. */
export function reportReturn(lastSeen: Date): Promise<AwayReport> {
  return invoke<AwayReport>("session_report_return", {
    lastSeen: lastSeen.toISOString(),
  });
}

export function discardAway(since: Date): Promise<SessionSnapshot> {
  return invoke<SessionSnapshot>("session_discard_away", {
    since: since.toISOString(),
  });
}
