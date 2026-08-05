//! Android / iOS backend.
//!
//! Both mobile platforms share this module because the shape of the problem is
//! the same — the OS may suspend the app at any time, so a running timer has
//! to survive as a wall-clock interval rather than as a live counter.
//!
//! TODO (Android): `inhibit_sleep` — `FLAG_KEEP_SCREEN_ON` or a
//!       `PARTIAL_WAKE_LOCK` from a foreground service, reached through a
//!       Tauri mobile plugin. `notify` — `NotificationManager` channel.
//! TODO (iOS): `inhibit_sleep` — `UIApplication.isIdleTimerDisabled`.
//!       `notify` — `UNUserNotificationCenter` local notifications.

use super::{PlatformError, PlatformServices};

/// Wake locks and local notifications on Android and iOS.
#[derive(Debug, Default)]
pub struct MobilePlatform {
    // TODO: handle to the wake lock / idle-timer state.
}

impl PlatformServices for MobilePlatform {
    fn inhibit_sleep(&mut self) -> Result<(), PlatformError> {
        Ok(())
    }

    fn release_sleep(&mut self) -> Result<(), PlatformError> {
        Ok(())
    }

    fn notify(&self, _title: &str, _body: &str) -> Result<(), PlatformError> {
        Ok(())
    }
}
