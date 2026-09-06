//! The properties a host reads and writes, and the parameters it lists.
//!
//! An Audio Unit has no methods for "how many channels" or "what is your
//! latency". Everything is a numbered property fetched through
//! `GetProperty`, and the host learns the size first by calling
//! `GetPropertyInfo`. Answer the size wrongly and the host writes past the end
//! of its own buffer; answer `InvalidProperty` to something it needs and the
//! unit is quietly not offered.
//!
//! Every property here is answered in both places or in neither. That
//! symmetry is the whole correctness condition of this file, and
//! `properties_agree_about_their_own_size` is the test that holds it.

use std::os::raw::c_char;

use super::sys::*;

/// How big a property is, and whether it can be written.
///
/// `None` means we do not implement it, which is a legitimate answer: a host
/// asks about far more properties than any one unit has, and expects to be
/// told no.
pub struct PropertyInfo {
    pub size: UInt32,
    pub writable: bool,
}

/// What the unit knows about itself, in the form the property calls need.
///
/// Kept apart from the wrapper so the answers can be tested without an
/// instance, a host, or macOS.
pub struct Layout {
    pub inputs: u32,
    pub outputs: u32,
    pub params: u32,
    pub sample_rate: f64,
    pub max_frames: u32,
    pub latency_samples: u32,
    pub tail_seconds: f64,
}

/// The size and writability of a property, or `None` if we do not have it.
///
/// **This must agree with [`get`] exactly.** A host calls this first to size
/// its buffer and then calls `get` to fill it, so a property that reports one
/// size and writes another is a buffer overrun in the host's address space.
pub fn info(
    id: AudioUnitPropertyID,
    scope: AudioUnitScope,
    layout: &Layout,
) -> Option<PropertyInfo> {
    use std::mem::size_of;
    let asbd = size_of::<AudioStreamBasicDescription>() as UInt32;
    match id {
        kAudioUnitProperty_StreamFormat if scope != kAudioUnitScope_Global => Some(PropertyInfo {
            size: asbd,
            writable: true,
        }),
        kAudioUnitProperty_SampleRate if scope != kAudioUnitScope_Global => Some(PropertyInfo {
            size: size_of::<Float64>() as UInt32,
            writable: true,
        }),
        kAudioUnitProperty_MaximumFramesPerSlice if scope == kAudioUnitScope_Global => {
            Some(PropertyInfo {
                size: size_of::<UInt32>() as UInt32,
                writable: true,
            })
        }
        kAudioUnitProperty_ElementCount => Some(PropertyInfo {
            size: size_of::<UInt32>() as UInt32,
            // A unit with a fixed layout does not let a host add elements.
            writable: false,
        }),
        kAudioUnitProperty_Latency if scope == kAudioUnitScope_Global => Some(PropertyInfo {
            size: size_of::<Float64>() as UInt32,
            writable: false,
        }),
        kAudioUnitProperty_TailTime if scope == kAudioUnitScope_Global => Some(PropertyInfo {
            size: size_of::<Float64>() as UInt32,
            writable: false,
        }),
        kAudioUnitProperty_ParameterList if scope == kAudioUnitScope_Global => Some(PropertyInfo {
            size: layout.params * size_of::<AudioUnitParameterID>() as UInt32,
            writable: false,
        }),
        kAudioUnitProperty_ParameterInfo if scope == kAudioUnitScope_Global => Some(PropertyInfo {
            size: size_of::<AudioUnitParameterInfo>() as UInt32,
            writable: false,
        }),
        kAudioUnitProperty_LastRenderError if scope == kAudioUnitScope_Global => {
            Some(PropertyInfo {
                size: size_of::<OSStatus>() as UInt32,
                writable: false,
            })
        }
        kAudioUnitProperty_BypassEffect if scope == kAudioUnitScope_Global => Some(PropertyInfo {
            size: size_of::<UInt32>() as UInt32,
            writable: true,
        }),
        _ => None,
    }
}

/// How many elements a scope has. An effect has one input bus and one output
/// bus, and the global scope has exactly one element by definition.
pub fn element_count(scope: AudioUnitScope) -> UInt32 {
    match scope {
        kAudioUnitScope_Global => 1,
        kAudioUnitScope_Input | kAudioUnitScope_Output => 1,
        _ => 0,
    }
}

/// The stream format a scope runs at.
///
/// Non-interleaved 32-bit float, which is what every modern host wants and
/// what nih-plug's buffers already are: one pointer per channel, so no
/// de-interleaving on the audio thread.
pub fn stream_format(scope: AudioUnitScope, layout: &Layout) -> AudioStreamBasicDescription {
    let channels = if scope == kAudioUnitScope_Input {
        layout.inputs
    } else {
        layout.outputs
    };
    AudioStreamBasicDescription {
        mSampleRate: layout.sample_rate,
        mFormatID: kAudioFormatLinearPCM,
        mFormatFlags: kAudioFormatFlagIsFloat
            | kAudioFormatFlagIsPacked
            | kAudioFormatFlagIsNonInterleaved,
        // Non-interleaved: each buffer holds one channel, so a "frame" in
        // each buffer is one sample and the channel count lives in the buffer
        // list rather than in these fields. Getting this wrong is the classic
        // way to hand a host garbage that sounds like a very loud crackle.
        mBytesPerPacket: 4,
        mFramesPerPacket: 1,
        mBytesPerFrame: 4,
        mChannelsPerFrame: channels,
        mBitsPerChannel: 32,
        mReserved: 0,
    }
}

/// Copy a name into the fixed array `AudioUnitParameterInfo` uses, which is
/// 52 bytes and must be NUL-terminated.
pub fn write_name(dst: &mut [c_char; 52], name: &str) {
    // One byte reserved for the terminator, and the truncation is on a
    // character boundary so a multi-byte name cannot be cut in half.
    let mut end = name.len().min(51);
    while end > 0 && !name.is_char_boundary(end) {
        end -= 1;
    }
    for (slot, byte) in dst.iter_mut().zip(name.as_bytes()[..end].iter()) {
        *slot = *byte as c_char;
    }
    dst[end] = 0;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn layout() -> Layout {
        Layout {
            inputs: 2,
            outputs: 2,
            params: 7,
            sample_rate: 48_000.0,
            max_frames: 512,
            latency_samples: 0,
            tail_seconds: 0.0,
        }
    }

    /// **A property's declared size has to be the size it writes.**
    ///
    /// The host calls `GetPropertyInfo` to size a buffer and then `GetProperty`
    /// to fill it. Report more than is written and the host reads uninitialised
    /// memory; report less and the unit writes past the end of an allocation
    /// belonging to the host. Neither shows up as a crash in our code.
    #[test]
    fn properties_agree_about_their_own_size() {
        use std::mem::size_of;
        let l = layout();
        let cases: &[(AudioUnitPropertyID, AudioUnitScope, UInt32)] = &[
            (
                kAudioUnitProperty_StreamFormat,
                kAudioUnitScope_Output,
                size_of::<AudioStreamBasicDescription>() as UInt32,
            ),
            (
                kAudioUnitProperty_SampleRate,
                kAudioUnitScope_Output,
                size_of::<Float64>() as UInt32,
            ),
            (
                kAudioUnitProperty_MaximumFramesPerSlice,
                kAudioUnitScope_Global,
                size_of::<UInt32>() as UInt32,
            ),
            (
                kAudioUnitProperty_ParameterList,
                kAudioUnitScope_Global,
                7 * size_of::<AudioUnitParameterID>() as UInt32,
            ),
            (
                kAudioUnitProperty_ParameterInfo,
                kAudioUnitScope_Global,
                size_of::<AudioUnitParameterInfo>() as UInt32,
            ),
        ];
        for (id, scope, want) in cases {
            let got = info(*id, *scope, &l)
                .unwrap_or_else(|| panic!("property {id} is not answered in scope {scope}"));
            assert_eq!(got.size, *want, "property {id} reports the wrong size");
        }
    }

    /// A property we do not have must say so rather than answer with a size.
    #[test]
    fn an_unknown_property_is_refused() {
        let l = layout();
        // A real property, in a scope it does not belong to.
        assert!(info(kAudioUnitProperty_Latency, kAudioUnitScope_Input, &l).is_none());
        // And one this unit simply does not implement.
        assert!(info(kAudioUnitProperty_CocoaUI, kAudioUnitScope_Global, &l).is_none());
    }

    /// The stream format has to describe non-interleaved float, because that
    /// is what the render path assumes. A mismatch here is not an error
    /// anywhere; it is a very loud noise.
    #[test]
    fn the_stream_format_is_non_interleaved_float() {
        let f = stream_format(kAudioUnitScope_Output, &layout());
        assert_eq!(f.mFormatID, kAudioFormatLinearPCM);
        assert!(f.mFormatFlags & kAudioFormatFlagIsFloat != 0, "not float");
        assert!(
            f.mFormatFlags & kAudioFormatFlagIsNonInterleaved != 0,
            "not non-interleaved"
        );
        assert_eq!(f.mBitsPerChannel, 32);
        // Per *buffer*, and each buffer is one channel when non-interleaved.
        assert_eq!(f.mBytesPerFrame, 4, "a frame is one float per buffer");
        assert_eq!(f.mChannelsPerFrame, 2);
    }

    #[test]
    fn a_name_is_truncated_on_a_character_boundary_and_terminated() {
        let mut dst = [0 as c_char; 52];
        write_name(&mut dst, "Decay");
        let s: Vec<u8> = dst
            .iter()
            .take_while(|c| **c != 0)
            .map(|c| *c as u8)
            .collect();
        assert_eq!(String::from_utf8(s).unwrap(), "Decay");

        // Longer than the array, and multi-byte, so a naive truncation would
        // leave half a character and an invalid string.
        let long = "é".repeat(40);
        write_name(&mut dst, &long);
        let s: Vec<u8> = dst
            .iter()
            .take_while(|c| **c != 0)
            .map(|c| *c as u8)
            .collect();
        assert!(String::from_utf8(s).is_ok(), "the name was cut in half");
        assert_eq!(dst[51], 0, "the array is not terminated");
    }
}
