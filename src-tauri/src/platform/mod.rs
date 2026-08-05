//! OS-specific services behind a single trait.
//!
//! Everything the app needs from the host OS — keeping the machine awake
//! during a study session, showing a notification — goes through
//! [`PlatformServices`]. The rest of the codebase depends on the trait, never
//! on a concrete backend, so [`crate::core`] stays portable and testable.
//!
//! All backend modules are declared unconditionally while they are still
//! stubs, so `cargo clippy` and `cargo fmt` cover every one of them on every
//! host. Once a backend pulls in OS-specific dependencies, gate its `mod`
//! declaration with `#[cfg(target_os = "…")]` and gate the dependency itself
//! with `[target.'cfg(…)'.dependencies]` in `Cargo.toml`.

use std::error::Error;
use std::fmt;

pub mod linux;
pub mod mobile;
pub mod noop;
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
        // Stubs are no-ops today; this asserts the trait object is wired up,
        // not that the OS actually did anything.
        assert!(services.inhibit_sleep().is_ok());
        assert!(services.release_sleep().is_ok());
        assert!(services.notify("StudyApp", "skeleton").is_ok());
    }
}
