use serde::{Deserialize, Serialize};

/// A single regex pattern within a profile.
/// `role` determines the behavior: "heading_l1", "definition", "ignore".
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    pub role: String,
    pub regex: String,
    /// Regex flags: "i" = case-insensitive, "m" = multiline.
    #[serde(default)]
    pub flags: String,
    /// Higher priority wins when multiple matches occur on the same line.
    #[serde(default)]
    pub priority: i64,
}

/// Configuration for the layout-based chunking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    #[serde(default = "default_min")]
    pub min_chunk_chars: usize,
    #[serde(default = "default_max")]
    pub max_chunk_chars: usize,
    pub patterns: Vec<Pattern>,
}

fn default_min() -> usize {
    200
}

fn default_max() -> usize {
    1500
}

/// Typed metadata for a chunk.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChunkMetadata {
    /// Extracted section (e.g., "§ 25a", "Article 3").
    #[serde(default)]
    pub section: String,
    /// Extracted heading/title.
    #[serde(default)]
    pub heading: String,
    /// 1-based page number (of the page where the chunk starts).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<usize>,
}

/// A single, extracted chunk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    /// 0-based index within the document.
    pub index: usize,
    /// Byte offset in the reconstructed plain text (start, inclusive).
    pub char_start: usize,
    /// Byte offset in the reconstructed plain text (end, exclusive).
    pub char_end: usize,
    /// The extracted chunk text.
    pub text: String,
    /// The first ~80 characters as a signature (for deduplication).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    /// Structured metadata.
    pub metadata: ChunkMetadata,
}
