//! Windows backend.
//!
//! TODO: `inhibit_sleep` — `SetThreadExecutionState` with
//!       `ES_CONTINUOUS | ES_SYSTEM_REQUIRED | ES_DISPLAY_REQUIRED`;
//!       `release_sleep` restores plain `ES_CONTINUOUS`. Note the flag is
//!       per-thread, so it must be set from a thread we keep alive.
//! TODO: `notify` — toast notifications via WinRT / the Tauri plugin.
//! TODO: `watch_sleep` — `WM_POWERBROADCAST` with `PBT_APMSUSPEND` and
//!       `PBT_APMRESUMEAUTOMATIC`, which needs a window procedure to hook
//!       into (этап M14). Until then the default «cannot tell» applies.

use super::{PlatformError, PlatformServices};

/// Sleep inhibition and notifications via the Win32 / WinRT APIs.
#[derive(Debug, Default)]
pub struct WindowsPlatform {
    // TODO: track whether the execution state is currently held.
}

impl PlatformServices for WindowsPlatform {
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
