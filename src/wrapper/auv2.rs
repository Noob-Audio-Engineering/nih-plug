//! An Audio Unit (AUv2) wrapper.
//!
//! nih-plug exports CLAP and VST3. On macOS that leaves out Logic, which takes
//! Audio Units and nothing else, so a plug-in built with this framework simply
//! does not exist there. The alternative to this backend is to wrap the CLAP
//! with `clap-wrapper`, which works and is what we shipped first, but that
//! puts another layer between the host and the audio and leaves the format in
//! somebody else's hands.
//!
//! # The shape of the thing
//!
//! An Audio Unit is a bundle whose `Info.plist` names a factory function. The
//! host calls it with a description, and gets back an
//! [`AudioComponentPlugInInterface`](sys::AudioComponentPlugInInterface): three
//! function pointers, of which the interesting one is `Lookup`. Everything
//! else --- initialising, properties, parameters, rendering --- is reached by
//! asking `Lookup` for a selector and calling what comes back.
//!
//! That indirection is the whole API. There is no vtable of named methods as
//! in VST3, and no struct of callbacks as in CLAP; there is one function that
//! turns a number into a function pointer, and the signatures those pointers
//! really have are known only by convention.
//!
//! # State
//!
//! **This backend is not finished.** What is here is the entry point, the
//! interface, and the dispatch, which is the part that has to be right before
//! any of the rest can be written or tested. The selectors are wired to
//! implementations that answer honestly --- `kAudioUnitErr_InvalidProperty`
//! rather than a plausible lie --- and are being filled in one at a time.
//!
//! `docs/AUV2-STATUS.md` in this fork tracks what a host can and cannot do
//! yet, and `auval` is the arbiter: a unit that `auval` passes is one Logic
//! will load, and nothing else is evidence.

pub mod properties;
pub mod sys;
pub mod wrapper;

/// Export an Audio Unit entry point for a plugin.
///
/// The bundle's `Info.plist` must name the generated factory function in its
/// `AudioComponents` entry, along with the type, subtype and manufacturer
/// codes. A bundle without that entry loads without complaint and is offered
/// by no host at all.
///
/// ```ignore
/// nih_export_auv2!(MyPlugin);
/// ```
#[macro_export]
macro_rules! nih_export_auv2 {
    ($plugin_ty:ty) => {
        /// The factory the `Info.plist` names. macOS calls this once per
        /// instance, with the description it was asked for.
        ///
        /// # Safety
        ///
        /// Called by the host across an FFI boundary with a pointer to the
        /// description it wants. Returns a pointer the host owns until it
        /// calls `Close`.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn nih_plug_auv2_factory(
            desc: *const $crate::wrapper::auv2::sys::AudioComponentDescription,
        ) -> *mut ::std::os::raw::c_void {
            $crate::wrapper::auv2::wrapper::factory::<$plugin_ty>(desc)
        }
    };
}
