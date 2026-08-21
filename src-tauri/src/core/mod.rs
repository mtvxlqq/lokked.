//! Pure domain logic for Lokked.
//!
//! Everything under `core` must stay free of Tauri, of the filesystem and of
//! the database, so it can be unit-tested with plain `cargo test`. Anything
//! that needs the outside world (wall-clock time, storage, notifications) is
//! injected as a trait — see [`clock`] and [`crate::platform`].
//!
//! Referenced as `crate::core::…`; a bare `core::…` would resolve to Rust's
//! built-in `core` crate instead.

pub mod card;
pub mod clock;
pub mod dayline;
pub mod deck;
pub mod import;
pub mod preset;
pub mod scheduler;
pub mod session;
pub mod settings;
pub mod stats;
pub mod subject;
pub mod timer;
