//! Wrappers for different plugin types. Each wrapper has an entry point macro that you can pass the
//! name of a type that implements `Plugin` to. The macro will handle the rest.

pub mod clap;
pub mod state;
pub(crate) mod util;

// macOS only: there is no AudioToolbox anywhere else, and gating it here
// means a Linux or Windows build never has to reason about it.
#[cfg(all(feature = "auv2", target_os = "macos"))]
pub mod auv2;
#[cfg(feature = "standalone")]
pub mod standalone;
#[cfg(feature = "vst3")]
pub mod vst3;

// This is used by the wrappers.
pub use util::setup_logger;
