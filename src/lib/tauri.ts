/**
 * Typed wrappers around the Tauri command layer.
 *
 * Every `invoke` call in the app goes through this module, so the Rust command
 * signatures in `src-tauri/src/commands.rs` are mirrored in exactly one place
 * and the rest of the frontend never touches stringly-typed command names.
 */
import { invoke } from "@tauri-apps/api/core";

/**
 * Health check for the Rust ↔ TypeScript bridge. Resolves to `"pong"`.
 */
export function ping(): Promise<string> {
  return invoke<string>("ping");
}
