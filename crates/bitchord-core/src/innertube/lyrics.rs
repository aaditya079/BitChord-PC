use crate::innertube::models::LyricLine;
use regex::Regex;
use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LyricsError {
    #[error("Network request failed: {0}")]
    Network(#[from] reqwest::Error),
    #[error("Lyrics not found")]
    NotFound,
    #[error("Failed to parse lyrics")]
    ParseError,
}

#[derive(Debug, Deserialize)]
struct LrcLibResponse {
    synced_lyrics: Option<String>,
    plain_lyrics: Option<String>,
}

pub struct LyricsClient {
    client: reqwest::Client,
}

impl LyricsClient {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }

    /// Fetches and parses synchronized lyrics from LRCLIB.
    pub async fn get_lyrics(&self, artist: &str, title: &str) -> Result<Vec<LyricLine>, LyricsError> {
        let url = format!(
            "https://lrclib.net/api/get?artist_name={}&track_name={}",
            urlencoding::encode(artist),
            urlencoding::encode(title)
        );

        let resp = self
            .client
            .get(&url)
            .header("User-Agent", "BitChord/3.0 (https://github.com/aaditya079/BitChord-PC)")
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(LyricsError::NotFound);
        }

        let data: LrcLibResponse = resp.json().await?;
        if let Some(synced) = data.synced_lyrics {
            Ok(Self::parse_lrc(&synced))
        } else if let Some(plain) = data.plain_lyrics {
            // Unsynchronized fallback: space lines evenly or at 0
            Ok(plain
                .lines()
                .enumerate()
                .map(|(i, line)| LyricLine {
                    timestamp_ms: (i as u64) * 3000,
                    text: line.trim().to_string(),
                })
                .filter(|l| !l.text.is_empty())
                .collect())
        } else {
            Err(LyricsError::NotFound)
        }
    }

    /// Parses standard LRC timestamp strings like `[01:23.45] lyric text` into `LyricLine`s.
    pub fn parse_lrc(lrc_text: &str) -> Vec<LyricLine> {
        let re = Regex::new(r"\[(\d{2}):(\d{2})(?:\.(\d{2,3}))?\](.*)").unwrap();
        let mut lines = Vec::new();

        for line in lrc_text.lines() {
            if let Some(caps) = re.captures(line) {
                let mins: u64 = caps[1].parse().unwrap_or(0);
                let secs: u64 = caps[2].parse().unwrap_or(0);
                let millis: u64 = caps.get(3).map_or(0, |m| {
                    let s = m.as_str();
                    if s.len() == 2 {
                        s.parse::<u64>().unwrap_or(0) * 10
                    } else {
                        s.parse::<u64>().unwrap_or(0)
                    }
                });

                let timestamp_ms = (mins * 60 + secs) * 1000 + millis;
                let text = caps[4].trim().to_string();

                lines.push(LyricLine {
                    timestamp_ms,
                    text,
                });
            }
        }

        lines.sort_by_key(|l| l.timestamp_ms);
        lines
    }
}

mod urlencoding {
    pub fn encode(s: &str) -> String {
        url::form_urlencoded::byte_serialize(s.as_bytes()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_lrc() {
        let lrc = "[00:12.34]Hello world\n[01:05.800]Second line\n";
        let parsed = LyricsClient::parse_lrc(lrc);
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].timestamp_ms, 12340);
        assert_eq!(parsed[0].text, "Hello world");
        assert_eq!(parsed[1].timestamp_ms, 65800);
        assert_eq!(parsed[1].text, "Second line");
    }
}
