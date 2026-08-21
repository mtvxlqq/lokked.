/**
 * Десктопные события: горячая клавиша через второй запуск и сон машины.
 *
 * Имена событий продублированы в `src-tauri/src/desktop.rs` — это контракт
 * между процессами, и другого места для него нет.
 */

import { invoke } from "@tauri-apps/api/core";

/** Открыть чёрный экран: пришло `lokked --zen`. */
export const ZEN_EVENT = "lokked://zen";

/** Машина проснулась, сессия стоит на паузе. */
export const WOKE_EVENT = "lokked://woke";

export type WokeUp = {
  /** Сколько машина спала, в секундах. */
  asleep_seconds: number;
};

/**
 * Просили ли открыть чёрный экран ещё до того, как окно появилось.
 *
 * Отвечает `true` один раз: перезагрузка фронтенда не должна снова уводить
 * студента на чёрный экран.
 */
export function cliPendingZen(): Promise<boolean> {
  return invoke<boolean>("cli_pending_zen");
}
