//! Reddit API integration for fetching and processing stories.
//!
//! This module handles communication with Reddit's JSON API to fetch posts from
//! specified subreddits, filters them based on content guidelines, and manages
//! a history of used posts to avoid duplicates.

use crate::utils::{correct_grammar, load_forbidden_words, sanitize_post};
use reqwest::header::USER_AGENT;
use serde::Deserialize;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use tracing::{debug, info};

/// Top-level Reddit API response structure for subreddit listings
#[derive(Debug, Deserialize)]
pub struct RedditListing {
    pub data: RedditListingData,
}

/// Data container for Reddit listing responses
#[derive(Debug, Deserialize)]
pub struct RedditListingData {
    pub children: Vec<RedditChild>,
}

/// Wrapper for individual Reddit posts in API responses
#[derive(Debug, Deserialize)]
pub struct RedditChild {
    pub data: RedditPost,
}

/// Individual Reddit post data structure
#[derive(Debug, Deserialize)]
pub struct RedditPost {
    /// Unique post identifier
    pub id: String,
    /// Post title
    pub title: String,
    /// Post body text (for self posts)
    pub selftext: String,
    /// Whether this is a self post (text post vs link)
    pub is_self: Option<bool>,
    /// Whether the post is marked as NSFW
    pub over_18: Option<bool>,
}

/// Fetches a suitable Reddit story from the specified subreddit.
///
/// This function retrieves posts from Reddit's JSON API, filters them based on
/// content guidelines (NSFW, forbidden words, length requirements), and returns
/// the first suitable story found. It also maintains a history of used posts
/// to avoid duplicates.
///
/// # Arguments
/// * `subreddit` - The subreddit name to fetch from (without 'r/' prefix)
/// * `limit` - Maximum number of posts to fetch from Reddit API
/// * `min_chars` - Minimum character count required for a story
///
/// # Returns
/// * `Ok(String)` - The selected and processed story text
/// * `Err` - If no suitable posts are found or API errors occur
pub async fn fetch_reddit_story(
    subreddit: &str,
    limit: usize,
    min_chars: usize,
) -> anyhow::Result<String> {
    let url = format!("https://www.reddit.com/r/{subreddit}/hot.json?limit={limit}");
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

        if let Some(clean) = sanitize_post(&text, &forbidden, max_words)
            && !clean.trim().is_empty()
            && clean.chars().count() >= min_chars
        {
            let corrected = correct_grammar(&clean).await.unwrap_or(clean.clone());
            info!("Selected post: {}", post.title);
            used_ids.insert(post.id.clone());
            save_used_ids(used_path, &used_ids)?;
            return Ok(corrected);
        }
    }
    anyhow::bail!("No suitable posts found in subreddit {}", subreddit);
}

/// Loads the set of previously used Reddit post IDs from a JSON file.
///
/// # Arguments
/// * `path` - Path to the JSON file containing used post IDs
///
/// # Returns
/// * `Ok(HashSet<String>)` - Set of used post IDs, empty if file doesn't exist
/// * `Err` - If file exists but cannot be read or parsed
fn load_used_ids(path: &str) -> anyhow::Result<HashSet<String>> {
    if !Path::new(path).exists() {
        return Ok(HashSet::new());
    }
    let data = fs::read_to_string(path)?;
    let ids: Vec<String> = serde_json::from_str(&data)?;
    Ok(ids.into_iter().collect())
}

/// Saves the set of used Reddit post IDs to a JSON file.
///
/// # Arguments
/// * `path` - Path where the JSON file should be written
/// * `ids` - Set of post IDs to save
///
/// # Returns
/// * `Ok(())` - If the file was successfully written
/// * `Err` - If the file cannot be written or JSON serialization fails
fn save_used_ids(path: &str, ids: &HashSet<String>) -> anyhow::Result<()> {
    let data = serde_json::to_string_pretty(&ids)?;
    fs::write(path, data)?;
    Ok(())
}
