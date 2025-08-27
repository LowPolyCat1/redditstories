//! Command-line argument definitions for the Reddit stories video generator.

use clap::Parser;

/// Command-line arguments for configuring the Reddit stories video generation process.
///
/// This structure defines all the configurable parameters including subreddit selection,
/// file paths, TTS settings, and text processing options.
#[derive(Parser, Debug)]
pub struct Args {
    /// The subreddit to fetch stories from (without the 'r/' prefix)
    #[clap(long, default_value = "AITAH")]
    pub subreddit: String,

    /// Path to the background video file
    #[clap(long, default_value = "./res/bg.mp4")]
    pub background: String,

    /// Output path for the generated video file
    #[clap(long, default_value = "out.mp4")]
    pub out: String,

    /// Path to the Piper TTS model file (.onnx format)
    #[clap(long, default_value = "./tts/en_US-hfc_male-medium.onnx")]
    pub piper_model: String,

    /// Maximum number of posts to try before giving up
    #[clap(long, default_value_t = usize::MAX)]
    pub try_posts: usize,

    /// Maximum characters per TTS chunk for processing
    #[clap(long, default_value_t = 250)]
    pub chunk_chars: usize,

    /// Minimum character count required for a story to be considered
    #[clap(long, default_value_t = 1000)]
    pub min_chars: usize,
}
