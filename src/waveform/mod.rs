//! Waveform memory construction for DA728x haptic drivers.
//!
//! This module provides type-safe builders for constructing waveform memory
//! data that can be uploaded to the DA728x's internal memory for RTWM
//! (Register-Triggered Waveform Memory) playback.
//!
//! # Memory Structure
//!
//! The waveform memory layout is:
//! - Byte 0: Number of snippets (1-15)
//! - Byte 1: Number of sequences (1-16)
//! - Bytes 2..: End pointers (1 per snippet + 1 per sequence)
//! - Remaining: Snippet data followed by sequence data
//!
//! # Example
//!
//! ```no_run
//! use da728x::waveform::{SnippetBuilder, FrameBuilder, SequenceBuilder, WaveformMemoryBuilder, Timebase};
//!
//! // Create a simple click snippet
//! let click = SnippetBuilder::new()
//!     .ramp(1, 15)?   // Quick rise to 100%
//!     .ramp(1, 0)?    // Quick fall
//!     .build()?;
//!
//! // Create a frame using the snippet
//! let frame = FrameBuilder::new(1)?
//!     .timebase(Timebase::Ms5_44)
//!     .build()?;
//!
//! // Create a sequence with the frame
//! let seq = SequenceBuilder::new()
//!     .add_frame(frame)?
//!     .build()?;
//!
//! // Build the waveform memory
//! let memory = WaveformMemoryBuilder::new(true)  // acceleration enabled
//!     .add_snippet(click)?
//!     .add_sequence(seq)?
//!     .build()?;
//! # Ok::<(), da728x::errors::Error>(())
//! ```

mod snippet;
mod frame;
mod sequence;
mod memory;

pub use snippet::{PwlPoint, Snippet, SnippetBuilder};
pub use frame::{Frame, FrameBuilder, Gain, Timebase};
pub use sequence::{Sequence, SequenceBuilder};
pub use memory::{WaveformMemory, WaveformMemoryBuilder};
