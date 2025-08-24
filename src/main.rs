use clap::Parser;
use hound::WavReader;
use regex::Regex;
use reqwest::header::USER_AGENT;
use serde::Deserialize;
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, error, info, warn};
use tracing_subscriber;

#[derive(Parser, Debug)]
struct Args {
    #[clap(long, default_value = "AITAH")]
    subreddit: String,

    #[clap(long, default_value = "./res/bg2.mp4")]
    background: String,

    #[clap(long, default_value = "final_video.mp4")]
    out: String,

    #[clap(long, default_value = "./en_US-amy-medium.onnx")]
    piper_model: String,

    #[clap(long, default_value_t = 100)]
    try_posts: usize,

    #[clap(long, default_value_t = 250)]
    chunk_chars: usize,
}

#[derive(Debug, Deserialize)]
struct RedditListing {
    data: RedditListingData,
}

#[derive(Debug, Deserialize)]
struct RedditListingData {
    children: Vec<RedditChild>,
}

#[derive(Debug, Deserialize)]
struct RedditChild {
    data: RedditPost,
}

#[derive(Debug, Deserialize)]
struct RedditPost {
    id: String,
    title: String,
    selftext: String,
    is_self: Option<bool>,
    over_18: Option<bool>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("info") // set to "debug" for more logs
        .init();

    info!("Starting reddit story video generation pipeline");

    let args = Args::parse();

    if !Path::new(&args.background).exists() {
        error!("Background video not found: {}", args.background);
        std::process::exit(1);
    }
    info!("Background video found: {}", args.background);

    info!(
        "Fetching reddit story from r/{} (up to {} posts)",
        args.subreddit, args.try_posts
    );
    let story = fetch_reddit_story(&args.subreddit, args.try_posts).await?;
    info!("Using story (short preview): {:.200}", story.replace('\n', " "));

    let chunks = chunk_text(&story, args.chunk_chars);
    info!("Split story into {} chunks", chunks.len());
    debug!(
        "First chunk preview: {}",
        &chunks[0].chars().take(100).collect::<String>()
    );

    let tmp_dir = "rs_tmp";
    if Path::new(tmp_dir).exists() {
        info!("Removing existing tmp dir '{}'", tmp_dir);
        fs::remove_dir_all(tmp_dir)?;
    }
    fs::create_dir_all(tmp_dir)?;
    info!("Created tmp directory '{}'", tmp_dir);

    let mut part_files = Vec::new();
    for (i, chunk) in chunks.iter().enumerate() {
        let fname = format!("{}/part_{:03}.wav", tmp_dir, i);
        info!(
            "Generating TTS chunk {}/{} ({} chars)",
            i + 1,
            chunks.len(),
            chunk.len()
        );
        debug!("Chunk text: {}", chunk);
        match tts_generate_chunk(&args.piper_model, chunk, &fname) {
            Ok(_) => info!("Finished TTS chunk {}: {}", i, fname),
            Err(e) => {
                error!("Failed to generate TTS chunk {}: {:?}", i, e);
                return Err(e);
            }
        }
        part_files.push(fname);
        sleep(Duration::from_millis(150)).await;
    }

    info!("Calculating WAV durations and building subtitles");
    let mut srt_entries = Vec::new();
    let mut cumulative_seconds = 0.0_f64;

    for (i, part) in part_files.iter().enumerate() {
        let dur = wav_duration_seconds(part)?;
        info!("Chunk {} duration: {:.2} seconds", i, dur);
        let start = cumulative_seconds;
        let end = cumulative_seconds + dur;
        let chunk_text = &chunks[i];

        // Power law smoothing factor alpha < 1 for more time on short words
        let alpha = 0.5;

        // Split chunk text into words
        let words: Vec<&str> = chunk_text.split_whitespace().collect();
        if words.is_empty() {
            // no words, fallback to whole chunk subtitle
            srt_entries.push((start, end, chunk_text.clone()));
            cumulative_seconds = end;
            continue;
        }

        // Calculate total weight using powf(alpha)
        let total_weight: f64 = words
            .iter()
            .map(|w| (w.chars().count() as f64).powf(alpha))
            .sum();

        let mut word_start = start;
        for word in words {
            let word_chars = word.chars().count();
            let word_weight = (word_chars as f64).powf(alpha);
            let word_duration = dur * word_weight / total_weight;
            let word_end = word_start + word_duration;

            srt_entries.push((word_start, word_end, word.to_string()));
            word_start = word_end;
        }

        cumulative_seconds = end;
    }

    let srt_path = format!("{}/subs.srt", tmp_dir);
    info!("Writing subtitles to {}", srt_path);
    write_srt(&srt_path, &srt_entries)?;

    let concat_list = format!("{}/files.txt", tmp_dir);
    {
        let mut f = File::create(&concat_list)?;
        for p in &part_files {
            let fname = Path::new(p)
                .file_name()
                .and_then(|n| n.to_str())
                .ok_or_else(|| anyhow::anyhow!("Invalid filename"))?;
            writeln!(f, "file '{}'", fname)?;
        }
    }
    info!("Created concat list file {}", concat_list);

    let combined_path = format!("{}/combined.wav", tmp_dir);
    info!("Concatenating WAV chunks into one file {}", combined_path);

    let status = Command::new("ffmpeg")
        .current_dir(tmp_dir)
        .args([
            "-y", "-f", "concat", "-safe", "0", "-i", "files.txt", "-c", "copy", "combined.wav",
        ])
        .status()?;

    if !status.success() {
        warn!("ffmpeg concat with copy failed; retrying with re-encode");
        let status2 = Command::new("ffmpeg")
            .current_dir(tmp_dir)
            .args([
                "-y", "-f", "concat", "-safe", "0", "-i", "files.txt", "-c:a", "pcm_s16le",
                "combined.wav",
            ])
            .status()?;
        if !status2.success() {
            error!("ffmpeg failed to concatenate WAV files");
            anyhow::bail!("ffmpeg failed to concatenate WAV files");
        }
    }
    info!("Combined audio written to {}", combined_path);

    info!("Merging audio and subtitles into final video {}", &args.out);
    let ff_args = [
        "-y",
        "-i",
        &args.background,
        "-i",
        &combined_path,
        "-vf",
        &format!(
            "scale=1080:1920,subtitles={}:force_style='Fontsize=28,OutlineColour=&H000000&,Outline=3,Shadow=0'",
            srt_path
        ),
        "-map",
        "0:v:0",
        "-map",
        "1:a:0",
        "-c:v",
        "libx264",
        "-c:a",
        "aac",
        "-r",
        "60",
        "-shortest",
        &args.out,
    ];
    let status = Command::new("ffmpeg").args(&ff_args).status()?;
    if !status.success() {
        error!("ffmpeg failed to produce final video");
        anyhow::bail!("ffmpeg failed to produce final video");
    }
    info!("Final video written to {}", &args.out);

    // Optional cleanup (comment out to keep temp files)
    // fs::remove_dir_all(tmp_dir)?;

    info!("Process complete.");
    Ok(())
}

async fn fetch_reddit_story(subreddit: &str, limit: usize) -> anyhow::Result<String> {
    let url = format!("https://www.reddit.com/r/{}/hot.json?limit={}", subreddit, limit);
    let client = reqwest::Client::new();
    let res = client
        .get(&url)
        .header(USER_AGENT, "reddit-story-bot-rust/0.1")
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;

    let parsed: RedditListing = serde_json::from_str(&res)?;

    let used_path = "used_posts.json";
    let mut used_ids = load_used_ids(used_path)?;

    for child in parsed.data.children {
        let post = child.data;
        let is_self = post.is_self.unwrap_or(true);
        let nsfw = post.over_18.unwrap_or(false);

        if nsfw || used_ids.contains(&post.id) {
            debug!("Skipping post (NSFW or already used): {}", post.title);
            continue;
        }

        let text = if is_self && !post.selftext.trim().is_empty() {
            format!("{}.\n\n{}", post.title.trim(), post.selftext.trim())
        } else {
            post.title.trim().to_string()
        };

        if !text.trim().is_empty() {
            info!("Selected post: {}", post.title);

            // mark as used
            used_ids.insert(post.id.clone());
            save_used_ids(used_path, &used_ids)?;

            return Ok(text);
        }
    }

    anyhow::bail!("No suitable posts found in subreddit {}", subreddit);
}

fn load_used_ids(path: &str) -> anyhow::Result<HashSet<String>> {
    if !Path::new(path).exists() {
        return Ok(HashSet::new());
    }
    let data = fs::read_to_string(path)?;
    let ids: Vec<String> = serde_json::from_str(&data)?;
    Ok(ids.into_iter().collect())
}

fn save_used_ids(path: &str, ids: &HashSet<String>) -> anyhow::Result<()> {
    let data = serde_json::to_string_pretty(&ids)?;
    fs::write(path, data)?;
    Ok(())
}

fn chunk_text(text: &str, max_chars: usize) -> Vec<String> {
    info!("Splitting text into chunks with max {} chars", max_chars);
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
    info!("Created {} text chunks", chunks.len());
    chunks
}

fn tts_generate_chunk(model: &str, text: &str, out_path: &str) -> anyhow::Result<()> {
    info!("Calling Piper TTS for output file {}", out_path);

    let mut child = Command::new("piper")
        .args(["--model", model, "--output_file", out_path])
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit())
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

    info!("Piper TTS chunk generated successfully: {}", out_path);
    Ok(())
}

fn wav_duration_seconds(path: &str) -> anyhow::Result<f64> {
    let reader = WavReader::open(path)?;
    let spec = reader.spec();
    let samples = reader.len();
    let frames = samples as f64 / spec.channels as f64;
    let duration = frames / spec.sample_rate as f64;
    Ok(duration)
}

fn write_srt(path: &str, entries: &Vec<(f64, f64, String)>) -> anyhow::Result<()> {
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
