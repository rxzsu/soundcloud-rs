use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct App {
    pub id: usize,
    pub urn: Option<String>,
    pub uri: String,
    pub permalink_url: String,
    pub external_url: String,
    pub creator: Option<String>,
}
