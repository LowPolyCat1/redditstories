# Reddit Stories Video Generator

A Rust application that automatically creates engaging video content from Reddit stories. The tool fetches stories from specified subreddits, converts them to speech using Piper TTS, and combines them with background video and synchronized subtitles.

## Features

- **Automated Story Fetching**: Retrieves stories from any public subreddit
- **Content Filtering**: Filters out NSFW content and posts with forbidden words
- **Grammar Correction**: Automatically improves text quality using LanguageTool API
- **High-Quality TTS**: Uses Piper TTS for natural-sounding narration
- **Smart Subtitles**: Generates precisely timed subtitles with word-level synchronization
- **Duplicate Prevention**: Tracks used posts to avoid repetition
- **Customizable Output**: Configurable video resolution, subtitle styling, and more

## Requirements

### System Dependencies

- **Rust** (latest stable version)
- **Piper TTS** - Download from [Piper releases](https://github.com/rhasspy/piper/releases)
- **FFmpeg** - For video processing and encoding

### Installation

1. Install Rust from [rustup.rs](https://rustup.rs/)
2. Install FFmpeg:
   - Windows: Download from [ffmpeg.org](https://ffmpeg.org/download.html)
   - macOS: `brew install ffmpeg`
   - Linux: `sudo apt install ffmpeg` (Ubuntu/Debian)
3. Download Piper TTS and place the executable in your PATH

## Quick Start

1. **Clone and build the project:**

   ```bash
   git clone <repository-url>
   cd reddit-stories
   cargo build --release
   ```

2. **Set up required directories and files:**

   ```txt
   res/
   ├── bg.mp4          # Background video file
   tts/
   ├── *.onnx          # Piper TTS model files
   config/
   ├── forbidden_words.txt  # List of words to filter (one per line)
   └── used_posts.json      # Automatically managed post history
   ```

3. **Run the application:**

   ```bash
   cargo run --release -- --subreddit AITAH --background "./res/bg.mp4" --out out.mp4 --piper-model "./tts/en_US-hfc_male-medium.onnx"
   ```

## Command Line Options

| Option | Default | Description |
|--------|---------|-------------|
| `--subreddit` | `AITAH` | Subreddit to fetch stories from (without r/ prefix) |
| `--background` | `./res/bg.mp4` | Path to background video file |
| `--out` | `out.mp4` | Output path for generated video |
| `--piper-model` | `./tts/en_US-hfc_male-medium.onnx` | Path to Piper TTS model |
| `--try-posts` | `unlimited` | Maximum posts to try before giving up |
| `--chunk-chars` | `250` | Maximum characters per TTS chunk |
| `--min-chars` | `1000` | Minimum story length to consider |

## Recommended Subreddits

### Story-Based Content

- **AITAH**: `AITAH`, `AmITheAsshole`, `AmItheButtface`
- **Confessions**: `offmychest`, `TrueOffMyChest`, `Confessions`
- **Relationship**: `RelationshipAdvice`, `relationships`, `dating_advice`
- **Drama**: `MaliciousCompliance`, `ProRevenge`, `NuclearRevenge`, `PettyRevenge`

### Workplace & Life

- **Work Stories**: `AntiWork`, `WorkReform`, `Teachers`
- **Service Industry**: `TalesFromYourServer`, `TalesFromTechSupport`
- **Personal**: `TodayIFuckedUp`, `TooAfraidToAsk`

### Entertainment

- **Entitlement**: `EntitledPeople`, `ChoosingBeggars`
- **Questions**: `AskReddit`, `AskMen`, `AskWomen`

## Configuration Files

### Forbidden Words (`config/forbidden_words.txt`)

Create a text file with words to filter out, one per line:

```txt
spam
advertisement
promotion
```

### Used Posts (`config/used_posts.json`)

Automatically managed JSON file tracking processed posts to prevent duplicates.

## TTS Models

Download Piper TTS models from the [official repository](https://github.com/rhasspy/piper/releases). Recommended models:

- `en_US-hfc_male-medium.onnx` - Natural male voice
- `en_US-amy-medium.onnx` - Clear female voice
- `en_US-lessac-medium.onnx` - Professional narrator voice

## Video Output

The generated videos feature:

- **Resolution**: 1080x1920 (vertical format for mobile)
- **Frame Rate**: 60 FPS
- **Subtitles**: Embedded with custom styling
- **Audio**: High-quality AAC encoding
- **Video**: H.264 encoding for broad compatibility

## Troubleshooting

### Common Issues

1. **"Piper not found"**: Ensure Piper is installed and in your PATH
2. **"FFmpeg failed"**: Check FFmpeg installation and file permissions
3. **"No suitable posts found"**: Try different subreddits or adjust `--min-chars`
4. **Grammar correction fails**: Network issue with LanguageTool API (continues with original text)

### Performance Tips

- Use `--release` flag for faster processing
- Adjust `--chunk-chars` for different TTS processing speeds
- Use SSD storage for temporary files during processing

## License

This project is open source. Please ensure compliance with Reddit's API terms of service and respect content creators' rights.
