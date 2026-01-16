//! PWL (Piecewise Linear) point and snippet construction.

use crate::errors::Error;

/// Maximum number of PWL points per snippet.
pub const MAX_POINTS_PER_SNIPPET: usize = 16;

/// A single PWL (Piecewise Linear) point in a waveform snippet.
///
/// Each point is encoded as a single byte:
/// - Bit 7: RMP (0 = step to amplitude, 1 = ramp to amplitude)
/// - Bits 6:4: TIME (number of timebases minus 1, range 0-7 for 1-8 timebases)
/// - Bits 3:0: AMP (amplitude value)
///
/// Amplitude encoding depends on acceleration mode:
/// - With acceleration: 0-15 maps to 0%-100%
/// - Without acceleration: signed 4-bit value (-8 to +7)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PwlPoint {
    byte: u8,
}

impl PwlPoint {
    /// Create a new PWL point with raw byte value.
    pub(crate) fn from_byte(byte: u8) -> Self {
        Self { byte }
    }

    /// Create a new ramp point.
    ///
    /// # Arguments
    /// * `timebases` - Number of timebases (1-8)
    /// * `amplitude` - Amplitude value (interpretation depends on acceleration mode)
    ///
    /// # Errors
    /// Returns `InvalidTimebase` if timebases is not in range 1-8.
    /// Returns `InvalidAmplitude` if amplitude is not in range 0-15.
    pub fn ramp(timebases: u8, amplitude: u8) -> Result<Self, Error> {
        Self::new(true, timebases, amplitude)
    }

    /// Create a new step point.
    ///
    /// # Arguments
    /// * `timebases` - Number of timebases (1-8)
    /// * `amplitude` - Amplitude value (interpretation depends on acceleration mode)
    ///
    /// # Errors
    /// Returns `InvalidTimebase` if timebases is not in range 1-8.
    /// Returns `InvalidAmplitude` if amplitude is not in range 0-15.
    pub fn step(timebases: u8, amplitude: u8) -> Result<Self, Error> {
        Self::new(false, timebases, amplitude)
    }

    fn new(ramp: bool, timebases: u8, amplitude: u8) -> Result<Self, Error> {
        if !(1..=8).contains(&timebases) {
            return Err(Error::InvalidTimebase);
        }
        if amplitude > 15 {
            return Err(Error::InvalidAmplitude);
        }

        let rmp_bit = if ramp { 0x80 } else { 0x00 };
        let time_bits = (timebases - 1) << 4;
        let amp_bits = amplitude & 0x0F;

        Ok(Self {
            byte: rmp_bit | time_bits | amp_bits,
        })
    }

    /// Get the raw byte representation of this point.
    pub fn as_byte(&self) -> u8 {
        self.byte
    }

    /// Check if this is a ramp (true) or step (false) point.
    pub fn is_ramp(&self) -> bool {
        (self.byte & 0x80) != 0
    }

    /// Get the number of timebases (1-8).
    pub fn timebases(&self) -> u8 {
        ((self.byte >> 4) & 0x07) + 1
    }

    /// Get the amplitude value (0-15).
    pub fn amplitude(&self) -> u8 {
        self.byte & 0x0F
    }
}

/// A waveform snippet containing PWL points.
///
/// Snippets are the basic building blocks of waveforms. Each snippet
/// contains 1-16 PWL points that define the waveform shape.
#[derive(Debug, Clone, Copy)]
pub struct Snippet {
    points: [PwlPoint; MAX_POINTS_PER_SNIPPET],
    len: u8,
}

impl Snippet {
    /// Get the points in this snippet.
    pub fn points(&self) -> &[PwlPoint] {
        &self.points[..self.len as usize]
    }

    /// Get the number of bytes this snippet occupies in memory.
    pub fn byte_len(&self) -> usize {
        self.len as usize
    }

    /// Encode the snippet into the provided buffer.
    ///
    /// Returns the number of bytes written.
    pub fn encode_into(&self, buffer: &mut [u8]) -> usize {
        let len = (self.len as usize).min(buffer.len());
        for (i, dest) in buffer.iter_mut().enumerate().take(len) {
            *dest = self.points[i].as_byte();
        }
        self.len as usize
    }
}

/// Builder for constructing snippets.
///
/// # Example
///
/// ```
/// use da728x::waveform::SnippetBuilder;
///
/// let snippet = SnippetBuilder::new()
///     .ramp(1, 15)?  // Ramp to max in 1 timebase
///     .step(2, 15)?  // Hold at max for 2 timebases
///     .ramp(1, 0)?   // Ramp to zero in 1 timebase
///     .build()?;
/// # Ok::<(), da728x::errors::Error>(())
/// ```
#[derive(Debug, Clone, Copy)]
pub struct SnippetBuilder {
    points: [PwlPoint; MAX_POINTS_PER_SNIPPET],
    len: u8,
}

impl Default for SnippetBuilder {
    fn default() -> Self {
        Self {
            points: [PwlPoint::from_byte(0); MAX_POINTS_PER_SNIPPET],
            len: 0,
        }
    }
}

impl SnippetBuilder {
    /// Create a new snippet builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a ramp point to the snippet.
    ///
    /// # Arguments
    /// * `timebases` - Number of timebases (1-8)
    /// * `amplitude` - Amplitude value (0-15)
    ///
    /// # Errors
    /// Returns error if the snippet already has 16 points or parameters are invalid.
    pub fn ramp(mut self, timebases: u8, amplitude: u8) -> Result<Self, Error> {
        let point = PwlPoint::ramp(timebases, amplitude)?;
        if self.len as usize >= MAX_POINTS_PER_SNIPPET {
            return Err(Error::TooManySnippets);
        }
        self.points[self.len as usize] = point;
        self.len += 1;
        Ok(self)
    }

    /// Add a step point to the snippet.
    ///
    /// # Arguments
    /// * `timebases` - Number of timebases (1-8)
    /// * `amplitude` - Amplitude value (0-15)
    ///
    /// # Errors
    /// Returns error if the snippet already has 16 points or parameters are invalid.
    pub fn step(mut self, timebases: u8, amplitude: u8) -> Result<Self, Error> {
        let point = PwlPoint::step(timebases, amplitude)?;
        if self.len as usize >= MAX_POINTS_PER_SNIPPET {
            return Err(Error::TooManySnippets);
        }
        self.points[self.len as usize] = point;
        self.len += 1;
        Ok(self)
    }

    /// Add a raw PWL point to the snippet.
    pub fn point(mut self, point: PwlPoint) -> Result<Self, Error> {
        if self.len as usize >= MAX_POINTS_PER_SNIPPET {
            return Err(Error::TooManySnippets);
        }
        self.points[self.len as usize] = point;
        self.len += 1;
        Ok(self)
    }

    /// Build the snippet.
    ///
    /// # Errors
    /// Returns `EmptySnippet` if no points have been added.
    pub fn build(self) -> Result<Snippet, Error> {
        if self.len == 0 {
            return Err(Error::EmptySnippet);
        }
        Ok(Snippet {
            points: self.points,
            len: self.len,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pwl_point_ramp() {
        let point = PwlPoint::ramp(1, 15).unwrap();
        assert!(point.is_ramp());
        assert_eq!(point.timebases(), 1);
        assert_eq!(point.amplitude(), 15);
        assert_eq!(point.as_byte(), 0x8F); // 1000_1111
    }

    #[test]
    fn test_pwl_point_step() {
        let point = PwlPoint::step(4, 8).unwrap();
        assert!(!point.is_ramp());
        assert_eq!(point.timebases(), 4);
        assert_eq!(point.amplitude(), 8);
        assert_eq!(point.as_byte(), 0x38); // 0011_1000
    }

    #[test]
    fn test_pwl_point_max_timebases() {
        let point = PwlPoint::ramp(8, 0).unwrap();
        assert_eq!(point.timebases(), 8);
        assert_eq!(point.as_byte(), 0xF0); // 1111_0000
    }

    #[test]
    fn test_pwl_point_invalid_timebase() {
        assert!(matches!(PwlPoint::ramp(0, 0), Err(Error::InvalidTimebase)));
        assert!(matches!(PwlPoint::ramp(9, 0), Err(Error::InvalidTimebase)));
    }

    #[test]
    fn test_pwl_point_invalid_amplitude() {
        assert!(matches!(PwlPoint::ramp(1, 16), Err(Error::InvalidAmplitude)));
    }

    #[test]
    fn test_snippet_builder() {
        let snippet = SnippetBuilder::new()
            .ramp(1, 15).unwrap()
            .step(2, 15).unwrap()
            .ramp(1, 0).unwrap()
            .build()
            .unwrap();

        assert_eq!(snippet.byte_len(), 3);
        assert_eq!(snippet.points().len(), 3);
    }

    #[test]
    fn test_snippet_empty() {
        assert!(matches!(
            SnippetBuilder::new().build(),
            Err(Error::EmptySnippet)
        ));
    }

    #[test]
    fn test_snippet_encode() {
        let snippet = SnippetBuilder::new()
            .ramp(1, 15).unwrap()
            .ramp(1, 0).unwrap()
            .build()
            .unwrap();

        let mut buffer = [0u8; 4];
        let len = snippet.encode_into(&mut buffer);
        assert_eq!(len, 2);
        assert_eq!(buffer[0], 0x8F); // ramp, 1 timebase, amp 15
        assert_eq!(buffer[1], 0x80); // ramp, 1 timebase, amp 0
    }
}
