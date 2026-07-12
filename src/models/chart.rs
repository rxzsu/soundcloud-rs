use serde::Deserialize;

use crate::models::{Playlist, Track};

#[derive(Debug, Clone, Deserialize)]
pub struct ChartEntry {
    pub score: Option<f64>,
    pub track: Option<Track>,
    pub playlist: Option<Playlist>,
}
