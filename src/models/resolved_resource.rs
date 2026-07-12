use crate::models::{Playlist, Track, User};

#[derive(Debug, Clone)]
pub enum ResolvedResource {
    Track(Box<Track>),
    User(Box<User>),
    Playlist(Box<Playlist>),
}
