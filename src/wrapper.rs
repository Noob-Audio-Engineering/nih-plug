//! Wrappers for different plugin types. Each wrapper has an entry point macro that you can pass the
//! name of a type that implements `Plugin` to. The macro will handle the rest.

pub mod clap;
pub mod state;
pub(crate) mod util;

// macOS only: there is no AudioToolbox anywhere else, and gating it here
// means a Linux or Windows build never has to reason about it.
// Gated on the feature and not on the platform. None of this links to
// AudioToolbox --- `sys` is declarations and the rest is exported C
// functions --- so it compiles anywhere, which means its tests run
// anywhere. An Audio Unit that can only be type-checked on a macOS
// runner is one whose every mistake costs a five-minute round trip.
#[cfg(feature = "auv2")]
pub mod auv2;
#[cfg(feature = "standalone")]
pub mod standalone;
#[cfg(feature = "vst3")]
pub mod vst3;

// This is used by the wrappers.
pub use util::setup_logger;
