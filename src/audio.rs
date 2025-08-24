use hound::WavReader;

pub fn wav_duration_seconds(path: &str) -> anyhow::Result<f64> {
    let reader = WavReader::open(path)?;
    let spec = reader.spec();
    let samples = reader.len();
    let frames = samples as f64 / spec.channels as f64;
    let duration = frames / spec.sample_rate as f64;
    Ok(duration)
}
