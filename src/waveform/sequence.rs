//! Sequence construction for waveform memory.

use crate::errors::Error;
use super::frame::Frame;

/// Maximum number of frames per sequence.
/// Limited to keep total memory under 100 bytes.
pub const MAX_FRAMES_PER_SEQUENCE: usize = 32;

/// Maximum total bytes per sequence.
pub const MAX_SEQUENCE_BYTES: usize = 96;

/// A sequence of frames to be played back.
///
/// Sequences are collections of frames that are played in order.
/// Each sequence can contain multiple frames, where each frame
/// references a snippet with optional gain, timebase, loop, and
/// frequency overrides.
#[derive(Debug, Clone, Copy)]
pub struct Sequence {
    data: [u8; MAX_SEQUENCE_BYTES],
    len: u8,
}

impl Sequence {
    /// Get the number of bytes this sequence occupies in memory.
    pub fn byte_len(&self) -> usize {
        self.len as usize
    }

    /// Get the raw bytes of this sequence.
    pub fn as_bytes(&self) -> &[u8] {
        &self.data[..self.len as usize]
    }

    /// Encode the sequence into the provided buffer.
    ///
    /// Returns the number of bytes written.
    pub fn encode_into(&self, buffer: &mut [u8]) -> usize {
        let len = (self.len as usize).min(buffer.len());
        buffer[..len].copy_from_slice(&self.data[..len]);
        len
    }
}

/// Builder for constructing sequences.
///
/// # Example
///
/// ```
/// use da728x::waveform::{SequenceBuilder, FrameBuilder, Timebase};
///
/// let frame1 = FrameBuilder::new(1)?.timebase(Timebase::Ms5_44).build()?;
/// let frame2 = FrameBuilder::new(2)?.timebase(Timebase::Ms21_76).build()?;
///
/// let sequence = SequenceBuilder::new()
///     .add_frame(frame1)?
///     .add_frame(frame2)?
///     .build()?;
/// # Ok::<(), da728x::errors::Error>(())
/// ```
#[derive(Debug, Clone, Copy)]
pub struct SequenceBuilder {
    data: [u8; MAX_SEQUENCE_BYTES],
    len: u8,
    frame_count: u8,
}

impl Default for SequenceBuilder {
    fn default() -> Self {
        Self {
            data: [0u8; MAX_SEQUENCE_BYTES],
            len: 0,
            frame_count: 0,
        }
    }
}

impl SequenceBuilder {
    /// Create a new sequence builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a frame to the sequence.
    ///
    /// # Errors
    /// Returns `TooManySequences` if adding this frame would exceed
    /// the maximum sequence size.
    pub fn add_frame(mut self, frame: Frame) -> Result<Self, Error> {
        let frame_bytes = frame.as_bytes();

        // Check if we have room
        if self.len as usize + frame_bytes.len() > MAX_SEQUENCE_BYTES {
            return Err(Error::WaveformMemoryFull);
        }
        if self.frame_count as usize >= MAX_FRAMES_PER_SEQUENCE {
            return Err(Error::TooManySequences);
        }

        for &byte in frame_bytes {
            self.data[self.len as usize] = byte;
            self.len += 1;
        }
        self.frame_count += 1;

        Ok(self)
    }

    /// Get the current byte length of the sequence being built.
    pub fn current_len(&self) -> usize {
        self.len as usize
    }

    /// Build the sequence.
    ///
    /// # Errors
    /// Returns `EmptySequence` if no frames have been added.
    pub fn build(self) -> Result<Sequence, Error> {
        if self.len == 0 {
            return Err(Error::EmptySequence);
        }
        Ok(Sequence { data: self.data, len: self.len })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::waveform::FrameBuilder;

    #[test]
    fn test_sequence_single_frame() {
        let frame = FrameBuilder::new(1).unwrap().build().unwrap();
        let sequence = SequenceBuilder::new()
            .add_frame(frame)
            .unwrap()
            .build()
            .unwrap();

        assert_eq!(sequence.byte_len(), 1);
    }

    #[test]
    fn test_sequence_multiple_frames() {
        let frame1 = FrameBuilder::new(1).unwrap().build().unwrap();
        let frame2 = FrameBuilder::new(2).unwrap().loop_count(3).unwrap().build().unwrap();

        let sequence = SequenceBuilder::new()
            .add_frame(frame1)
            .unwrap()
            .add_frame(frame2)
            .unwrap()
            .build()
            .unwrap();

        // Frame 1: 1 byte, Frame 2: 2 bytes
        assert_eq!(sequence.byte_len(), 3);
    }

    #[test]
    fn test_sequence_empty() {
        assert!(matches!(
            SequenceBuilder::new().build(),
            Err(Error::EmptySequence)
        ));
    }

    #[test]
    fn test_sequence_encode() {
        let frame = FrameBuilder::new(1).unwrap().build().unwrap();
        let sequence = SequenceBuilder::new()
            .add_frame(frame)
            .unwrap()
            .build()
            .unwrap();

        let mut buffer = [0u8; 4];
        let len = sequence.encode_into(&mut buffer);
        assert_eq!(len, 1);
        assert_eq!(buffer[0], 0x01); // Default gain=Full(0), timebase=0, snp_id=1
    }
}
