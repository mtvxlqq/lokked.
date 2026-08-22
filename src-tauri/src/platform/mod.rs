//! OS-specific services behind a single trait.
//!
//! Everything the app needs from the host OS — keeping the machine awake
//! during a study session, showing a notification — goes through
//! [`PlatformServices`]. The rest of the codebase depends on the trait, never
//! on a concrete backend, so [`crate::core`] stays portable and testable.
//!
//! Reading the wall clock is a host service too, so it lives here as well, in
//! [`clock::SystemClock`]. It sits outside `PlatformServices` because it needs
//! no per-target implementation and because [`crate::core`] consumes it
//! through its own [`crate::core::clock::Clock`] trait.
//!
//! A backend that cannot compile off its own OS is gated at its `mod`
//! declaration, and so is its dependency in `Cargo.toml`: `linux` needs
//! `zbus` and `windows` calls Win32 directly, so each exists only where it
//! runs. Forgetting the gate does not show up on the machine that has the
//! dependency — it shows up on the other one, as «unresolved crate zbus» in
//! a Windows build. The stubs that are still portable stay declared
//! unconditionally, so `cargo clippy` and `cargo fmt` cover them on every
//! host.

use std::error::Error;
use std::fmt;

pub mod clock;
#[cfg(target_os = "linux")]
pub mod linux;
pub mod mobile;
pub mod noop;
pub mod shortcuts;
#[cfg(target_os = "windows")]
pub mod windows;

/// Something the host OS refused or could not do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlatformError {
    /// The current platform has no implementation for this operation.
    Unsupported(&'static str),
    /// The OS backend was reachable but returned an error.
    Backend(String),
}

impl fmt::Display for PlatformError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unsupported(what) => write!(f, "unsupported on this platform: {what}"),
            Self::Backend(msg) => write!(f, "platform backend error: {msg}"),
        }
    }
}

impl Error for PlatformError {}

/// What happened to the machine's power state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SleepEvent {
    /// The system is about to suspend. Whatever has to be written down has
    /// to be written down now.
    GoingToSleep,
    /// The system came back. However long that took, it was not study time.
    WokeUp,
}

/// What to run when the machine falls asleep or wakes up.
///
/// Called from a background thread owned by the backend, so it has to be
/// `Send + Sync` and must not block for long.
pub type SleepWatcher = Box<dyn Fn(SleepEvent) + Send + Sync + 'static>;

/// Host services a study session needs.
///
/// Implementations are expected to be cheap to construct and safe to share
/// across threads, since a single instance is held in Tauri's managed state.
///
/// `inhibit_sleep` / `release_sleep` are paired and reference-counted by the
/// caller: the app inhibits when a timer starts running and releases when it
/// stops. Implementations should tolerate a release without a matching
/// inhibit rather than panicking.
pub trait PlatformServices: Send + Sync {
    /// Ask the OS not to suspend the machine or blank the screen.
    fn inhibit_sleep(&mut self) -> Result<(), PlatformError>;

    /// Drop a previously acquired sleep inhibitor.
    fn release_sleep(&mut self) -> Result<(), PlatformError>;

    /// Show a user-visible notification.
    fn notify(&self, title: &str, body: &str) -> Result<(), PlatformError>;

    /// Watch for the machine suspending and resuming.
    ///
    /// A laptop lid closed mid-session must not count as an hour of study,
    /// so the session is paused on the way down and the student is asked
    /// about the gap on the way back up.
    ///
    /// The default is «this platform cannot tell»: on Android the app is
    /// suspended rather than the machine, and the timer already survives
    /// that by deriving elapsed time from timestamps.
    fn watch_sleep(&mut self, on_event: SleepWatcher) -> Result<(), PlatformError> {
        drop(on_event);

        Err(PlatformError::Unsupported("sleep notifications"))
    }
}

/// The app's single [`PlatformServices`] instance, ready to be handed to
/// Tauri's managed state.
///
/// `inhibit_sleep` takes `&mut self`, so the trait object needs a lock; this
/// wrapper is that lock, and it keeps `Mutex<Box<dyn …>>` out of every command
/// signature. Poisoning is treated as fatal, same as [`crate::db::Database`].
pub struct SharedPlatform(std::sync::Mutex<Box<dyn PlatformServices>>);

impl SharedPlatform {
    pub fn new(services: Box<dyn PlatformServices>) -> Self {
        Self(std::sync::Mutex::new(services))
    }

    /// Start watching for suspend and resume, if the platform can say.
    ///
    /// Like [`keep_awake`](Self::keep_awake), a refusal is swallowed: on a
    /// desktop without `logind` the timer still works, it just cannot tell
    /// a suspended hour from a studied one until the student says so.
    pub fn watch_sleep(&self, on_event: SleepWatcher) {
        let mut services = self.0.lock().expect("platform mutex poisoned");
        let _ = services.watch_sleep(on_event);
    }

    /// Ask the OS not to blank the screen while a work phase is running, or
    /// let it again once one is not.
    ///
    /// Failures are swallowed on purpose: a desktop portal that refuses to
    /// inhibit is a worse screensaver, not a broken timer, and there is
    /// nothing useful to tell the student about it mid-session.
    pub fn keep_awake(&self, awake: bool) {
        let mut services = self.0.lock().expect("platform mutex poisoned");
        let _ = if awake {
            services.inhibit_sleep()
        } else {
            services.release_sleep()
        };
    }
}

impl Default for SharedPlatform {
    fn default() -> Self {
        Self::new(platform_services())
    }
}

/// Build the [`PlatformServices`] implementation for the current target.
///
/// Falls back to [`noop::NoopPlatform`] on targets we do not ship to, so the
/// crate still builds and tests everywhere.
pub fn platform_services() -> Box<dyn PlatformServices> {
    #[cfg(target_os = "linux")]
    {
        Box::new(linux::LinuxPlatform::default())
    }
    #[cfg(target_os = "windows")]
    {
        Box::new(windows::WindowsPlatform::default())
    }
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        Box::new(mobile::MobilePlatform::default())
    }
    #[cfg(not(any(
        target_os = "linux",
        target_os = "windows",
        target_os = "android",
        target_os = "ios"
    )))]
    {
        // Constructed directly rather than via `default()`, which clippy flags
        // as `default_constructed_unit_structs` for a unit struct.
        Box::new(noop::NoopPlatform)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn platform_services_builds_for_the_host_target() {
        let mut services = platform_services();
        // Whether the OS actually inhibits anything depends on the machine
        // this runs on — a build server has no session bus and is expected
        // to refuse. What is asserted here is that the trait object is wired
        // up and that a refusal comes back as an error instead of a panic.
        let _ = services.inhibit_sleep();
        let _ = services.release_sleep();
        assert!(services.notify("Lokked", "skeleton").is_ok());
    }
}
