//! Thin Tauri command layer.
//!
//! Commands in this module stay deliberately thin: they parse arguments,
//! delegate to [`crate::core`] / [`crate::db`] / [`crate::platform`], and map
//! the result into something serde can hand to the frontend. No domain logic
//! lives here, so it stays testable without a running Tauri app.

/// Health check for the Rust ↔ TypeScript bridge.
///
/// The frontend calls this on startup; seeing `"pong"` in the window proves
/// the IPC layer is wired up correctly.
#[tauri::command]
pub fn ping() -> String {
    "pong".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ping_returns_pong() {
        assert_eq!(ping(), "pong");
    }
}
