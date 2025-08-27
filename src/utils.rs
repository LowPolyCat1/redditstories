//! Utility functions for text processing and content filtering.
//!
//! This module provides various text processing utilities including text chunking,
//! content sanitization, grammar correction, and forbidden word filtering.

use regex::Regex;
use reqwest::Client;
use serde_json::Value;
use std::fs::File;
use std::io::{BufRead, BufReader};
use tracing::warn;

/// Splits text into chunks based on sentence boundaries and character limits.
///
/// This function intelligently breaks text at sentence endings while respecting
/// the maximum character limit per chunk. This is essential for TTS processing
/// to maintain natural speech patterns.
///
/// # Arguments
/// * `text` - The input text to be chunked
/// * `max_chars` - Maximum characters allowed per chunk
///
/// # Returns
/// * `Vec<String>` - Vector of text chunks, each within the character limit
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

/// Loads a list of forbidden words from a text file for content filtering.
///
/// # Arguments
/// * `path` - Path to the text file containing forbidden words (one per line)
///
/// # Returns
/// * `Vec<String>` - Vector of lowercase forbidden words
///
/// # Panics
/// * If the forbidden words file cannot be found or opened
pub fn load_forbidden_words(path: &str) -> Vec<String> {
    let file = File::open(path).expect("forbidden_words.txt not found");
    BufReader::new(file)
        .lines()
        .map_while(Result::ok)
        .map(|w| w.trim().to_lowercase())
        .filter(|w| !w.is_empty())
        .collect()
}

/// Sanitizes Reddit post content by removing URLs, checking for forbidden words,
/// and enforcing word count limits.
///
/// # Arguments
/// * `text` - The raw post text to sanitize
/// * `forbidden` - List of forbidden words to check against
/// * `max_words` - Maximum allowed word count
///
/// # Returns
/// * `Some(String)` - Sanitized text if it passes all filters
/// * `None` - If the text contains forbidden content or exceeds limits
pub fn sanitize_post(text: &str, forbidden: &[String], max_words: usize) -> Option<String> {
    let text = Regex::new(r"https?://\S+").unwrap().replace_all(text, "");

    let lower = text.to_lowercase();
    if forbidden.iter().any(|w| lower.contains(w)) {
        return None;
    }

    let words: Vec<&str> = text.split_whitespace().collect();
    if words.len() > max_words {
        return None;
    }

    let clean = text.chars().filter(|c| c.is_ascii()).collect::<String>();
    Some(clean.trim().to_string())
}

/// Corrects grammar in text using the LanguageTool API.
///
/// This function sends text to the LanguageTool service for grammar checking
/// and applies suggested corrections to improve text quality for TTS.
///
/// # Arguments
/// * `text` - The text to check and correct
///
/// # Returns
/// * `Some(String)` - Corrected text if the API call succeeds
/// * `None` - If the API is unavailable or returns an error
pub async fn correct_grammar(text: &str) -> Option<String> {
    let client = Client::new();
    let params = [("language", "en-US"), ("text", text)];
    let resp: reqwest::Response = client
        .post("https://api.languagetoolplus.com/v2/check")
        .form(&params)
        .send()
        .await
        .ok()?;
    let resp: Value = resp.json().await.ok()?;

    let mut corrected = text.to_string();
    if let Some(matches) = resp.get("matches").and_then(|m| m.as_array()) {
        for m in matches.iter().rev() {
            if let (Some(offset), Some(length), Some(replacements)) = (
                m.get("offset").and_then(|o| o.as_u64()),
                m.get("length").and_then(|l| l.as_u64()),
                m.get("replacements").and_then(|r| r.as_array()),
            ) {
                if let Some(replacement) = replacements
                    .first()
                    .and_then(|r| r.get("value"))
                    .and_then(|v| v.as_str())
                {
                    let offset = offset as usize;
                    let length = length as usize;
                    if offset + length <= corrected.len() {
                        corrected.replace_range(offset..offset + length, replacement);
                    }
                }
            }
        }
    }
    Some(corrected)
}
