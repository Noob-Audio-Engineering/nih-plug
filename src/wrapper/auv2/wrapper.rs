//! The instance a host drives, and the dispatch that reaches it.

use std::os::raw::c_void;
use std::sync::Arc;

use super::sys::*;
use crate::plugin::Plugin;

/// One Audio Unit instance.
///
/// The host holds a pointer to this for the life of the unit and hands it
/// back to every selector, which is why the first field of the allocation has
/// to be the interface: the host is given `&self.interface` and casts back.
#[repr(C)]
pub struct Wrapper<P: Plugin> {
    /// **Must stay first.** The host is handed a pointer to this struct and
    /// treats it as an `AudioComponentPlugInInterface`, so the vtable has to
    /// be at offset zero.
    interface: AudioComponentPlugInInterface,

    /// The instance the host gave us in `Open`, which identifies this unit in
    /// every callback it makes.
    instance: AudioComponentInstance,

    plugin: Arc<parking_lot::Mutex<P>>,

    /// Set by `Initialize` and cleared by `Uninitialize`. Several selectors
    /// are only legal on one side of that line and a host will check.
    initialised: bool,

    /// What the host told us, before it is allowed to start.
    sample_rate: f64,
    max_frames: u32,
}

impl<P: Plugin> Wrapper<P> {
    fn new() -> Self {
        Wrapper {
            interface: AudioComponentPlugInInterface {
                Open: Some(open::<P>),
                Close: Some(close::<P>),
                Lookup: Some(lookup::<P>),
                reserved: std::ptr::null_mut(),
            },
            instance: std::ptr::null_mut(),
            plugin: Arc::new(parking_lot::Mutex::new(P::default())),
            initialised: false,
            sample_rate: 44_100.0,
            max_frames: 512,
        }
    }
}

/// The entry point named by the bundle's `Info.plist`.
///
/// # Safety
///
/// `desc` is the description the host wants, or null. The returned pointer is
/// the host's until it calls `Close`.
pub unsafe fn factory<P: Plugin>(_desc: *const AudioComponentDescription) -> *mut c_void {
    // Leaked deliberately: the host owns this now and gives it back through
    // `Close`, which is where it is reclaimed.
    let wrapper = Box::new(Wrapper::<P>::new());
    Box::into_raw(wrapper) as *mut c_void
}

/// # Safety
///
/// `this` is a pointer this crate returned from [`factory`].
unsafe extern "C" fn open<P: Plugin>(
    this: *mut c_void,
    instance: AudioComponentInstance,
) -> OSStatus {
    if this.is_null() {
        return kAudioUnitErr_FailedInitialization;
    }
    let wrapper = unsafe { &mut *(this as *mut Wrapper<P>) };
    wrapper.instance = instance;
    noErr
}

/// # Safety
///
/// `this` is a pointer this crate returned from [`factory`], and the host will
/// not use it again.
unsafe extern "C" fn close<P: Plugin>(this: *mut c_void) -> OSStatus {
    if this.is_null() {
        return noErr;
    }
    // Taking it back into a `Box` is what frees it.
    drop(unsafe { Box::from_raw(this as *mut Wrapper<P>) });
    noErr
}

/// Turn a selector into the function that implements it.
///
/// A selector we do not implement must return `None` rather than a function
/// that fails: the host reads `None` as "this unit does not do that" and works
/// around it, and reads a failing function as a broken unit.
///
/// # Safety
///
/// Called by the host. The returned pointer is transmuted by the caller to the
/// signature that selector is defined to have, which is how this C API works.
unsafe extern "C" fn lookup<P: Plugin>(selector: SInt16) -> AudioComponentMethod {
    // Every arm casts a correctly-typed function to the generic one the API is
    // declared with. The cast is the API's, not ours: `AudioComponentMethod`
    // is deliberately signature-less and each selector's real signature is
    // fixed by convention.
    match selector {
        kAudioUnitInitializeSelect => Some(unsafe {
            std::mem::transmute::<
                unsafe extern "C" fn(*mut c_void) -> OSStatus,
                unsafe extern "C" fn() -> OSStatus,
            >(initialize::<P> as unsafe extern "C" fn(*mut c_void) -> OSStatus)
        }),
        kAudioUnitUninitializeSelect => Some(unsafe {
            std::mem::transmute::<
                unsafe extern "C" fn(*mut c_void) -> OSStatus,
                unsafe extern "C" fn() -> OSStatus,
            >(uninitialize::<P> as unsafe extern "C" fn(*mut c_void) -> OSStatus)
        }),
        // Not yet implemented. `None` is the honest answer and the one a host
        // handles; a stub that returns an error reads as a unit that is broken
        // rather than one that is incomplete.
        _ => None,
    }
}

/// # Safety
///
/// `this` is a live wrapper pointer from the host.
unsafe extern "C" fn initialize<P: Plugin>(this: *mut c_void) -> OSStatus {
    if this.is_null() {
        return kAudioUnitErr_FailedInitialization;
    }
    let wrapper = unsafe { &mut *(this as *mut Wrapper<P>) };
    wrapper.initialised = true;
    noErr
}

/// # Safety
///
/// `this` is a live wrapper pointer from the host.
unsafe extern "C" fn uninitialize<P: Plugin>(this: *mut c_void) -> OSStatus {
    if this.is_null() {
        return noErr;
    }
    let wrapper = unsafe { &mut *(this as *mut Wrapper<P>) };
    wrapper.initialised = false;
    noErr
}
