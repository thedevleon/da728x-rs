//! Waveform memory layout and construction.

use crate::errors::Error;
use super::snippet::Snippet;
use super::sequence::Sequence;

/// Maximum waveform memory size in bytes.
pub const MAX_MEMORY_SIZE: usize = 100;

/// Maximum number of snippets (1-15, ID 0 is reserved).
pub const MAX_SNIPPETS: usize = 15;

/// Maximum number of sequences (0-15).
pub const MAX_SEQUENCES: usize = 16;

/// Waveform memory containing snippets and sequences.
///
/// The memory layout is:
/// - Byte 0: Number of snippets (1-15)
/// - Byte 1: Number of sequences (1-16)
/// - Bytes 2..: End pointers (1 per snippet + 1 per sequence)
/// - Remaining: Snippet data followed by sequence data
///
/// End pointers are relative to the start of the data area (after the header
/// and pointer bytes), with each pointer indicating the end position of that
/// snippet or sequence.
#[derive(Debug, Clone, Copy)]
pub struct WaveformMemory {
    data: [u8; MAX_MEMORY_SIZE],
    len: u8,
    num_snippets: u8,
    num_sequences: u8,
}

impl WaveformMemory {
    /// Get the total number of bytes in the waveform memory.
    pub fn len(&self) -> usize {
        self.len as usize
    }

    /// Check if the memory is empty.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Get the raw bytes of the waveform memory.
    pub fn as_bytes(&self) -> &[u8] {
        &self.data[..self.len as usize]
    }

    /// Get the number of snippets.
    pub fn num_snippets(&self) -> u8 {
        self.num_snippets
    }

    /// Get the number of sequences.
    pub fn num_sequences(&self) -> u8 {
        self.num_sequences
    }
}

/// Builder for constructing waveform memory.
///
/// # Memory Layout
///
/// The waveform memory uses the following layout:
/// ```text
/// [0]      num_snippets
/// [1]      num_sequences
/// [2..]    end pointers (num_snippets + num_sequences bytes)
/// [..]     snippet data (concatenated)
/// [..]     sequence data (concatenated)
/// ```
///
/// End pointers are relative offsets from the start of the data area.
///
/// # Example
///
/// ```
/// use da728x::waveform::{WaveformMemoryBuilder, SnippetBuilder, SequenceBuilder, FrameBuilder};
///
/// let snippet = SnippetBuilder::new()
///     .ramp(1, 15)?
///     .ramp(1, 0)?
///     .build()?;
///
/// let frame = FrameBuilder::new(1)?.build()?;
/// let sequence = SequenceBuilder::new()
///     .add_frame(frame)?
///     .build()?;
///
/// let memory = WaveformMemoryBuilder::new(true)  // acceleration enabled
///     .add_snippet(snippet)?   // Returns snippet ID 1
///     .add_sequence(sequence)? // Returns sequence ID 0
///     .build()?;
///
/// assert!(memory.len() <= 100);
/// # Ok::<(), da728x::errors::Error>(())
/// ```
#[derive(Debug, Clone, Copy)]
pub struct WaveformMemoryBuilder {
    snippets: [Option<Snippet>; MAX_SNIPPETS],
    sequences: [Option<Sequence>; MAX_SEQUENCES],
    num_snippets: u8,
    num_sequences: u8,
    #[allow(dead_code)]
    acceleration_enabled: bool,
}

impl WaveformMemoryBuilder {
    /// Create a new waveform memory builder.
    ///
    /// # Arguments
    /// * `acceleration_enabled` - Whether acceleration mode is enabled.
    ///   This affects how amplitude values are interpreted.
    pub fn new(acceleration_enabled: bool) -> Self {
        Self {
            snippets: [None; MAX_SNIPPETS],
            sequences: [None; MAX_SEQUENCES],
            num_snippets: 0,
            num_sequences: 0,
            acceleration_enabled,
        }
    }

    /// Add a snippet to the waveform memory.
    ///
    /// Returns the assigned snippet ID (1-15). Snippet ID 0 is reserved.
    ///
    /// # Errors
    /// Returns `TooManySnippets` if 15 snippets have already been added.
    pub fn add_snippet(mut self, snippet: Snippet) -> Result<Self, Error> {
        if self.num_snippets as usize >= MAX_SNIPPETS {
            return Err(Error::TooManySnippets);
        }
        self.snippets[self.num_snippets as usize] = Some(snippet);
        self.num_snippets += 1;
        Ok(self)
    }

    /// Get the ID that would be assigned to the next snippet.
    pub fn next_snippet_id(&self) -> u8 {
        self.num_snippets + 1
    }

    /// Add a sequence to the waveform memory.
    ///
    /// Returns the assigned sequence ID (0-15).
    ///
    /// # Errors
    /// Returns `TooManySequences` if 16 sequences have already been added.
    pub fn add_sequence(mut self, sequence: Sequence) -> Result<Self, Error> {
        if self.num_sequences as usize >= MAX_SEQUENCES {
            return Err(Error::TooManySequences);
        }
        self.sequences[self.num_sequences as usize] = Some(sequence);
        self.num_sequences += 1;
        Ok(self)
    }

    /// Get the ID that would be assigned to the next sequence.
    pub fn next_sequence_id(&self) -> u8 {
        self.num_sequences
    }

    /// Calculate the total size of the waveform memory.
    fn calculate_size(&self) -> usize {
        let header_size = 2; // num_snippets + num_sequences
        let pointer_size = self.num_snippets as usize + self.num_sequences as usize;
        let snippet_data_size: usize = self.snippets[..self.num_snippets as usize]
            .iter()
            .filter_map(|s| s.as_ref())
            .map(|s| s.byte_len())
            .sum();
        let sequence_data_size: usize = self.sequences[..self.num_sequences as usize]
            .iter()
            .filter_map(|s| s.as_ref())
            .map(|s| s.byte_len())
            .sum();

        header_size + pointer_size + snippet_data_size + sequence_data_size
    }

    /// Build the waveform memory.
    ///
    /// # Errors
    /// Returns `WaveformMemoryFull` if the total size exceeds 100 bytes.
    /// Returns `EmptySnippet` if no snippets have been added.
    /// Returns `EmptySequence` if no sequences have been added.
    pub fn build(self) -> Result<WaveformMemory, Error> {
        if self.num_snippets == 0 {
            return Err(Error::EmptySnippet);
        }
        if self.num_sequences == 0 {
            return Err(Error::EmptySequence);
        }

        let total_size = self.calculate_size();
        if total_size > MAX_MEMORY_SIZE {
            return Err(Error::WaveformMemoryFull);
        }

        let mut data = [0u8; MAX_MEMORY_SIZE];
        let mut pos = 0usize;

        // Byte 0: Number of snippets
        data[pos] = self.num_snippets;
        pos += 1;

        // Byte 1: Number of sequences
        data[pos] = self.num_sequences;
        pos += 1;

        // Calculate end pointers
        // End pointers are ABSOLUTE indices pointing to the LAST byte of each
        // snippet/sequence within the entire memory array.
        let num_pointers = self.num_snippets as usize + self.num_sequences as usize;
        let data_area_start = 2 + num_pointers;

        // Calculate snippet end pointers (absolute index of last byte)
        let mut current_offset = 0usize;
        for i in 0..self.num_snippets as usize {
            if let Some(ref snippet) = self.snippets[i] {
                current_offset += snippet.byte_len();
                // End pointer = data_area_start + bytes_used - 1 (index of last byte)
                let end_ptr = data_area_start + current_offset - 1;
                data[pos] = end_ptr as u8;
                pos += 1;
            }
        }

        // Calculate sequence end pointers (continue from where snippets ended)
        for i in 0..self.num_sequences as usize {
            if let Some(ref sequence) = self.sequences[i] {
                current_offset += sequence.byte_len();
                let end_ptr = data_area_start + current_offset - 1;
                data[pos] = end_ptr as u8;
                pos += 1;
            }
        }

        // Write snippet data
        for i in 0..self.num_snippets as usize {
            if let Some(ref snippet) = self.snippets[i] {
                for point in snippet.points() {
                    data[pos] = point.as_byte();
                    pos += 1;
                }
            }
        }

        // Write sequence data
        for i in 0..self.num_sequences as usize {
            if let Some(ref sequence) = self.sequences[i] {
                for &byte in sequence.as_bytes() {
                    data[pos] = byte;
                    pos += 1;
                }
            }
        }

        debug_assert_eq!(pos, total_size);
        debug_assert_eq!(pos, data_area_start + current_offset);

        Ok(WaveformMemory {
            data,
            len: pos as u8,
            num_snippets: self.num_snippets,
            num_sequences: self.num_sequences,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::waveform::{SnippetBuilder, SequenceBuilder, FrameBuilder};

    #[test]
    fn test_memory_basic() {
        let snippet = SnippetBuilder::new()
            .ramp(1, 15).unwrap()
            .ramp(1, 0).unwrap()
            .build()
            .unwrap();

        let frame = FrameBuilder::new(1).unwrap().build().unwrap();
        let sequence = SequenceBuilder::new()
            .add_frame(frame)
            .unwrap()
            .build()
            .unwrap();

        let memory = WaveformMemoryBuilder::new(true)
            .add_snippet(snippet)
            .unwrap()
            .add_sequence(sequence)
            .unwrap()
            .build()
            .unwrap();

        // Layout:
        // [0] = 1 (num snippets)
        // [1] = 1 (num sequences)
        // [2] = 5 (end pointer for snippet 1: last byte at index 5)
        // [3] = 6 (end pointer for sequence 0: last byte at index 6)
        // [4] = 0x8F (snippet point 1: ramp, 1 timebase, amp 15)
        // [5] = 0x80 (snippet point 2: ramp, 1 timebase, amp 0)
        // [6] = 0x01 (frame: gain=Full(0), timebase=0, snp_id=1)
        assert_eq!(memory.len(), 7);
        assert_eq!(memory.num_snippets(), 1);
        assert_eq!(memory.num_sequences(), 1);

        let bytes = memory.as_bytes();
        assert_eq!(bytes[0], 1); // num_snippets
        assert_eq!(bytes[1], 1); // num_sequences
        assert_eq!(bytes[2], 5); // snippet end pointer (absolute index of last byte)
        assert_eq!(bytes[3], 6); // sequence end pointer (absolute index of last byte)
        assert_eq!(bytes[4], 0x8F); // snippet data
        assert_eq!(bytes[5], 0x80);
        assert_eq!(bytes[6], 0x01); // sequence data (gain=Full(0), timebase=0, snp_id=1)
    }

    #[test]
    fn test_memory_multiple_snippets() {
        let snippet1 = SnippetBuilder::new()
            .ramp(1, 15).unwrap()
            .build()
            .unwrap();

        let snippet2 = SnippetBuilder::new()
            .step(2, 8).unwrap()
            .ramp(1, 0).unwrap()
            .build()
            .unwrap();

        let frame1 = FrameBuilder::new(1).unwrap().build().unwrap();
        let frame2 = FrameBuilder::new(2).unwrap().build().unwrap();

        let sequence = SequenceBuilder::new()
            .add_frame(frame1)
            .unwrap()
            .add_frame(frame2)
            .unwrap()
            .build()
            .unwrap();

        let memory = WaveformMemoryBuilder::new(true)
            .add_snippet(snippet1)
            .unwrap()
            .add_snippet(snippet2)
            .unwrap()
            .add_sequence(sequence)
            .unwrap()
            .build()
            .unwrap();

        assert_eq!(memory.num_snippets(), 2);
        assert_eq!(memory.num_sequences(), 1);

        let bytes = memory.as_bytes();
        // Header
        assert_eq!(bytes[0], 2); // num_snippets
        assert_eq!(bytes[1], 1); // num_sequences
        // End pointers (absolute indices)
        // Data starts at byte 5 (2 header + 3 pointers)
        assert_eq!(bytes[2], 5); // snippet 1 (1 byte at pos 5) ends at index 5
        assert_eq!(bytes[3], 7); // snippet 2 (2 bytes at pos 6-7) ends at index 7
        assert_eq!(bytes[4], 9); // sequence (2 bytes at pos 8-9) ends at index 9
    }

    #[test]
    fn test_memory_too_many_snippets() {
        let mut builder = WaveformMemoryBuilder::new(true);

        for _ in 0..15 {
            let snippet = SnippetBuilder::new()
                .ramp(1, 15).unwrap()
                .build()
                .unwrap();
            builder = builder.add_snippet(snippet).unwrap();
        }

        let snippet = SnippetBuilder::new()
            .ramp(1, 15).unwrap()
            .build()
            .unwrap();

        assert!(matches!(
            builder.add_snippet(snippet),
            Err(Error::TooManySnippets)
        ));
    }

    #[test]
    fn test_memory_no_snippets() {
        let frame = FrameBuilder::new(1).unwrap().build().unwrap();
        let sequence = SequenceBuilder::new()
            .add_frame(frame)
            .unwrap()
            .build()
            .unwrap();

        let result = WaveformMemoryBuilder::new(true)
            .add_sequence(sequence)
            .unwrap()
            .build();

        assert!(matches!(result, Err(Error::EmptySnippet)));
    }

    #[test]
    fn test_memory_no_sequences() {
        let snippet = SnippetBuilder::new()
            .ramp(1, 15).unwrap()
            .build()
            .unwrap();

        let result = WaveformMemoryBuilder::new(true)
            .add_snippet(snippet)
            .unwrap()
            .build();

        assert!(matches!(result, Err(Error::EmptySequence)));
    }

    #[test]
    fn test_next_ids() {
        let builder = WaveformMemoryBuilder::new(true);
        assert_eq!(builder.next_snippet_id(), 1);
        assert_eq!(builder.next_sequence_id(), 0);

        let snippet = SnippetBuilder::new()
            .ramp(1, 15).unwrap()
            .build()
            .unwrap();

        let builder = builder.add_snippet(snippet).unwrap();
        assert_eq!(builder.next_snippet_id(), 2);
        assert_eq!(builder.next_sequence_id(), 0);
    }
}
