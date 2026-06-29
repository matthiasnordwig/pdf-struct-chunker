use pdf_struct_chunker::{Profile, chunk_pdf};
use std::fs;

#[test]
fn chunking_produces_nonempty_chunks() {
    let bytes = fs::read("fixtures/sample.pdf").expect("Failed to read fixture");
    let chunks = chunk_pdf(&bytes, None).expect("Chunking failed");

    assert!(!chunks.is_empty(), "Expected at least one chunk");
    for chunk in chunks {
        assert!(!chunk.text.trim().is_empty(), "Found empty chunk text");
    }
}

#[test]
fn chunks_are_ordered() {
    let bytes = fs::read("fixtures/sample.pdf").unwrap();
    let chunks = chunk_pdf(&bytes, None).unwrap();

    assert!(!chunks.is_empty());
    for (i, chunk) in chunks.iter().enumerate() {
        assert_eq!(chunk.index, i, "Chunk index mismatch");
    }
}

#[test]
fn custom_profile_from_json() {
    let bytes = fs::read("fixtures/sample.pdf").unwrap();
    let profile_str = fs::read_to_string("fixtures/example_profile.json").unwrap();
    let profile: Profile = serde_json::from_str(&profile_str).unwrap();

    let chunks = chunk_pdf(&bytes, Some(&profile)).unwrap();
    assert!(!chunks.is_empty(), "Expected chunks with custom profile");
}

#[test]
fn chunk_metadata_has_page_numbers() {
    let bytes = fs::read("fixtures/sample.pdf").unwrap();
    let chunks = chunk_pdf(&bytes, None).unwrap();

    assert!(!chunks.is_empty());
    assert!(
        chunks[0].metadata.page.is_some(),
        "First chunk should have a page number"
    );
}

#[test]
fn empty_profile_still_works() {
    let bytes = fs::read("fixtures/sample.pdf").unwrap();
    let profile = Profile {
        min_chunk_chars: 50,
        max_chunk_chars: 500,
        patterns: vec![],
    };

    let chunks = chunk_pdf(&bytes, Some(&profile)).unwrap();
    assert!(
        !chunks.is_empty(),
        "Empty pattern list should fallback to font heuristics"
    );
}
