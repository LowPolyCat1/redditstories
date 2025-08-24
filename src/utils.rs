use regex::Regex;
use tracing::warn;

pub fn chunk_text(text: &str, max_chars: usize) -> Vec<String> {
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
    chunks
}
