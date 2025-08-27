
//! Reddit Stories Video Generator
//!
//! This application fetches Reddit stories from specified subreddits and converts them
//! into video content with text-to-speech narration and subtitles overlaid on a background video.

mod args;
use clap::Parser;
mod reddit;
mod tts;
mod audio;
mod subtitle;
mod utils;

use crate::args::Args;
use crate::reddit::fetch_reddit_story;
use crate::tts::tts_generate_chunk;
use crate::subtitle::write_srt;
use crate::utils::chunk_text;
use tracing::{debug, error, info, warn};
use std::fs;
use std::path::Path;
use std::fs::File;
use std::io::Write;
use std::process::Command;

/// Main entry point for the Reddit stories video generator.
///
/// This function orchestrates the entire pipeline:
/// 1. Fetches a suitable Reddit story from the specified subreddit
/// 2. Applies grammar correction to the story text
/// 3. Splits the text into manageable chunks for TTS processing
/// 4. Generates audio files using Piper TTS for each chunk
/// 5. Creates subtitle files with proper timing
/// 6. Combines audio chunks and merges with background video
/// 7. Outputs the final video with embedded subtitles
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    info!("Starting reddit story video generation pipeline");
    let args = Args::parse();

    if !Path::new(&args.background).exists() {
        error!("Background video not found: {}", args.background);
        std::process::exit(1);
    }
    info!("Background video found: {}", args.background);

    info!("Fetching reddit story from r/{} (up to {} posts, min {} chars)", args.subreddit, args.try_posts, args.min_chars);
    let story = fetch_reddit_story(&args.subreddit, args.try_posts, args.min_chars).await?;
    info!("Using story (short preview): {:.200}", story.replace('\n', " "));

    let story = match crate::utils::correct_grammar(&story).await {
        Some(corrected) => {
            info!("Grammar corrected.");
            corrected
        },
        None => {
            warn!("Grammar correction failed, using original text.");
            story
        }
    };

    let chunks = chunk_text(&story, args.chunk_chars);
    let num_chunks = chunks.len();
    info!("Split story into {} chunks", num_chunks);
    debug!("First chunk preview: {}", &chunks[0].chars().take(100).collect::<String>());

    let tmp_dir = "rs_tmp";
    if Path::new(tmp_dir).exists() {
        info!("Removing existing tmp dir '{}'", tmp_dir);
        fs::remove_dir_all(tmp_dir)?;
    }
    fs::create_dir_all(tmp_dir)?;
    info!("Created tmp directory '{}'", tmp_dir);

    let mut tasks = Vec::new();
    for (i, chunk) in chunks.into_iter().enumerate() {
        let fname = format!("{}/part_{:03}.wav", tmp_dir, i);
        let piper_model = args.piper_model.clone();
        info!("Spawning TTS generation for chunk {}/{} ({} chars)", i + 1, num_chunks, chunk.len());
        let task = tokio::task::spawn(async move {
            match tts_generate_chunk(&piper_model, &chunk, &fname) {
                Ok(_) => {
                    info!("Finished TTS chunk {}: {}", i, fname);
                    Ok((fname, chunk))
                }
                Err(e) => {
                    error!("Failed to generate TTS chunk {}: {:?}", i, e);
                    Err(e)
                }
            }
        });
        tasks.push(task);
    }

    let mut tts_results = Vec::new();
    for task in tasks {
        let (fname, chunk) = task.await??;
        tts_results.push((fname, chunk));
    }

    info!("Calculating WAV durations and building subtitles");
    let srt_entries = subtitle::build_srt_entries(&tts_results)?;

    let srt_path = format!("{}/subs.srt", tmp_dir);
    info!("Writing subtitles to {}", srt_path);
    write_srt(&srt_path, &srt_entries)?;

    let concat_list = format!("{}/files.txt", tmp_dir);
    {
        let mut f = File::create(&concat_list)?;
        for p in tts_results.iter().map(|(p, _)| p) {
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
        .args(["-y", "-f", "concat", "-safe", "0", "-i", "files.txt", "-c", "copy", "combined.wav"])
        .status()?;

    if !status.success() {
        warn!("ffmpeg concat with copy failed; retrying with re-encode");
        let status2 = Command::new("ffmpeg")
            .current_dir(tmp_dir)
            .args(["-y", "-f", "concat", "-safe", "0", "-i", "files.txt", "-c:a", "pcm_s16le", "combined.wav"])
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
            "scale=1080:1920,subtitles={}:force_style='Fontsize=28,OutlineColour=&H00C4903C&,Outline=3,Shadow=0,Alignment=10'",
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

    fs::remove_dir_all(tmp_dir)?;

    info!("Process complete.");
    Ok(())
}
