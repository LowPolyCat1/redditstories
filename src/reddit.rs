use serde::Deserialize;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use reqwest::header::USER_AGENT;
use tracing::{debug, info};
use crate::utils::{load_forbidden_words, sanitize_post};

#[derive(Debug, Deserialize)]
pub struct RedditListing {
    pub data: RedditListingData,
}

#[derive(Debug, Deserialize)]
pub struct RedditListingData {
    pub children: Vec<RedditChild>,
}

#[derive(Debug, Deserialize)]
pub struct RedditChild {
    pub data: RedditPost,
}

#[derive(Debug, Deserialize)]
pub struct RedditPost {
    pub id: String,
    pub title: String,
    pub selftext: String,
    pub is_self: Option<bool>,
    pub over_18: Option<bool>,
}

pub async fn fetch_reddit_story(subreddit: &str, limit: usize) -> anyhow::Result<String> {
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

    let used_path = "./config/used_posts.json";
    let mut used_ids = load_used_ids(used_path)?;

    // Forbidden words laden
    let forbidden_path = "./config/forbidden_words.txt";
    let forbidden = load_forbidden_words(forbidden_path);
    let max_words = 300;

    for child in parsed.data.children {
        let post = child.data;
        let is_self = post.is_self.unwrap_or(true);
        let nsfw = post.over_18.unwrap_or(false);

        if nsfw || used_ids.contains(&post.id) {
            debug!("Skipping post (NSFW or already used): {}", post.title);
            continue;
        }

        let text = if is_self && !post.selftext.trim().is_empty() {
            format!("{}\n\n{}", post.title.trim(), post.selftext.trim())
        } else {
            post.title.trim().to_string()
        };

        // Sanetisierung anwenden
        if let Some(clean) = sanitize_post(&text, &forbidden, max_words) {
            if !clean.trim().is_empty() {
                info!("Selected post: {}", post.title);
                used_ids.insert(post.id.clone());
                save_used_ids(used_path, &used_ids)?;
                return Ok(clean);
            }
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
