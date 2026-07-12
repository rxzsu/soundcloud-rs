use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct TrackInsight {
    pub track_id: u64,
    pub plays: Option<u64>,
    pub downloads: Option<u64>,
    pub comments: Option<u64>,
    pub likes: Option<u64>,
    pub reposts: Option<u64>,
}
