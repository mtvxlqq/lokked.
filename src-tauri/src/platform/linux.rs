//! Linux backend (Fedora / GNOME / Wayland is the reference target).
//!
//! TODO: `inhibit_sleep` — take a D-Bus inhibitor. Preferred order:
//!       `org.freedesktop.portal.Inhibit` (works inside Flatpak),
//!       then `org.gnome.SessionManager`, then `org.freedesktop.ScreenSaver`.
//!       Hold the returned cookie/fd in `self` and drop it in `release_sleep`.
//! TODO: `notify` — `org.freedesktop.Notifications`, or Tauri's notification
//!       plugin if we end up wanting one code path across desktop platforms.

use super::{PlatformError, PlatformServices};

/// Sleep inhibition and notifications via D-Bus.
#[derive(Debug, Default)]
pub struct LinuxPlatform {
    // TODO: hold the inhibitor cookie / fd returned by the portal here.
}

impl PlatformServices for LinuxPlatform {
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
