//! Fallback backend for targets Lokked does not ship to (macOS, BSDs, …).
//!
//! Every method succeeds without doing anything, so the crate builds and
//! `cargo test` runs on any host a contributor happens to use.

use super::{PlatformError, PlatformServices};

/// Does nothing, successfully.
#[derive(Debug, Default)]
pub struct NoopPlatform;

impl PlatformServices for NoopPlatform {
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
