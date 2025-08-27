//! Text-to-speech generation using Piper TTS engine.
//!
//! This module provides functionality to convert text chunks into audio files
//! using the Piper TTS system.

use std::process::{Command, Stdio};
use std::io::Write;
use tracing::error;

/// Generates an audio file from text using the Piper TTS engine.
///
/// This function spawns a Piper process to convert the provided text into
/// a WAV audio file using the specified voice model.
///
/// # Arguments
/// * `model` - Path to the Piper TTS model file (.onnx format)
/// * `text` - Text content to convert to speech
/// * `out_path` - Output path for the generated WAV file
///
/// # Returns
/// * `Ok(())` - If the audio file was successfully generated
/// * `Err` - If the Piper process fails or cannot be spawned
pub fn tts_generate_chunk(model: &str, text: &str, out_path: &str) -> anyhow::Result<()> {
    let mut child = Command::new("piper")
        .args(["--model", model, "--output_file", out_path])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("Failed to spawn piper process");

    {
        let stdin = child.stdin.as_mut().expect("Failed to open stdin");
        stdin.write_all(text.as_bytes())?;
    }

    let status = child.wait()?;
    if !status.success() {
        error!("Piper TTS command failed for chunk: {}", out_path);
        anyhow::bail!("TTS engine failed for chunk, command returned non-zero");
    }
    Ok(())
}
