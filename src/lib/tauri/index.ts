/**
 * Typed wrappers around the Tauri command layer.
 *
 * Every `invoke` call in the app goes through this folder, so the Rust command
 * signatures in `src-tauri/src/commands/` are mirrored in exactly one place
 * and the rest of the frontend never touches stringly-typed command names.
 *
 * Split by screen, like the commands themselves; this module re-exports the
 * lot, so `@/lib/tauri` stays the one import everywhere.
 */
import { invoke } from "@tauri-apps/api/core";

/**
 * Health check for the Rust ↔ TypeScript bridge. Resolves to `"pong"`.
 */
export function ping(): Promise<string> {
  return invoke<string>("ping");
}

/** Как команда сообщает об отказе. Зеркало `commands::CommandError`. */
export type CommandError = {
  kind: "validation" | "not_found" | "database";
  message: string;
};

/**
 * Сообщение из отказа команды.
 *
 * `invoke` отклоняет промис тем, что вернул Rust, — это обычный объект, а не
 * `Error`, поэтому `catch (e) { e.message }` молча даёт `undefined`. Разбор
 * живёт здесь, чтобы каждый экран не переизобретал его заново.
 */
export function errorMessage(error: unknown): string {
  if (typeof error === "object" && error !== null && "message" in error) {
    const { message } = error as { message: unknown };
    if (typeof message === "string") return message;
  }
  if (error instanceof Error) return error.message;
  return String(error);
}

export * from "@/lib/tauri/cards";
export * from "@/lib/tauri/desktop";
export * from "@/lib/tauri/duel";
export * from "@/lib/tauri/presets";
export * from "@/lib/tauri/session";
export * from "@/lib/tauri/settings";
export * from "@/lib/tauri/streak";
export * from "@/lib/tauri/stats";
export * from "@/lib/tauri/study";
export * from "@/lib/tauri/subjects";
export * from "@/lib/tauri/today";
