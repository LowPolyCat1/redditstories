use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(long, default_value = "AITAH")]
    pub subreddit: String,

    #[clap(long, default_value = "./res/bg2.mp4")]
    pub background: String,

    #[clap(long, default_value = "final_video.mp4")]
    pub out: String,

    #[clap(long, default_value = "./en_US-amy-medium.onnx")]
    pub piper_model: String,

    #[clap(long, default_value_t = 10)]
    pub try_posts: usize,

    #[clap(long, default_value_t = 250)]
    pub chunk_chars: usize,
}
