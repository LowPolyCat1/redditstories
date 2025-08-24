use regex::Regex;
use tracing::warn;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn chunk_text(text: &str, max_chars: usize) -> Vec<String> {
    let re = Regex::new(r"(?s)([^.!?]+[.!?]+)|([^.!?]+$)").unwrap();
    let mut sentences = Vec::new();
    for cap in re.captures_iter(text) {
        let s = cap.get(0).unwrap().as_str().trim();
        if !s.is_empty() {
            sentences.push(s.to_string());
        }
    }
    if sentences.is_empty() {
        warn!("No sentence breaks found; returning whole text as one chunk");
        return vec![text.to_string()];
    }
    let mut chunks = Vec::new();
    let mut current = String::new();
    for s in sentences {
        if current.is_empty() {
            current.push_str(&s);
        } else if current.len() + 1 + s.len() <= max_chars {
            current.push(' ');
            current.push_str(&s);
        } else {
            chunks.push(current);
            current = s;
        }
    }
    if !current.is_empty() {
        chunks.push(current);
    }
    chunks
}

/// Lädt verbotene Wörter aus einer Datei (je Zeile ein Wort)
pub fn load_forbidden_words(path: &str) -> Vec<String> {
    let file = File::open(path).expect("forbidden_words.txt nicht gefunden");
    BufReader::new(file)
        .lines()
        .filter_map(Result::ok)
        .map(|w| w.trim().to_lowercase())
        .filter(|w| !w.is_empty())
        .collect()
}

/// Sanetisiert einen Reddit-Post-Text
pub fn sanitize_post(text: &str, forbidden: &[String], max_words: usize) -> Option<String> {
    // Entferne URLs
    let text = Regex::new(r"https?://\S+").unwrap().replace_all(text, "");

    // Prüfe auf verbotene Wörter
    let lower = text.to_lowercase();
    if forbidden.iter().any(|w| lower.contains(w)) {
        return None;
    }

    // Begrenze die Wortzahl
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.len() > max_words {
        return None;
    }

    // Entferne nicht-ASCII-Zeichen (z.B. Emojis)
    let clean = text.chars().filter(|c| c.is_ascii()).collect::<String>();
    Some(clean.trim().to_string())
}
