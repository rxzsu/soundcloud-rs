use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct TrackStreams {
    pub hls_aac_160_url: Option<String>,
    pub hls_aac_96_url: Option<String>,
    pub preview_mp3_128_url: Option<String>,
    #[serde(default)]
    pub http_mp3_128_url: Option<String>,
    #[serde(default)]
    pub hls_mp3_128_url: Option<String>,
    #[serde(default)]
    pub hls_opus_64_url: Option<String>,
}
