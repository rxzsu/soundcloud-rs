use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    pub id: usize,
    pub urn: Option<String>,
    pub permalink: String,
    pub username: String,
    pub uri: String,
    pub permalink_url: String,
    pub avatar_url: String,
    pub country: Option<String>,
    pub full_name: Option<String>,
    pub city: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "discogs-name")]
    pub discogs_name: Option<String>,
    #[serde(rename = "myspace-name")]
    pub myspace_name: Option<String>,
    pub website: Option<String>,
    #[serde(rename = "website-title")]
    pub website_title: Option<String>,
    pub online: Option<bool>,
    pub track_count: Option<usize>,
    pub playlist_count: Option<usize>,
    pub followers_count: Option<usize>,
    pub followings_count: Option<usize>,
    pub public_favorites_count: Option<usize>,
}
