//! Pure-Rust, offline rule-based sentence segmentation with an abbreviation
//! guard. No ML model, no heavy dependencies. Offsets are byte offsets into
//! the original string so callers can slice it directly.

/// Lowercased abbreviations (without the trailing dot) that must NOT end a
/// sentence. Single letters (initials like "A.") are guarded separately.
const ABBREVIATIONS: &[&str] = &[
    "mr", "mrs", "ms", "dr", "prof", "sr", "jr", "st", "vs", "etc", "fig", "no", "vol", "pp",
    "inc", "ltd", "co", "cf", "al", "eq", "sec", "art", "approx", "e.g", "i.e", "jan", "feb",
    "mar", "apr", "jun", "jul", "aug", "sep", "sept", "oct", "nov", "dec",
];

#[derive(Debug, Clone)]
pub struct Sentence {
    pub index: usize,
    pub byte_start: usize,
    pub byte_end: usize,
    pub text: String,
}

#[inline]
fn is_terminator(c: char) -> bool {
    matches!(c, '.' | '!' | '?' | '…')
}

#[inline]
fn is_closing(c: char) -> bool {
    matches!(c, '"' | '\'' | ')' | ']' | '}' | '”' | '’' | '»')
}

/// True when a '.' sits between two digits (e.g. "v1.5", "3.14").
fn is_number_dot(chars: &[(usize, char)], i: usize) -> bool {
    let prev = if i > 0 { chars[i - 1].1 } else { ' ' };
    let next = if i + 1 < chars.len() {
        chars[i + 1].1
    } else {
        ' '
    };
    prev.is_ascii_digit() && next.is_ascii_digit()
}

/// True when the token preceding a '.' is a known abbreviation or a single-letter initial.
fn is_abbreviation(chars: &[(usize, char)], i: usize) -> bool {
    let mut token = String::new();
    let mut k = i;
    while k > 0 {
        k -= 1;
        let c = chars[k].1;
        if c.is_alphanumeric() || c == '.' {
            token.push(c);
        } else {
            break;
        }
    }
    let token: String = token.chars().rev().collect::<String>().to_lowercase();
    let core = token.trim_matches('.');
    if core.is_empty() {
        return false;
    }
    if core.chars().count() == 1 && core.chars().all(|c| c.is_alphabetic()) {
        return true; // initial, e.g. "A."
    }
    ABBREVIATIONS.contains(&core)
}

/// Split `text` into sentences. Best-effort, deterministic, offline.
pub fn split_sentences(text: &str) -> Vec<Sentence> {
    let chars: Vec<(usize, char)> = text.char_indices().collect();
    let n = chars.len();
    let mut sentences = Vec::new();
    let mut sent_start: Option<usize> = None;
    let mut i = 0;

    while i < n {
        let (boff, c) = chars[i];
        if sent_start.is_none() && !c.is_whitespace() {
            sent_start = Some(boff);
        }

        if is_terminator(c) {
            let mut split = true;
            if c == '.' && (is_number_dot(&chars, i) || is_abbreviation(&chars, i)) {
                split = false;
            }
            if split {
                // Absorb consecutive terminators and closing punctuation ("?!", '."').
                let mut j = i;
                while j + 1 < n && (is_terminator(chars[j + 1].1) || is_closing(chars[j + 1].1)) {
                    j += 1;
                }
                let last = chars[j];
                let end_byte = last.0 + last.1.len_utf8();
                let at_boundary = j + 1 >= n || chars[j + 1].1.is_whitespace();
                if at_boundary {
                    if let Some(s) = sent_start {
                        sentences.push(Sentence {
                            index: sentences.len(),
                            byte_start: s,
                            byte_end: end_byte,
                            text: text[s..end_byte].to_string(),
                        });
                    }
                    sent_start = None;
                    i = j + 1;
                    continue;
                }
            }
        }
        i += 1;
    }

    // Trailing text with no terminal punctuation.
    if let Some(s) = sent_start {
        let end = text.trim_end().len();
        if end > s {
            sentences.push(Sentence {
                index: sentences.len(),
                byte_start: s,
                byte_end: end,
                text: text[s..end].to_string(),
            });
        }
    }
    sentences
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_abbreviation_guard() {
        let sentences = split_sentences("Dr. Müller ging nach Hause.");
        assert_eq!(sentences.len(), 1);
        assert_eq!(sentences[0].text, "Dr. Müller ging nach Hause.");
    }

    #[test]
    fn test_number_dot_guard() {
        let sentences = split_sentences("Version 1.5 ist stabil. Das steht fest.");
        assert_eq!(sentences.len(), 2);
        assert_eq!(sentences[0].text, "Version 1.5 ist stabil.");
        assert_eq!(sentences[1].text, "Das steht fest.");
    }

    #[test]
    fn test_multiple_sentences() {
        let sentences = split_sentences("Erster Satz. Zweiter Satz. Dritter Satz.");
        assert_eq!(sentences.len(), 3);
        assert_eq!(sentences[0].text, "Erster Satz.");
        assert_eq!(sentences[1].text, "Zweiter Satz.");
        assert_eq!(sentences[2].text, "Dritter Satz.");
    }

    #[test]
    fn test_trailing_text() {
        let sentences = split_sentences("Text ohne Satzzeichen am Ende");
        assert_eq!(sentences.len(), 1);
        assert_eq!(sentences[0].text, "Text ohne Satzzeichen am Ende");
    }
}
