use regex::Regex;
use crate::audio::wav_duration_seconds;
use std::fs::File;
use std::io::Write;

pub fn build_srt_entries(tts_results: &Vec<(String, String)>) -> anyhow::Result<Vec<(f64, f64, String)>> {
    let mut srt_entries = Vec::new();
    let mut cumulative_seconds = 0.0_f64;
    for (_i, (part, chunk_text)) in tts_results.iter().enumerate() {
        let dur = crate::audio::wav_duration_seconds(part)?;
        let leading_silence = crate::audio::detect_leading_silence(part, 500, 2000).unwrap_or(0.0); // Schwellenwert und Mindestlänge ggf. anpassen
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
                "," => current_time_in_chunk += COMMA_PAUSE,
                "." | "!" | "?" => current_time_in_chunk += SENTENCE_END_PAUSE,
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
