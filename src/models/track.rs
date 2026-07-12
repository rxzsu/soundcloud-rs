use serde::{Deserialize, Serialize};

use crate::models::{App, User};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Track {
    pub id: u64,
    pub urn: Option<String>,
    pub created_at: String,
    pub user: User,
    pub title: String,
    pub permalink_url: String,
    pub uri: String,
    pub sharing: String,
    pub purchase_url: Option<String>,
    pub artwork_url: Option<String>,
    pub description: Option<String>,
    pub duration: u64,
    pub genre: Option<String>,
    pub tags: Option<String>,
    pub label_name: Option<String>,
    pub release: Option<String>,
    pub release_day: Option<u64>,
    pub release_month: Option<u64>,
    pub release_year: Option<u64>,
    pub streamable: bool,
    pub downloadable: bool,
    pub purchase_title: Option<String>,
    pub license: String,
    pub waveform_url: String,
    pub download_url: Option<String>,
    pub stream_url: Option<String>,
    pub bpm: Option<u64>,
    pub commentable: bool,
    pub isrc: Option<String>,
    pub key_signature: Option<String>,
    pub comment_count: Option<u64>,
    pub download_count: Option<u64>,
    pub playback_count: Option<u64>,
    pub favoritings_count: Option<u64>,
    pub created_with: Option<App>,
    pub asset_data: Option<Vec<u8>>,
    pub artwork_data: Option<Vec<u8>>,
    pub user_favorite: Option<bool>,
    pub reveal_stats: Option<bool>,
    pub reveal_comments: Option<bool>,
    pub access: Option<String>,
}

impl PartialEq for Track {
    fn eq(&self, other: &Track) -> bool {
        other.id == self.id
    }
}
