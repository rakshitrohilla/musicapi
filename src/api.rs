use anyhow::{anyhow, Result};
use serde::Deserialize;
#[derive(Debug, Clone, Deserialize)]
pub struct SCUser {
    pub username: String,
}
#[derive(Debug, Clone, Deserialize)]
pub struct SCTranscodingFormat {
    pub protocol: String,
    pub mime_type: String,
}
#[derive(Debug, Clone, Deserialize)]
pub struct SCTranscoding {
    pub url: String,
    pub format: SCTranscodingFormat,
}
#[derive(Debug, Clone, Deserialize)]
pub struct SCMedia {
    pub transcodings: Vec<SCTranscoding>,
}
#[derive(Debug, Clone, Deserialize)]
pub struct SCTrack {
    pub id: u64,
    pub title: String,
    pub user: SCUser,
    pub duration: u64, // milliseconds
    pub media: Option<SCMedia>,
}
#[derive(Debug, Deserialize)]
pub struct SCSearchResult {
    pub collection: Vec<SCTrack>,
}
#[derive(Debug, Deserialize)]
struct StreamResponse {
    url: String,
}
pub struct SoundCloudClient {
    client_id: String,
    http: reqwest::Client,
}
impl SoundCloudClient {
    pub fn new(client_id: String) -> Self {
        Self {
            client_id,
            http: reqwest::Client::new(),
        }
    }
    pub async fn search(&self, query: &str, limit: u32) -> Result<Vec<SCTrack>> {
        let url = format!(
            "https://api-v2.soundcloud.com/search/tracks?q={}&client_id={}&limit={}",
            urlencoding::encode(query),
            self.client_id,
            limit
        );
        let resp: SCSearchResult = self.http.get(&url).send().await?.json().await?;
        Ok(resp.collection)
    }
    pub async fn get_stream_url(&self, track: &SCTrack) -> Result<String> {
        let media = track.media.as_ref().ok_or_else(|| anyhow!("No media"))?;
        // Prefer progressive (direct mp3)
        let transcoding = media
            .transcodings
            .iter()
            .find(|t| t.format.protocol == "progressive")
            .or_else(|| media.transcodings.first())
            .ok_or_else(|| anyhow!("No transcodings"))?;
        let url = format!("{}?client_id={}", transcoding.url, self.client_id);
        let resp: StreamResponse = self.http.get(&url).send().await?.json().await?;
        Ok(resp.url)
    }
}
pub fn format_duration(ms: u64) -> String {
    let total_sec = ms / 1000;
    let min = total_sec / 60;
    let sec = total_sec % 60;
    format!("{}:{:02}", min, sec)
}