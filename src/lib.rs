//! # pdf-struct-chunker
//!
//! LLM-free, layout-aware PDF chunking for RAG pipelines.
//!
//! This crate analyzes PDF documents using X/Y coordinates and font sizes
//! (via [`pdf_oxide`]) to detect structural elements like headings, sections,
//! and paragraph breaks. It then assembles semantically coherent text chunks
//! suitable for embedding in vector databases.
//!
//! ## Key Features
//! - **Zero LLM dependency**: Pure heuristic-based chunking using layout analysis
//! - **Customizable regex profiles**: Define your own patterns for headings, definitions, and ignored lines
//! - **In-memory API**: `chunk_pdf(&[u8], Option<&Profile>) -> Result<Vec<Chunk>>`
//! - **CLI tool**: Process PDFs from the command line with JSON/JSONL output
//!
//! ## Quick Start (Library)
//! ```rust,no_run
//! use pdf_struct_chunker::{chunk_pdf, Profile};
//!
//! let bytes = std::fs::read("document.pdf").unwrap();
//! let chunks = chunk_pdf(&bytes, None).unwrap();
//! for chunk in &chunks {
//!     println!("{}: {}", chunk.index, &chunk.text[..80.min(chunk.text.len())]);
//! }
//! ```

pub mod chunking;
pub mod models;
pub mod sentences;

pub use chunking::{ChunkerError, chunk_pdf};
pub use models::{Chunk, ChunkMetadata, Pattern, Profile};
pub use sentences::{Sentence, split_sentences};
