//! Audio processing utilities for WAV file analysis.
//!
//! This module provides functions for analyzing WAV audio files, including
//! detecting silence periods and calculating audio duration.

/// Detects the duration of leading silence in a WAV audio file.
///
/// This function analyzes the beginning of an audio file to find periods of silence,
/// which is useful for subtitle timing synchronization.
///
/// # Arguments
/// * `path` - Path to the WAV file to analyze
/// * `silence_threshold` - Amplitude threshold below which audio is considered silence
/// * `min_silence_len` - Minimum number of samples required to count as silence
///
/// # Returns
/// * `Ok(f64)` - Duration of leading silence in seconds
/// * `Err` - If the file cannot be read or is not a valid WAV file
pub fn detect_leading_silence(
    path: &str,
    silence_threshold: i16,
    min_silence_len: usize,
) -> anyhow::Result<f64> {
    let mut reader = hound::WavReader::open(path)?;
    let spec = reader.spec();
    let sample_rate = spec.sample_rate as usize;
    let mut silence_samples = 0;
    for sample in reader.samples::<i16>() {
        let s = sample?;
        if s.abs() < silence_threshold {
            silence_samples += 1;
        } else if silence_samples >= min_silence_len {
            break;
        } else {
            silence_samples = 0;
        }
    }
    let silence_seconds = silence_samples as f64 / sample_rate as f64;
    Ok(silence_seconds)
}
use hound::WavReader;

/// Calculates the total duration of a WAV audio file in seconds.
///
/// # Arguments
/// * `path` - Path to the WAV file to analyze
///
/// # Returns
/// * `Ok(f64)` - Duration of the audio file in seconds
/// * `Err` - If the file cannot be read or is not a valid WAV file
pub fn wav_duration_seconds(path: &str) -> anyhow::Result<f64> {
    let reader = WavReader::open(path)?;
    let spec = reader.spec();
    let samples = reader.len();
    let frames = samples as f64 / spec.channels as f64;
    let duration = frames / spec.sample_rate as f64;
    Ok(duration)
}
