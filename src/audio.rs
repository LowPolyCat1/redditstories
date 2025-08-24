pub fn detect_leading_silence(path: &str, silence_threshold: i16, min_silence_len: usize) -> anyhow::Result<f64> {
    let mut reader = hound::WavReader::open(path)?;
    let spec = reader.spec();
    let sample_rate = spec.sample_rate as usize;
    let mut silence_samples = 0;
    for sample in reader.samples::<i16>() {
        let s = sample?;
        if s.abs() < silence_threshold {
            silence_samples += 1;
        } else {
            if silence_samples >= min_silence_len {
                break;
            } else {
                silence_samples = 0;
            }
        }
    }
    let silence_seconds = silence_samples as f64 / sample_rate as f64;
    Ok(silence_seconds)
}
use hound::WavReader;

pub fn wav_duration_seconds(path: &str) -> anyhow::Result<f64> {
    let reader = WavReader::open(path)?;
    let spec = reader.spec();
    let samples = reader.len();
    let frames = samples as f64 / spec.channels as f64;
    let duration = frames / spec.sample_rate as f64;
    Ok(duration)
}
