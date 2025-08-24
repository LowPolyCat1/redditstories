use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(long, default_value = "AITAH")]
    pub subreddit: String,

    #[clap(long, default_value = "./res/bg.mp4")]
    pub background: String,

    #[clap(long, default_value = "out.mp4")]
    pub out: String,

    #[clap(long, default_value = "./tts/en_US-hfc_male-medium.onnx")]
    pub piper_model: String,

    #[clap(long, default_value_t = usize::MAX)]
    pub try_posts: usize,

    #[clap(long, default_value_t = 250)]
    pub chunk_chars: usize,
}
