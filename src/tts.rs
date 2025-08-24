use std::process::{Command, Stdio};
use std::io::Write;
use tracing::error;

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
