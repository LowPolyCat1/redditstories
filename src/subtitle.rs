//! Subtitle generation and formatting utilities.
//!
//! This module handles the creation of SRT subtitle files with precise timing
//! based on TTS audio chunks and text analysis.

use regex::Regex;
use std::fs::File;
use std::io::Write;

/// Builds SRT subtitle entries with precise timing from TTS audio chunks.
///
/// This function analyzes the generated TTS audio files and corresponding text
/// to create properly timed subtitle entries. It accounts for silence periods,
/// word-level timing, and natural pauses at punctuation marks.
///
/// # Arguments
/// * `tts_results` - Vector of tuples containing (audio_file_path, text_content)
///
/// # Returns
/// * `Ok(Vec<(f64, f64, String)>)` - Vector of (start_time, end_time, text) tuples
/// * `Err` - If audio files cannot be analyzed or timing calculation fails
pub fn build_srt_entries(tts_results: &Vec<(String, String)>) -> anyhow::Result<Vec<(f64, f64, String)>> {
    let mut srt_entries = Vec::new();
    let mut cumulative_seconds = 0.0_f64;
    for (_i, (part, chunk_text)) in tts_results.iter().enumerate() {
        let dur = crate::audio::wav_duration_seconds(part)?;
        let leading_silence = crate::audio::detect_leading_silence(part, 500, 2000).unwrap_or(0.0);
        let start_time_of_chunk = cumulative_seconds + leading_silence;
        let end_time_of_chunk = start_time_of_chunk + (dur - leading_silence);
        const COMMA_PAUSE: f64 = 0.2;
        const SENTENCE_END_PAUSE: f64 = 0.4;
        let word_regex = Regex::new(r"(\w[\w'-]*)|([,.!?])").unwrap();
        let elements: Vec<&str> = word_regex.find_iter(chunk_text).map(|m| m.as_str()).collect();
        if elements.is_empty() {
            srt_entries.push((start_time_of_chunk, end_time_of_chunk, chunk_text.clone()));
            cumulative_seconds = end_time_of_chunk;
            continue;
        }
        let mut total_pause_time = 0.0;
        let mut word_elements = Vec::new();
        for &element in &elements {
            match element {
                "," => total_pause_time += COMMA_PAUSE,
                "." | "!" | "?" => total_pause_time += SENTENCE_END_PAUSE,
                _ => word_elements.push(element),
            }
        }
        let word_time_available = (dur - leading_silence - total_pause_time).max(0.0);
        let alpha = 0.75;
        let total_weight: f64 = word_elements.iter().map(|w| (w.chars().count() as f64).powf(alpha)).sum();
        let mut current_time_in_chunk = start_time_of_chunk;
        for element in elements {
            match element {
                "," => {
                    let pause_start = current_time_in_chunk;
                    let pause_end = pause_start + COMMA_PAUSE;
                    srt_entries.push((pause_start, pause_end, String::from(" ")));
                    current_time_in_chunk = pause_end;
                },
                "." | "!" | "?" | ";" => {
                    let pause_start = current_time_in_chunk;
                    let pause_end = pause_start + SENTENCE_END_PAUSE;
                    srt_entries.push((pause_start, pause_end, String::from(" ")));
                    current_time_in_chunk = pause_end;
                },
                word => {
                    let word_weight = (word.chars().count() as f64).powf(alpha);
                    let word_duration = if total_weight > 0.0 {
                        word_time_available * word_weight / total_weight
                    } else { 0.0 };
                    let word_start = current_time_in_chunk;
                    let word_end = word_start + word_duration;
                    srt_entries.push((word_start, word_end, word.to_string()));
                    current_time_in_chunk = word_end;
                }
            }
        }
        cumulative_seconds = end_time_of_chunk;
    }
    Ok(srt_entries)
}

/// Writes subtitle entries to an SRT format file.
///
/// # Arguments
/// * `path` - Output path for the SRT file
/// * `entries` - Vector of (start_time, end_time, text) tuples
///
/// # Returns
/// * `Ok(())` - If the file was successfully written
/// * `Err` - If the file cannot be created or written to
pub fn write_srt(path: &str, entries: &Vec<(f64, f64, String)>) -> anyhow::Result<()> {
    let mut f = File::create(path)?;
    for (i, (start, end, text)) in entries.iter().enumerate() {
        writeln!(f, "{}", i + 1)?;
        writeln!(f, "{} --> {}", format_srt_time(*start), format_srt_time(*end))?;
        for line in wrap_text(text, 80) {
            writeln!(f, "{}", line)?;
        }
        writeln!(f)?;
    }
    Ok(())
}

/// Formats a time value in seconds to SRT timestamp format (HH:MM:SS,mmm).
///
/// # Arguments
/// * `seconds` - Time value in seconds
///
/// # Returns
/// * `String` - Formatted timestamp in SRT format
fn format_srt_time(seconds: f64) -> String {
    let total_ms = (seconds * 1000.0).round() as u64;
    let ms = total_ms % 1000;
    let total_sec = total_ms / 1000;
    let s = total_sec % 60;
    let total_min = total_sec / 60;
    let m = total_min % 60;
    let h = total_min / 60;
    format!("{:02}:{:02}:{:02},{:03}", h, m, s, ms)
}

/// Wraps text to fit within a specified character width for subtitle display.
///
/// # Arguments
/// * `s` - Text to wrap
/// * `width` - Maximum characters per line
///
/// # Returns
/// * `Vec<String>` - Vector of wrapped text lines
fn wrap_text(s: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in s.split_whitespace() {
        if current.len() + word.len() + 1 > width && !current.is_empty() {
            lines.push(current.clone());
            current.clear();
            current.push_str(word);
        } else {
            if !current.is_empty() {
                current.push(' ');
            }
            current.push_str(word);
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}
