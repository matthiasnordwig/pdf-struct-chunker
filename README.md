# pdf-struct-chunker

[![CI](https://github.com/YOUR_USERNAME/pdf-struct-chunker/actions/workflows/rust.yml/badge.svg)](https://github.com/YOUR_USERNAME/pdf-struct-chunker/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org)

A robust, structure- and layout-based PDF chunker for RAG (Retrieval-Augmented Generation) pipelines that operates completely **without LLMs**. Instead, it analyzes PDFs based on X/Y coordinates, font sizes (via `pdf_oxide`), and customizable regex profiles.

Ideal for Edge-AI or offline environments where hardware for large LLMs is not available.

> 🌐 **Author: Matthias Nordwig** · [programmiere.de](https://programmiere.de)

## How it Works

```
PDF bytes ──► pdf_oxide (X/Y + font extraction)
                │
                ▼
          Line Classification (regex profiles + font heuristics)
                │
                ▼
          Chunk Assembly (heading splits, backward merge, sentence-aware overflow)
                │
                ▼
          Vec<Chunk> { text, metadata: { section, heading, page } }
```

The chunker processes each PDF page by extracting character-level bounding boxes, reconstructing lines from Y-coordinates, classifying them using configurable regex patterns (or font-size heuristics as fallback), and assembling them into semantically coherent chunks with structural metadata.

## Example Output

```bash
$ pdf-struct-chunker -i legal_document.pdf --format json --pretty
```

```json
[
  {
    "index": 0,
    "char_start": 0,
    "char_end": 441,
    "text": "§ 1 Scope of Application\nThis regulation applies to all companies ...",
    "signature": "§ 1 Scope of Application\nThis regulation applies to all companies",
    "metadata": {
      "section": "§ 1",
      "heading": "Scope of Application",
      "page": 1
    }
  }
]
```

---

## Installation

### From Source
```bash
git clone https://github.com/YOUR_USERNAME/pdf-struct-chunker.git
cd pdf-struct-chunker
cargo build --release
```

### As a Dependency
```toml
[dependencies]
pdf-struct-chunker = { git = "https://github.com/YOUR_USERNAME/pdf-struct-chunker" }
```

---

## CLI Usage

```bash
pdf-struct-chunker [OPTIONS] --input <INPUT>
```

| Flag | Description | Default |
|------|-------------|---------|
| `-i, --input <FILE>` | Path to the input PDF file | **Required** |
| `-p, --profile <FILE>` | Path to a JSON profile configuration | Built-in defaults |
| `-o, --output <FILE>` | Output file path | `stdout` |
| `--format <FORMAT>` | Output format: `jsonl` or `json` | `jsonl` |
| `--pretty` | Pretty-print JSON output | `false` |
| `--stats` | Print statistics instead of chunks | `false` |

### Examples
```bash
# JSONL output to file
pdf-struct-chunker -i document.pdf -o result.jsonl

# Pretty JSON to console
pdf-struct-chunker -i document.pdf --format json --pretty

# Quick statistics
pdf-struct-chunker -i document.pdf --stats
```

---

## Library API

The core function operates entirely in-memory — no file I/O, no temp files:

```rust
use pdf_struct_chunker::{chunk_pdf, Profile};

// Bytes can come from anywhere: file, HTTP upload, S3, etc.
let bytes = std::fs::read("document.pdf").unwrap();

let chunks = chunk_pdf(&bytes, None).unwrap();
for chunk in &chunks {
    println!("[{}] {} (p.{})",
        chunk.metadata.section,
        chunk.metadata.heading,
        chunk.metadata.page.unwrap_or(0),
    );
}
```

---

## Custom Regex Profiles

Control how the chunker identifies structural elements via JSON profiles:

```json
{
  "min_chunk_chars": 200,
  "max_chunk_chars": 1500,
  "patterns": [
    {
      "role": "ignore",
      "regex": "(?:Page|Footer text)",
      "flags": "i",
      "priority": 200
    },
    {
      "role": "heading_l1",
      "regex": "^((?:Chapter|Section)\\s*[\\d]+)\\s*(.*)",
      "flags": "i",
      "priority": 100
    },
    {
      "role": "definition",
      "regex": "\\b(?:means|shall mean|is defined as)",
      "flags": "i",
      "priority": 50
    }
  ]
}
```

### Pattern Roles

| Role | Behavior |
|------|----------|
| `heading_l1` | Forces a new chunk. Capture group 1 → `section`, group 2 → `heading`. |
| `definition` | Flushes current chunk if it has reached `min_chunk_chars`. |
| `ignore` | Removes the matching line entirely (e.g., page numbers, footers). |

### Profile Fields

| Field | Description | Default |
|-------|-------------|---------|
| `min_chunk_chars` | Minimum chunk size before a "soft" split (definitions, list items) is allowed | `200` |
| `max_chunk_chars` | Maximum chunk size before a forced split at a sentence boundary | `1500` |
| `patterns[].regex` | Regular expression to match against each line | — |
| `patterns[].role` | One of: `heading_l1`, `definition`, `ignore` | — |
| `patterns[].flags` | Regex flags: `"i"` = case-insensitive, `"m"` = multiline | `""` |
| `patterns[].priority` | Higher value = evaluated first when multiple patterns match | `0` |

---

## License

MIT © Matthias Nordwig
