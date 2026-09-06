//! The parts of Apple's AudioUnit C API this wrapper uses.
//!
//! Declared here rather than pulled in from `coreaudio-sys` on purpose. That
//! crate runs `bindgen` at build time, which means libclang on every machine
//! and every CI runner that touches this fork, to produce several thousand
//! declarations of which we want about forty. The forty are stable --- this is
//! a C API that has not moved since it was documented --- and having them in
//! one readable file is worth more here than having them generated.
//!
//! Names and layouts follow `AudioToolbox/AudioComponent.h`,
//! `AudioToolbox/AUComponent.h` and `CoreAudioTypes/CoreAudioBaseTypes.h`.

#![allow(
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    dead_code
)]

use std::os::raw::{c_char, c_void};

pub type OSStatus = i32;
pub type OSType = u32;
pub type UInt32 = u32;
pub type SInt16 = i16;
pub type SInt32 = i32;
pub type Float32 = f32;
pub type Float64 = f64;
pub type Boolean = u8;

/// Everything went well.
pub const noErr: OSStatus = 0;

// The errors an Audio Unit is expected to be able to return. A host reads
// these; returning the wrong one is how a plug-in comes to be described as
// "not working in Logic" with nothing in any log.
pub const kAudioUnitErr_InvalidProperty: OSStatus = -10879;
pub const kAudioUnitErr_InvalidParameter: OSStatus = -10878;
pub const kAudioUnitErr_InvalidElement: OSStatus = -10877;
pub const kAudioUnitErr_NoConnection: OSStatus = -10876;
pub const kAudioUnitErr_FailedInitialization: OSStatus = -10875;
pub const kAudioUnitErr_TooManyFramesToProcess: OSStatus = -10874;
pub const kAudioUnitErr_InvalidPropertyValue: OSStatus = -10851;
pub const kAudioUnitErr_PropertyNotInUse: OSStatus = -10850;
pub const kAudioUnitErr_Initialized: OSStatus = -10849;
pub const kAudioUnitErr_InvalidOfflineRender: OSStatus = -10848;
pub const kAudioUnitErr_Unauthorized: OSStatus = -10847;
pub const kAudioUnitErr_Uninitialized: OSStatus = -10867;

/// A four-character code, written the way Apple writes them.
pub const fn fourcc(s: &[u8; 4]) -> OSType {
    ((s[0] as u32) << 24) | ((s[1] as u32) << 16) | ((s[2] as u32) << 8) | (s[3] as u32)
}

/// The component types an Audio Unit can be. An effect is `aufx` and an
/// instrument is `aumu`, and the choice decides which list in the host the
/// plug-in appears in.
pub const kAudioUnitType_Effect: OSType = fourcc(b"aufx");
pub const kAudioUnitType_MusicDevice: OSType = fourcc(b"aumu");
pub const kAudioUnitType_MusicEffect: OSType = fourcc(b"aumf");

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AudioComponentDescription {
    pub componentType: OSType,
    pub componentSubType: OSType,
    pub componentManufacturer: OSType,
    pub componentFlags: UInt32,
    pub componentFlagsMask: UInt32,
}

pub type AudioComponentInstance = *mut c_void;
pub type AudioUnit = AudioComponentInstance;

/// The generic shape of every selector's implementation. The real signatures
/// differ per selector and are cast to and from this; that is how the C API
/// itself is defined and not something invented here.
pub type AudioComponentMethod = Option<unsafe extern "C" fn() -> OSStatus>;

/// What a factory function hands back: the vtable a host drives the plug-in
/// through.
#[repr(C)]
pub struct AudioComponentPlugInInterface {
    pub Open: Option<
        unsafe extern "C" fn(this: *mut c_void, inInstance: AudioComponentInstance) -> OSStatus,
    >,
    pub Close: Option<unsafe extern "C" fn(this: *mut c_void) -> OSStatus>,
    pub Lookup: Option<unsafe extern "C" fn(selector: SInt16) -> AudioComponentMethod>,
    pub reserved: *mut c_void,
}

// The selectors a host looks up. These are the whole API surface: a host asks
// `Lookup` for each one it wants and calls what comes back.
pub const kAudioUnitInitializeSelect: SInt16 = 0x0001;
pub const kAudioUnitUninitializeSelect: SInt16 = 0x0002;
pub const kAudioUnitGetPropertyInfoSelect: SInt16 = 0x0003;
pub const kAudioUnitGetPropertySelect: SInt16 = 0x0004;
pub const kAudioUnitSetPropertySelect: SInt16 = 0x0005;
pub const kAudioUnitAddPropertyListenerSelect: SInt16 = 0x000A;
pub const kAudioUnitRemovePropertyListenerSelect: SInt16 = 0x000B;
pub const kAudioUnitRemovePropertyListenerWithUserDataSelect: SInt16 = 0x0012;
pub const kAudioUnitAddRenderNotifySelect: SInt16 = 0x000F;
pub const kAudioUnitRemoveRenderNotifySelect: SInt16 = 0x0010;
pub const kAudioUnitGetParameterSelect: SInt16 = 0x0006;
pub const kAudioUnitSetParameterSelect: SInt16 = 0x0007;
pub const kAudioUnitScheduleParametersSelect: SInt16 = 0x0011;
pub const kAudioUnitRenderSelect: SInt16 = 0x000E;
pub const kAudioUnitResetSelect: SInt16 = 0x0009;
pub const kAudioUnitComplexRenderSelect: SInt16 = 0x0013;
pub const kAudioUnitProcessSelect: SInt16 = 0x0014;
pub const kAudioUnitProcessMultipleSelect: SInt16 = 0x0015;

// The properties a host asks about. Far from all of them; these are the ones
// without which a unit does not load.
pub const kAudioUnitProperty_ClassInfo: UInt32 = 0;
pub const kAudioUnitProperty_MakeConnection: UInt32 = 1;
pub const kAudioUnitProperty_SampleRate: UInt32 = 2;
pub const kAudioUnitProperty_ParameterList: UInt32 = 3;
pub const kAudioUnitProperty_ParameterInfo: UInt32 = 4;
pub const kAudioUnitProperty_StreamFormat: UInt32 = 8;
pub const kAudioUnitProperty_ElementCount: UInt32 = 11;
pub const kAudioUnitProperty_Latency: UInt32 = 12;
pub const kAudioUnitProperty_SupportedNumChannels: UInt32 = 13;
pub const kAudioUnitProperty_MaximumFramesPerSlice: UInt32 = 14;
pub const kAudioUnitProperty_TailTime: UInt32 = 20;
pub const kAudioUnitProperty_BypassEffect: UInt32 = 21;
pub const kAudioUnitProperty_LastRenderError: UInt32 = 22;
pub const kAudioUnitProperty_SetRenderCallback: UInt32 = 23;
pub const kAudioUnitProperty_FactoryPresets: UInt32 = 24;
pub const kAudioUnitProperty_PresentPreset: UInt32 = 36;
pub const kAudioUnitProperty_ElementName: UInt32 = 30;
pub const kAudioUnitProperty_CocoaUI: UInt32 = 31;
pub const kAudioUnitProperty_SupportedChannelLayoutTags: UInt32 = 32;
pub const kAudioUnitProperty_ParameterValueStrings: UInt32 = 16;
pub const kAudioUnitProperty_AudioChannelLayout: UInt32 = 19;
pub const kAudioUnitProperty_ShouldAllocateBuffer: UInt32 = 51;
pub const kAudioUnitProperty_LastRenderSampleTime: UInt32 = 61;

/// Which side of the unit a property is about.
pub type AudioUnitScope = UInt32;
pub const kAudioUnitScope_Global: AudioUnitScope = 0;
pub const kAudioUnitScope_Input: AudioUnitScope = 1;
pub const kAudioUnitScope_Output: AudioUnitScope = 2;

pub type AudioUnitElement = UInt32;
pub type AudioUnitPropertyID = UInt32;
pub type AudioUnitParameterID = UInt32;
pub type AudioUnitParameterValue = Float32;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct AudioStreamBasicDescription {
    pub mSampleRate: Float64,
    pub mFormatID: OSType,
    pub mFormatFlags: UInt32,
    pub mBytesPerPacket: UInt32,
    pub mFramesPerPacket: UInt32,
    pub mBytesPerFrame: UInt32,
    pub mChannelsPerFrame: UInt32,
    pub mBitsPerChannel: UInt32,
    pub mReserved: UInt32,
}

pub const kAudioFormatLinearPCM: OSType = fourcc(b"lpcm");
pub const kAudioFormatFlagIsFloat: UInt32 = 1 << 0;
pub const kAudioFormatFlagIsPacked: UInt32 = 1 << 3;
pub const kAudioFormatFlagIsNonInterleaved: UInt32 = 1 << 5;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct AudioBuffer {
    pub mNumberChannels: UInt32,
    pub mDataByteSize: UInt32,
    pub mData: *mut c_void,
}

/// A variable-length struct: `mBuffers` is declared as one element in C and
/// is actually `mNumberBuffers` of them. Never construct one by value; always
/// walk it from a pointer the host gave you.
#[repr(C)]
pub struct AudioBufferList {
    pub mNumberBuffers: UInt32,
    pub mBuffers: [AudioBuffer; 1],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct SMPTETime {
    pub mSubframes: SInt16,
    pub mSubframeDivisor: SInt16,
    pub mCounter: UInt32,
    pub mType: UInt32,
    pub mFlags: UInt32,
    pub mHours: SInt16,
    pub mMinutes: SInt16,
    pub mSeconds: SInt16,
    pub mFrames: SInt16,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct AudioTimeStamp {
    pub mSampleTime: Float64,
    pub mHostTime: u64,
    pub mRateScalar: Float64,
    pub mWordClockTime: u64,
    pub mSMPTETime: SMPTETime,
    pub mFlags: UInt32,
    pub mReserved: UInt32,
}

pub type AudioUnitRenderActionFlags = UInt32;
pub const kAudioUnitRenderAction_PreRender: AudioUnitRenderActionFlags = 1 << 2;
pub const kAudioUnitRenderAction_PostRender: AudioUnitRenderActionFlags = 1 << 3;

/// What a parameter looks like to a host's generic interface.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct AudioUnitParameterInfo {
    pub name: [c_char; 52],
    pub unitName: *const c_void,
    pub clumpID: UInt32,
    pub cfNameString: *const c_void,
    pub unit: UInt32,
    pub minValue: AudioUnitParameterValue,
    pub maxValue: AudioUnitParameterValue,
    pub defaultValue: AudioUnitParameterValue,
    pub flags: UInt32,
}

pub const kAudioUnitParameterUnit_Generic: UInt32 = 0;
pub const kAudioUnitParameterUnit_Indexed: UInt32 = 1;
pub const kAudioUnitParameterUnit_Boolean: UInt32 = 2;
pub const kAudioUnitParameterUnit_Percent: UInt32 = 3;
pub const kAudioUnitParameterUnit_Seconds: UInt32 = 4;
pub const kAudioUnitParameterUnit_Hertz: UInt32 = 9;
pub const kAudioUnitParameterUnit_Decibels: UInt32 = 13;
pub const kAudioUnitParameterUnit_Milliseconds: UInt32 = 18;

pub const kAudioUnitParameterFlag_CFNameRelease: UInt32 = 1 << 4;
pub const kAudioUnitParameterFlag_IsHighResolution: UInt32 = 1 << 23;
pub const kAudioUnitParameterFlag_IsReadable: UInt32 = 1 << 30;
pub const kAudioUnitParameterFlag_IsWritable: UInt32 = 1 << 31;
