//! Frame construction for waveform sequences.

use crate::errors::Error;

/// Gain multiplier for frame playback.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum Gain {
    /// 0.25x gain
    Quarter = 0,
    /// 0.5x gain
    Half = 1,
    /// 0.75x gain
    ThreeQuarter = 2,
    /// 1.0x gain (default)
    #[default]
    Full = 3,
}

/// Timebase for PWL point duration.
///
/// Each PWL point's TIME field is multiplied by this timebase
/// to determine the actual duration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum Timebase {
    /// 5.44ms timebase
    #[default]
    Ms5_44 = 0,
    /// 10.88ms timebase
    Ms10_88 = 1,
    /// 21.76ms timebase
    Ms21_76 = 2,
    /// 43.52ms timebase
    Ms43_52 = 3,
}

/// Maximum frame size in bytes.
const MAX_FRAME_BYTES: usize = 3;

/// A frame in a waveform sequence.
///
/// Frames are encoded as 1-3 bytes:
/// - Byte 1: `0 | GAIN[6:5] | TIMEBASE[4:3] | SNP_ID_L[2:0]`
/// - Byte 2 (optional): `1 | LOOP[6:3] | FREQ_CMD[2] | FREQ[8] | SNP_ID_H[0]`
/// - Byte 3 (optional): `FREQ[7:0]`
#[derive(Debug, Clone, Copy)]
pub struct Frame {
    bytes: [u8; MAX_FRAME_BYTES],
    len: u8,
}

impl Frame {
    /// Get the number of bytes this frame occupies.
    pub fn byte_len(&self) -> usize {
        self.len as usize
    }

    /// Get the raw bytes of this frame.
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes[..self.len as usize]
    }

    /// Encode the frame into the provided buffer.
    ///
    /// Returns the number of bytes written.
    pub fn encode_into(&self, buffer: &mut [u8]) -> usize {
        let len = (self.len as usize).min(buffer.len());
        buffer[..len].copy_from_slice(&self.bytes[..len]);
        len
    }
}

/// Builder for constructing frames.
///
/// # Example
///
/// ```
/// use da728x::waveform::{FrameBuilder, Gain, Timebase};
///
/// // Simple frame with just snippet ID
/// let frame = FrameBuilder::new(1)?
///     .build()?;
///
/// // Frame with all options
/// let frame = FrameBuilder::new(1)?
///     .gain(Gain::Full)
///     .timebase(Timebase::Ms10_88)
///     .loop_count(3)?
///     .build()?;
/// # Ok::<(), da728x::errors::Error>(())
/// ```
#[derive(Debug, Clone)]
pub struct FrameBuilder {
    snippet_id: u8,
    gain: Gain,
    timebase: Timebase,
    loop_count: Option<u8>,
    frequency: Option<u16>,
}

impl FrameBuilder {
    /// Create a new frame builder for the given snippet ID.
    ///
    /// # Arguments
    /// * `snippet_id` - Snippet ID (1-15)
    ///
    /// # Errors
    /// Returns `InvalidSnippetId` if snippet_id is 0 or > 15.
    pub fn new(snippet_id: u8) -> Result<Self, Error> {
        if snippet_id == 0 || snippet_id > 15 {
            return Err(Error::InvalidSnippetId);
        }
        Ok(Self {
            snippet_id,
            gain: Gain::default(),
            timebase: Timebase::default(),
            loop_count: None,
            frequency: None,
        })
    }

    /// Set the gain multiplier.
    pub fn gain(mut self, gain: Gain) -> Self {
        self.gain = gain;
        self
    }

    /// Set the timebase.
    pub fn timebase(mut self, timebase: Timebase) -> Self {
        self.timebase = timebase;
        self
    }

    /// Set the loop count.
    ///
    /// # Arguments
    /// * `count` - Number of times to loop (0-15, where 0 means play once)
    ///
    /// # Errors
    /// Returns `InvalidLoopCount` if count > 15.
    pub fn loop_count(mut self, count: u8) -> Result<Self, Error> {
        if count > 15 {
            return Err(Error::InvalidLoopCount);
        }
        self.loop_count = Some(count);
        Ok(self)
    }

    /// Set a frequency override in Hz.
    ///
    /// # Arguments
    /// * `freq_hz` - Frequency in Hz (must fit in 9 bits, max 511)
    ///
    /// # Errors
    /// Returns `InvalidFrequency` if freq_hz > 511.
    pub fn frequency_hz(mut self, freq_hz: u16) -> Result<Self, Error> {
        if freq_hz > 511 {
            return Err(Error::InvalidFrequency);
        }
        self.frequency = Some(freq_hz);
        Ok(self)
    }

    /// Build the frame.
    pub fn build(self) -> Result<Frame, Error> {
        let mut bytes = [0u8; MAX_FRAME_BYTES];
        let mut len = 0u8;

        // Byte 1: always present
        // Bit 7: 0 (first byte marker)
        // Bits 6:5: GAIN
        // Bits 4:3: TIMEBASE
        // Bits 2:0: SNP_ID lower 3 bits
        let byte1 = ((self.gain as u8) << 5)
            | ((self.timebase as u8) << 3)
            | (self.snippet_id & 0x07);
        bytes[0] = byte1;
        len = 1;

        // Determine if we need byte 2
        let snp_id_high = (self.snippet_id >> 3) & 0x01;
        let need_byte2 = snp_id_high != 0
            || self.loop_count.is_some()
            || self.frequency.is_some();

        if need_byte2 {
            // Byte 2: continuation
            // Bit 7: 1 (continuation marker)
            // Bits 6:3: LOOP count
            // Bit 2: FREQ_CMD
            // Bit 1: FREQ[8]
            // Bit 0: SNP_ID[3]
            let loop_val = self.loop_count.unwrap_or(0);
            let freq_cmd = if self.frequency.is_some() { 1 } else { 0 };
            let freq_high_bit = self.frequency.map(|f| ((f >> 8) & 0x01) as u8).unwrap_or(0);

            let byte2 = 0x80
                | (loop_val << 3)
                | (freq_cmd << 2)
                | (freq_high_bit << 1)
                | snp_id_high;
            bytes[1] = byte2;
            len = 2;

            // Byte 3: only if frequency override
            if let Some(freq) = self.frequency {
                let byte3 = (freq & 0xFF) as u8;
                bytes[2] = byte3;
                len = 3;
            }
        }

        Ok(Frame { bytes, len })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_single_byte() {
        // Snippet ID 1, default gain and timebase
        let frame = FrameBuilder::new(1).unwrap().build().unwrap();
        assert_eq!(frame.byte_len(), 1);
        // Gain=3 (Full), Timebase=0, SNP_ID=1
        // 0_11_00_001 = 0x61
        assert_eq!(frame.as_bytes()[0], 0x61);
    }

    #[test]
    fn test_frame_snippet_id_high() {
        // Snippet ID 8 (needs high bit)
        let frame = FrameBuilder::new(8).unwrap().build().unwrap();
        assert_eq!(frame.byte_len(), 2);
        // Byte 1: Gain=3, Timebase=0, SNP_ID_L=0
        // 0_11_00_000 = 0x60
        assert_eq!(frame.as_bytes()[0], 0x60);
        // Byte 2: 1_0000_0_0_1 = 0x81
        assert_eq!(frame.as_bytes()[1], 0x81);
    }

    #[test]
    fn test_frame_with_loop() {
        let frame = FrameBuilder::new(1)
            .unwrap()
            .loop_count(5)
            .unwrap()
            .build()
            .unwrap();
        assert_eq!(frame.byte_len(), 2);
        // Byte 2: 1_0101_0_0_0 = 0xA8
        assert_eq!(frame.as_bytes()[1], 0xA8);
    }

    #[test]
    fn test_frame_with_frequency() {
        let frame = FrameBuilder::new(1)
            .unwrap()
            .frequency_hz(300)
            .unwrap()
            .build()
            .unwrap();
        assert_eq!(frame.byte_len(), 3);
        // 300 = 0x12C = 0b1_0010_1100
        // Byte 2: 1_0000_1_1_0 = 0x86
        assert_eq!(frame.as_bytes()[1], 0x86);
        // Byte 3: 0x2C
        assert_eq!(frame.as_bytes()[2], 0x2C);
    }

    #[test]
    fn test_frame_invalid_snippet_id() {
        assert!(matches!(FrameBuilder::new(0), Err(Error::InvalidSnippetId)));
        assert!(matches!(FrameBuilder::new(16), Err(Error::InvalidSnippetId)));
    }

    #[test]
    fn test_frame_invalid_loop() {
        assert!(matches!(
            FrameBuilder::new(1).unwrap().loop_count(16),
            Err(Error::InvalidLoopCount)
        ));
    }

    #[test]
    fn test_frame_invalid_frequency() {
        assert!(matches!(
            FrameBuilder::new(1).unwrap().frequency_hz(512),
            Err(Error::InvalidFrequency)
        ));
    }

    #[test]
    fn test_frame_all_options() {
        let frame = FrameBuilder::new(15)
            .unwrap()
            .gain(Gain::Half)
            .timebase(Timebase::Ms43_52)
            .loop_count(10)
            .unwrap()
            .frequency_hz(256)
            .unwrap()
            .build()
            .unwrap();

        assert_eq!(frame.byte_len(), 3);
        // Byte 1: 0_01_11_111 = 0x3F
        assert_eq!(frame.as_bytes()[0], 0x3F);
        // Byte 2: 1_1010_1_1_1 = 0xD7
        // Loop=10, FREQ_CMD=1, FREQ[8]=1, SNP_ID_H=1
        assert_eq!(frame.as_bytes()[1], 0xD7);
        // Byte 3: 256 & 0xFF = 0x00
        assert_eq!(frame.as_bytes()[2], 0x00);
    }
}
