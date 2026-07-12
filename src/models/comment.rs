use crate::models::User;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Comment {
    pub id: usize,
    pub urn: Option<String>,
    pub uri: String,
    pub created_at: String,
    pub body: String,
    pub timestamp: Option<usize>,
    pub user_id: usize,
    pub user: User,
    pub track_id: usize,
}
