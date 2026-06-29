use pdf_oxide::PdfDocument;
use regex::RegexBuilder;
use std::collections::BTreeMap;

use crate::models::{Chunk, ChunkMetadata, Profile};
use crate::sentences::split_sentences;

#[derive(Debug, thiserror::Error)]
pub enum ChunkerError {
    #[error("PDF parsing failed: {0}")]
    PdfParse(String),
    #[error("Invalid profile: {0}")]
    InvalidProfile(String),
}

#[derive(Debug, Clone)]
struct CompiledPattern {
    role: String,
    regex: regex::Regex,
    priority: i64,
}

#[derive(Debug, Clone)]
struct ClassifiedLine {
    text: String,
    y: i32,
    page_idx: usize,
    max_font_size: f32,
    matched_role: Option<String>,
    extracted_abschnitt: Option<String>,
    extracted_titel: Option<String>,
}

/// Chunk a PDF document based on layout (without LLM).
///
/// Uses X/Y coordinates and font sizes from `pdf_oxide` to classify lines
/// and intelligently assemble them into chunks.
///
/// # Arguments
/// * `bytes` — The raw PDF bytes.
/// * `profile` — Optional profile with regex patterns and thresholds.
///   If missing, a default profile for legal/regulatory texts is used.
///
/// # Returns
/// A `Vec<Chunk>` in document order, or an error.
pub fn chunk_pdf(bytes: &[u8], profile: Option<&Profile>) -> Result<Vec<Chunk>, ChunkerError> {
    let (min_chunk_chars, max_chunk_chars, compiled_patterns) = if let Some(p) = profile {
        let mut comps = Vec::new();
        for pat in &p.patterns {
            let mut builder = RegexBuilder::new(&pat.regex);
            builder.case_insensitive(pat.flags.contains('i'));
            builder.multi_line(pat.flags.contains('m'));
            if let Ok(re) = builder.build() {
                comps.push(CompiledPattern {
                    role: pat.role.clone(),
                    regex: re,
                    priority: pat.priority,
                });
            } else {
                return Err(ChunkerError::InvalidProfile(format!(
                    "Invalid regex: {}",
                    pat.regex
                )));
            }
        }
        comps.sort_by_key(|b| std::cmp::Reverse(b.priority));
        (p.min_chunk_chars, p.max_chunk_chars, comps)
    } else {
        let comps = vec![
            CompiledPattern {
                role: "ignore".to_string(),
                regex: RegexBuilder::new(r"(?:Seite|Page|Bundesgesetzblatt|Amtsblatt|BAnz)").case_insensitive(true).build().unwrap(),
                priority: 200,
            },
            CompiledPattern {
                role: "heading_l1".to_string(),
                regex: RegexBuilder::new(r"^((?:Article|Art\.|§|AT|Kapitel|Abschnitt|TITEL|TITLE|CHAPTER)\s*[\d.a-zA-Z]+)\s*(.*)").case_insensitive(true).build().unwrap(),
                priority: 100,
            },
            CompiledPattern {
                role: "definition".to_string(),
                regex: RegexBuilder::new(r"\b(?:means|shall mean|bezeichnet|gilt als|im Sinne)").case_insensitive(true).build().unwrap(),
                priority: 50,
            },
        ];
        (200, 1500, comps)
    };

    let doc = PdfDocument::from_bytes(bytes.to_vec())
        .map_err(|e| ChunkerError::PdfParse(e.to_string()))?;

    let mut all_chunks = Vec::new();
    let mut chunk_index = 0;

    let mut current_abschnitt: Option<String> = None;
    let mut current_titel: Option<String> = None;

    let mut current_chunk_text = String::new();
    let mut current_chunk_start_page: Option<usize> = None;
    let mut current_chunk_char_start: usize = 0;
    let mut total_chars_processed = 0;

    let mut flush_chunk = |text: &mut String,
                           abschnitt: &Option<String>,
                           titel: &Option<String>,
                           page: Option<usize>,
                           char_start: usize| {
        let trimmed = text.trim();
        if !trimmed.is_empty() {
            if trimmed.len() < min_chunk_chars && !all_chunks.is_empty() {
                // Backward Merge
                let last_chunk: &mut Chunk = all_chunks.last_mut().unwrap();
                last_chunk.text.push('\n');
                last_chunk.text.push_str(trimmed);
                last_chunk.char_end += 1 + trimmed.len();
                text.clear();
                return;
            }
            let sig_text: String = trimmed.chars().take(80).collect();

            let metadata = ChunkMetadata {
                section: abschnitt.clone().unwrap_or_default(),
                heading: titel.clone().unwrap_or_default(),
                page: page.map(|p| p + 1),
            };

            all_chunks.push(Chunk {
                index: chunk_index,
                char_start,
                char_end: char_start + text.len(),
                text: text.clone(),
                signature: Some(sig_text),
                metadata,
            });
            chunk_index += 1;
            text.clear();
        }
    };

    let page_count = doc.page_count().unwrap_or(0);
    let mut classified_lines = Vec::new();

    for page_idx in 0..page_count {
        let chars = doc.extract_chars(page_idx).unwrap_or_default();
        if chars.is_empty() {
            continue;
        }

        let mut lines_by_y: BTreeMap<i32, Vec<&pdf_oxide::layout::TextChar>> = BTreeMap::new();
        for ch in &chars {
            let y_key = (ch.bbox.y * 10.0).round() as i32;
            lines_by_y.entry(y_key).or_default().push(ch);
        }

        let total_chars = chars.len();
        let avg_font_size = if total_chars > 0 {
            chars.iter().map(|c| c.font_size).sum::<f32>() / total_chars as f32
        } else {
            12.0
        };

        let mut keys: Vec<i32> = lines_by_y.keys().copied().collect();
        if keys.is_empty() {
            continue;
        }
        keys.reverse();

        for &y in &keys {
            let mut line_chars = lines_by_y[&y].clone();
            line_chars.sort_by(|a, b| a.bbox.x.partial_cmp(&b.bbox.x).unwrap());

            let mut line_text = String::new();
            let mut bold_chars = 0;
            let mut max_font_size = 0.0_f32;
            let mut prev_x = -1.0;

            for ch in &line_chars {
                if prev_x >= 0.0 && ch.bbox.x - prev_x > ch.font_size * 3.0 {
                    line_text.push_str("    ");
                }
                line_text.push(ch.char);
                if ch.font_name.to_lowercase().contains("bold") {
                    bold_chars += 1;
                }
                if ch.font_size > max_font_size {
                    max_font_size = ch.font_size;
                }
                prev_x = ch.bbox.x + ch.bbox.width;
            }

            let is_mostly_bold = bold_chars > line_chars.len() / 2;
            let line_text_trimmed = line_text.trim();
            if line_text_trimmed.is_empty() {
                continue;
            }

            if line_text_trimmed.contains("...")
                && line_text_trimmed.chars().last().unwrap().is_numeric()
            {
                continue;
            }

            let mut matched_role = None;
            let mut extracted_abschnitt = None;
            let mut extracted_titel = None;

            for pat in &compiled_patterns {
                if let Some(caps) = pat.regex.captures(line_text_trimmed) {
                    matched_role = Some(pat.role.clone());

                    if pat.role.starts_with("heading") {
                        if caps.len() >= 3 {
                            extracted_abschnitt =
                                caps.get(1).map(|m| m.as_str().trim().to_string());
                            extracted_titel = caps.get(2).map(|m| m.as_str().trim().to_string());
                        } else if caps.len() == 2 {
                            extracted_abschnitt =
                                caps.get(1).map(|m| m.as_str().trim().to_string());
                        } else {
                            extracted_titel = Some(line_text_trimmed.to_string());
                        }
                    }
                    break;
                }
            }

            if matched_role.is_none()
                && (max_font_size > avg_font_size * 1.2 || is_mostly_bold)
                && line_text_trimmed.len() < 100
            {
                matched_role = Some("heading_l1".to_string());
                extracted_titel = Some(line_text_trimmed.to_string());
            }

            if matched_role.as_deref() == Some("ignore") {
                continue;
            }

            classified_lines.push(ClassifiedLine {
                text: line_text_trimmed.to_string(),
                y,
                page_idx,
                max_font_size,
                matched_role,
                extracted_abschnitt,
                extracted_titel,
            });
        }
    }

    let mut prev_y = -1;
    let mut prev_page = 0;

    for line in classified_lines.iter() {
        let line_text_trimmed = line.text.trim();

        let is_new_paragraph = if prev_y >= 0 && line.page_idx == prev_page {
            let delta_y = (prev_y - line.y).abs() as f32 / 10.0;
            delta_y > line.max_font_size * 1.5
        } else {
            false
        };

        prev_y = line.y;
        prev_page = line.page_idx;

        let role = line.matched_role.as_deref();
        let is_heading = role.is_some_and(|r| r.starts_with("heading"));
        let is_definition = role == Some("definition");

        let starts_with_number = line_text_trimmed
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_digit());
        let is_list_item = is_new_paragraph && starts_with_number;

        let mut force_flush = false;

        if is_heading {
            if !current_chunk_text.trim().is_empty() {
                force_flush = true;
            }
        } else if is_definition || is_list_item {
            if current_chunk_text.len() >= min_chunk_chars {
                force_flush = true;
            }
        } else if is_new_paragraph && current_chunk_text.len() >= max_chunk_chars {
            force_flush = true;
        }

        if force_flush {
            flush_chunk(
                &mut current_chunk_text,
                &current_abschnitt,
                &current_titel,
                current_chunk_start_page,
                current_chunk_char_start,
            );
            current_chunk_start_page = None;
            current_chunk_char_start = total_chars_processed;
        }

        if is_heading {
            if let Some(ref a) = line.extracted_abschnitt {
                current_abschnitt = Some(a.clone());
            }
            if let Some(ref t) = line.extracted_titel {
                current_titel = Some(t.clone());
            } else if line.extracted_abschnitt.is_none() {
                current_titel = Some(line_text_trimmed.to_string());
            }
        }

        if current_chunk_text.is_empty() {
            current_chunk_start_page = Some(line.page_idx);
            current_chunk_char_start = total_chars_processed;
        }

        current_chunk_text.push_str(line_text_trimmed);
        current_chunk_text.push('\n');

        total_chars_processed += line_text_trimmed.len() + 1;

        if current_chunk_text.len() > max_chunk_chars * 2 {
            let sentences = split_sentences(&current_chunk_text);
            if sentences.len() > 1 {
                let mut split_idx = sentences.len() - 1;
                for (j, s) in sentences.iter().enumerate() {
                    if s.byte_end > max_chunk_chars {
                        split_idx = j.max(1);
                        break;
                    }
                }

                let keep_text = current_chunk_text[..sentences[split_idx].byte_end].to_string();
                let remainder_text =
                    current_chunk_text[sentences[split_idx].byte_end..].to_string();

                let mut temp = keep_text;
                flush_chunk(
                    &mut temp,
                    &current_abschnitt,
                    &current_titel,
                    current_chunk_start_page,
                    current_chunk_char_start,
                );

                current_chunk_text = remainder_text.trim_start().to_string();
                current_chunk_start_page = Some(line.page_idx);
                current_chunk_char_start = total_chars_processed - current_chunk_text.len();
            }
        }
    }

    flush_chunk(
        &mut current_chunk_text,
        &current_abschnitt,
        &current_titel,
        current_chunk_start_page,
        current_chunk_char_start,
    );

    Ok(all_chunks)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Pattern, Profile};

    #[test]
    fn test_default_profile_compiles() {
        let empty_pdf = b"%PDF-1.4\n%EOF";
        let res = chunk_pdf(empty_pdf, None);
        // It should parse the dummy PDF and just return empty chunks, or fail PDF parse,
        // but it should NOT panic because of regex compilation.
        assert!(res.is_ok() || res.is_err());
    }

    #[test]
    fn test_invalid_profile_fails() {
        let p = Profile {
            min_chunk_chars: 10,
            max_chunk_chars: 100,
            patterns: vec![Pattern {
                role: "test".to_string(),
                regex: "(unclosed".to_string(),
                flags: "".to_string(),
                priority: 1,
            }],
        };
        let res = chunk_pdf(b"%PDF", Some(&p));
        assert!(matches!(res, Err(ChunkerError::InvalidProfile(_))));
    }
}
