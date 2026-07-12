use crate::client::Client;
use crate::error::Result;
use crate::models::{Playlist, Track};

pub struct UserReposts<'a> {
    client: &'a Client,
    user_urn: String,
}

impl<'a> UserReposts<'a> {
    pub fn new(client: &'a Client, user_urn: &str) -> Self {
        UserReposts {
            client,
            user_urn: user_urn.to_owned(),
        }
    }

    pub async fn tracks(&self, limit: Option<u64>) -> Result<Vec<Track>> {
        let params = limit.map(|l| vec![("limit", l.to_string())]);
        let res = self
            .client
            .get(&format!("/users/{}/reposts/tracks", self.user_urn), params)
            .await?;
        Ok(res.json().await?)
    }

    pub async fn playlists(&self, limit: Option<u64>) -> Result<Vec<Playlist>> {
        let params = limit.map(|l| vec![("limit", l.to_string())]);
        let res = self
            .client
            .get(
                &format!("/users/{}/reposts/playlists", self.user_urn),
                params,
            )
            .await?;
        Ok(res.json().await?)
    }
}

pub struct MyReposts<'a> {
    client: &'a Client,
}

impl<'a> MyReposts<'a> {
    pub fn new(client: &'a Client) -> Self {
        MyReposts { client }
    }

    pub async fn tracks(&self, limit: Option<u64>) -> Result<Vec<Track>> {
        let params = limit.map(|l| vec![("limit", l.to_string())]);
        let res = self
            .client
            .get("/me/reposts/tracks", params)
            .await?;
        Ok(res.json().await?)
    }

    pub async fn playlists(&self, limit: Option<u64>) -> Result<Vec<Playlist>> {
        let params = limit.map(|l| vec![("limit", l.to_string())]);
        let res = self
            .client
            .get("/me/reposts/playlists", params)
            .await?;
        Ok(res.json().await?)
    }

    pub async fn repost_track(&self, track_urn: &str) -> Result<()> {
        self.client
            .post(&format!("/reposts/tracks/{}", track_urn))
            .await?;
        Ok(())
    }

    pub async fn remove_repost_track(&self, track_urn: &str) -> Result<()> {
        self.client
            .delete(&format!("/reposts/tracks/{}", track_urn))
            .await?;
        Ok(())
    }

    pub async fn repost_playlist(&self, playlist_urn: &str) -> Result<()> {
        self.client
            .post(&format!("/reposts/playlists/{}", playlist_urn))
            .await?;
        Ok(())
    }

    pub async fn remove_repost_playlist(&self, playlist_urn: &str) -> Result<()> {
        self.client
            .delete(&format!("/reposts/playlists/{}", playlist_urn))
            .await?;
        Ok(())
    }
}
