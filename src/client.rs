use std::borrow::Borrow;

use futures::future::BoxFuture;
use futures::io::AsyncWrite;
use futures::prelude::*;
use futures::stream::{BoxStream, TryStreamExt};
use serde::de::DeserializeOwned;
use url::Url;

use crate::apis::{
    MyReposts, PlaylistRequestBuilder, SinglePlaylistRequestBuilder, SingleTrackRequestBuilder,
    SingleUserRequestBuilder, TrackRequestBuilder, TrackUploadBuilder, UserReposts,
    UserRequestBuilder,
};
use crate::error::{Error, Result};
use crate::models::{ChartEntry, Playlist, ResolvedResource, Track, TrackInsight, User};
use crate::page::Page;

#[derive(Clone, Debug)]
pub struct Client {
    pub(crate) host: String,
    pub(crate) client_id: String,
    pub(crate) auth_token: Option<String>,
    pub(crate) http_client: reqwest::Client,
}

impl Client {
    /// Constructs a new `Client` with the provided `client_id`.
    ///
    /// # Examples
    ///
    /// ```
    /// use soundcloud::Client;
    ///
    /// let client = Client::new(env!("SOUNDCLOUD_CLIENT_ID"));
    /// ```
    pub fn new(client_id: &str) -> Client {
        let client = reqwest::ClientBuilder::new()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .unwrap();

        Client {
            host: super::API_HOST.to_owned(),
            client_id: client_id.to_owned(),
            http_client: client,
            auth_token: None,
        }
    }

    /// Returns the client id.
    pub fn client_id(&self) -> &str {
        &self.client_id
    }

    pub fn authenticate_with_token(&mut self, token: String) {
        self.auth_token = Some(token);
    }

    /// Creates and sends a HTTP GET request to the API endpoint.
    ///
    /// A `client_id` parameter will automatically be added to the request.
    ///
    /// Returns the HTTP response on success, an error otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::Read;
    /// use soundcloud::Client;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///   let client = Client::new(env!("SOUNDCLOUD_CLIENT_ID"));
    ///   let response = client.get("/resolve", Some(&[("url",
    ///   "https://soundcloud.com/firepowerrecs/afk-shellshock-kamikaze-promo-mix-lock-load-series-vol-20")])).await;
    ///
    ///   let buffer = response.unwrap().text().await.unwrap();
    ///
    ///   assert!(!buffer.is_empty());
    ///}
    /// ```
    pub async fn get<I, K, V>(&self, path: &str, params: Option<I>) -> Result<reqwest::Response>
    where
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        let mut url = Url::parse(&format!("{}{}", self.host, path))?;

        {
            let mut query_pairs = url.query_pairs_mut();
            query_pairs.append_pair("client_id", &self.client_id);

            if let Some(params) = params {
                query_pairs.extend_pairs(params);
            }
        }

        let mut headers = reqwest::header::HeaderMap::new();

        if self.auth_token.is_some() {
            let token = self.auth_token.clone().unwrap();
            headers.insert(
                reqwest::header::AUTHORIZATION,
                format!("OAuth {}", token).parse()?,
            );
        }

        let response = self.http_client.get(url).headers(headers).send().await?;
        response.error_for_status().map_err(Error::from)
    }

    /// Creates and sends a HTTP POST request.
    pub async fn post(&self, path: &str) -> Result<reqwest::Response> {
        let url = Url::parse(&format!("{}{}", self.host, path))?;
        let mut headers = reqwest::header::HeaderMap::new();
        if let Some(ref token) = self.auth_token {
            headers.insert(
                reqwest::header::AUTHORIZATION,
                format!("OAuth {}", token).parse()?,
            );
        }
        let response = self.http_client.post(url).headers(headers).send().await?;
        response.error_for_status().map_err(Error::from)
    }

    /// Creates and sends a HTTP PUT request.
    pub async fn put(&self, path: &str) -> Result<reqwest::Response> {
        let url = Url::parse(&format!("{}{}", self.host, path))?;
        let mut headers = reqwest::header::HeaderMap::new();
        if let Some(ref token) = self.auth_token {
            headers.insert(
                reqwest::header::AUTHORIZATION,
                format!("OAuth {}", token).parse()?,
            );
        }
        let response = self.http_client.put(url).headers(headers).send().await?;
        response.error_for_status().map_err(Error::from)
    }

    /// Creates and sends a HTTP DELETE request.
    pub async fn delete(&self, path: &str) -> Result<reqwest::Response> {
        let url = Url::parse(&format!("{}{}", self.host, path))?;
        let mut headers = reqwest::header::HeaderMap::new();
        if let Some(ref token) = self.auth_token {
            headers.insert(
                reqwest::header::AUTHORIZATION,
                format!("OAuth {}", token).parse()?,
            );
        }
        let response = self.http_client.delete(url).headers(headers).send().await?;
        response.error_for_status().map_err(Error::from)
    }

    pub fn get_stream<T>(&self, path: &str, num_pages: Option<u64>) -> BoxStream<'static, Result<T>>
    where
        T: DeserializeOwned + 'static + Send,
    {
        unfold(
            self.clone(),
            self.get_pages(path),
            num_pages.unwrap_or(u64::MAX),
        )
    }

    fn get_pages<T>(&self, path: &str) -> BoxFuture<'static, Result<Page<T>>>
    where
        T: DeserializeOwned + 'static + Send,
    {
        self.get_page(&(self.host.clone() + path))
    }

    fn get_pages_url<T>(&self, url: &str) -> BoxFuture<'static, Result<Page<T>>>
    where
        T: DeserializeOwned + 'static + Send,
    {
        self.get_page(url)
    }

    fn get_page<T>(&self, path: &str) -> BoxFuture<'static, Result<Page<T>>>
    where
        T: DeserializeOwned + 'static + Send,
    {
        let client = self.http_client.clone();
        let client_id = self.client_id.clone();
        let auth_token = self.auth_token.clone();

        let mut url = Url::parse(path).unwrap();

        if !url.query_pairs().any(|(q, _)| q == "client_id") {
            url.query_pairs_mut()
                .append_pair("client_id", &client_id);
        }

        let mut headers = reqwest::header::HeaderMap::new();

        if let Some(ref token) = auth_token {
            headers.insert(
                reqwest::header::AUTHORIZATION,
                format!("OAuth {}", token).parse().unwrap(),
            );
        }

        let response = client
            .get(url)
            .headers(headers)
            .send()
            .map_err(Error::from);

        Box::pin(response.and_then(move |response| response.json().map_err(Error::from)))
    }

    /// Starts streaming the track provided in the track's `stream_url` to the `writer` if the track
    /// is streamable via the API.
    ///
    /// Returns:
    ///     Number of bytes written if the track was streamed successfully, an error otherwise.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::path::Path;
    /// use soundcloud::Client;
    /// use tokio::fs::File;
    /// use tokio_util::compat::TokioAsyncWriteCompatExt;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///   let client = Client::new(env!("SOUNDCLOUD_CLIENT_ID"));
    ///   let path = Path::new("hi.mp3");
    ///   let track = client.tracks().id(263801976).get().await.unwrap();
    ///   let mut outfile = File::create(path).await.unwrap().compat_write();
    ///   let num_bytes = client.stream(&track, &mut outfile).await.unwrap();
    ///   assert!(num_bytes > 0);
    /// }
    /// ```
    pub async fn stream<W: AsyncWrite + Unpin>(&self, track: &Track, mut writer: W) -> Result<u64> {
        if !track.streamable {
            return Err(Error::TrackNotStreamable);
        }
        self.read_url(track.stream_url.as_ref().unwrap(), &mut writer)
            .await
    }

    /// Starts downloading the track provided in the tracks `download_url` to the `writer` if the track
    /// is downloadable via the API.
    ///
    /// Returns:
    ///     Number of bytes written if the track was downloaded successfully, an error otherwise.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::path::Path;
    /// use soundcloud::Client;
    /// use tokio::fs::File;
    /// use tokio_util::compat::TokioAsyncWriteCompatExt;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///   let client = Client::new(env!("SOUNDCLOUD_CLIENT_ID"));
    ///   let path = Path::new("hi.mp3");
    ///   let track = client.tracks().id(263801976).get().await.unwrap();
    ///   let mut outfile = File::create(path).await.unwrap().compat_write();
    ///   let num_bytes = client.download(&track, &mut outfile).await.unwrap();
    ///   assert!(num_bytes > 0);
    /// }
    /// ```
    pub async fn download<W: AsyncWrite + Unpin>(
        &self,
        track: &Track,
        mut writer: W,
    ) -> Result<u64> {
        if !track.downloadable {
            return Err(Error::TrackNotDownloadable);
        }
        self.read_url(track.download_url.as_ref().unwrap(), &mut writer)
            .await
    }

    /// Copies the data provided from reading in the `url` to the `writer`
    /// if the track is streamable via the API.
    ///
    /// Returns:
    ///     number of bytes written if the resource's data was copied successfully,
    ///     an error otherwise.
    ///
    /// ```
    async fn read_url<W: AsyncWrite + Unpin>(&self, url: &str, mut writer: W) -> Result<u64> {
        let url = self.parse_url(url)?;
        let mut response = self.http_client.get(url).send().await?;
        // Follow the redirect just this once.
        if let Some(header) = response.headers().get(reqwest::header::LOCATION).cloned() {
            let url = Url::parse(header.to_str()?).unwrap();
            response = self.http_client.get(url).send().await?;
        }
        let stream = response.bytes_stream();
        // convert the reqwest::Error into a futures::io::Error
        let stream = stream
            .map_err(futures::io::Error::other)
            .into_async_read();

        let num_bytes = futures::io::copy(stream, &mut writer).await?;

        Ok(num_bytes)
    }

    /// Resolves any soundcloud resource and returns it as a `Url`.
    pub async fn resolve(&self, url: &str) -> Result<Url> {
        let response = self.get("/resolve", Some(&[("url", url)])).await?;

        if let Some(header) = response.headers().get(reqwest::header::LOCATION) {
            Ok(Url::parse(header.to_str()?)?)
        } else {
            Err(Error::ApiError("expected location header".to_owned()))
        }
    }

    /// Returns a builder for a single track-by-id request.
    ///
    /// # Examples
    ///
    /// ```
    /// use soundcloud::Client;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///   let client = Client::new(env!("SOUNDCLOUD_CLIENT_ID"));
    ///   let track = client.track(262681089).get().await;
    ///
    ///   assert_eq!(track.unwrap().id, 262681089);
    /// }
    /// ```
    pub fn track(&self, id: usize) -> SingleTrackRequestBuilder<'_> {
        SingleTrackRequestBuilder::new(self, id)
    }

    /// Returns a builder for searching tracks with multiple criteria.
    ///
    /// # Examples
    ///
    /// ```
    /// use soundcloud::Client;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///   let client = Client::new(env!("SOUNDCLOUD_CLIENT_ID"));
    ///   let tracks = client.tracks().genres(Some(["HipHop"])).get().await;
    ///
    ///   assert!(tracks.unwrap().len() > 0);
    /// }
    /// ```
    pub fn tracks(&self) -> TrackRequestBuilder<'_> {
        TrackRequestBuilder::new(self)
    }

    /// Returns a builder for a single playlist-by-id request.
    ///
    /// # Examples
    ///
    /// ```
    /// use soundcloud::Client;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///   let client = Client::new(env!("SOUNDCLOUD_CLIENT_ID"));
    ///   let playlist = client.playlist(965640322).get().await;
    ///
    ///   assert_eq!(playlist.unwrap().id, 965640322);
    /// }
    /// ```
    pub fn playlist(&self, id: usize) -> SinglePlaylistRequestBuilder<'_> {
        SinglePlaylistRequestBuilder::new(self, id)
    }

    /// Returns a builder for searching playlists with multiple criteria.
    ///
    /// # Examples
    ///
    /// ```
    /// use soundcloud::Client;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///   let client = Client::new(env!("SOUNDCLOUD_CLIENT_ID"));
    ///   let playlists = client.playlists().query("Monstercat").get().await;
    ///
    ///   assert!(playlists.unwrap().len() > 0);
    /// }
    /// ```
    pub fn playlists(&self) -> PlaylistRequestBuilder<'_> {
        PlaylistRequestBuilder::new(self)
    }

    /// Returns list of playlists of the authenticated user
    pub async fn my_playlists(&self) -> Result<Vec<Playlist>> {
        let params = Some(vec![("limit", "500")]);
        let res = self.get("/me/playlists", params).await?;
        let playlists: Vec<Playlist> = res.json().await?;
        Ok(playlists)
    }

    /// Returns details about the given user
    pub fn user(&self, user_id: usize) -> SingleUserRequestBuilder<'_> {
        SingleUserRequestBuilder::new(self, user_id)
    }

    /// Returns a builder for searching users
    pub fn users(&self) -> UserRequestBuilder<'_> {
        UserRequestBuilder::new(self)
    }

    pub async fn likes(&self) -> Result<Vec<Track>> {
        let params = Some(vec![("limit", "500")]);
        let res = self.get("/me/likes/tracks", params).await?;
        let likes: Vec<Track> = res.json().await?;
        Ok(likes)
    }

    /// Like a track by its URN.
    pub async fn like_track(&self, track_urn: &str) -> Result<()> {
        self.post(&format!("/likes/tracks/{}", track_urn)).await?;
        Ok(())
    }

    /// Unlike a track by its URN.
    pub async fn unlike_track(&self, track_urn: &str) -> Result<()> {
        self.delete(&format!("/likes/tracks/{}", track_urn)).await?;
        Ok(())
    }

    /// Like a playlist by its URN.
    pub async fn like_playlist(&self, playlist_urn: &str) -> Result<()> {
        self.post(&format!("/likes/playlists/{}", playlist_urn)).await?;
        Ok(())
    }

    /// Unlike a playlist by its URN.
    pub async fn unlike_playlist(&self, playlist_urn: &str) -> Result<()> {
        self.delete(&format!("/likes/playlists/{}", playlist_urn)).await?;
        Ok(())
    }

    /// Follow a user by their URN.
    pub async fn follow_user(&self, user_urn: &str) -> Result<()> {
        self.put(&format!("/me/followings/{}", user_urn)).await?;
        Ok(())
    }

    /// Unfollow a user by their URN.
    pub async fn unfollow_user(&self, user_urn: &str) -> Result<()> {
        self.delete(&format!("/me/followings/{}", user_urn)).await?;
        Ok(())
    }

    /// Returns reposts for the authenticated user.
    pub fn my_reposts(&self) -> MyReposts<'_> {
        MyReposts::new(self)
    }

    /// Returns reposts for a specific user.
    pub fn user_reposts(&self, user_urn: &str) -> UserReposts<'_> {
        UserReposts::new(self, user_urn)
    }

    /// Returns a builder for uploading a new track with full metadata support.
    pub fn upload_track(&self, title: &str, asset_data: Vec<u8>) -> TrackUploadBuilder<'_> {
        TrackUploadBuilder::new(self, title, asset_data)
    }

    /// Upload a new track (simple method, title + audio only).
    /// For full metadata control, use `upload_track().genre(...).tags(...)...send().await`.
    pub async fn upload_track_simple(&self, title: &str, asset_data: Vec<u8>) -> Result<Track> {
        let mut url = Url::parse(&format!("{}/tracks", self.host))?;
        url.query_pairs_mut()
            .append_pair("client_id", &self.client_id);
        let mut headers = reqwest::header::HeaderMap::new();
        if let Some(ref token) = self.auth_token {
            headers.insert(
                reqwest::header::AUTHORIZATION,
                format!("OAuth {}", token).parse()?,
            );
        }

        let form = reqwest::multipart::Form::new()
            .text("track[title]", title.to_owned())
            .part("track[asset_data]", reqwest::multipart::Part::bytes(asset_data));

        let response = self
            .http_client
            .post(url)
            .headers(headers)
            .multipart(form)
            .send()
            .await?;
        let track: Track = response.error_for_status()?.json().await?;
        Ok(track)
    }

    /// Update a track's metadata.
    pub async fn update_track(&self, track_urn: &str, body: serde_json::Value) -> Result<Track> {
        let mut url = Url::parse(&format!("{}/tracks/{}", self.host, track_urn))?;
        url.query_pairs_mut()
            .append_pair("client_id", &self.client_id);
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            "application/json".parse().unwrap(),
        );
        if let Some(ref token) = self.auth_token {
            headers.insert(
                reqwest::header::AUTHORIZATION,
                format!("OAuth {}", token).parse()?,
            );
        }

        let response = self
            .http_client
            .put(url)
            .headers(headers)
            .json(&body)
            .send()
            .await?;
        let track: Track = response.error_for_status()?.json().await?;
        Ok(track)
    }

    /// Delete a track by its URN.
    pub async fn delete_track(&self, track_urn: &str) -> Result<()> {
        let mut url = Url::parse(&format!("{}/tracks/{}", self.host, track_urn))?;
        url.query_pairs_mut()
            .append_pair("client_id", &self.client_id);
        let mut headers = reqwest::header::HeaderMap::new();
        if let Some(ref token) = self.auth_token {
            headers.insert(
                reqwest::header::AUTHORIZATION,
                format!("OAuth {}", token).parse()?,
            );
        }

        self.http_client
            .delete(url)
            .headers(headers)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    /// Returns the authenticated user's recently played tracks.
    pub async fn recently_played(&self) -> Result<Vec<Track>> {
        let res = self.get("/me/recently-played/tracks", None::<&[(&str, &str)]>).await?;
        Ok(res.json().await?)
    }

    /// Returns the authenticated user's feed.
    pub async fn feed(&self, limit: Option<u64>) -> Result<Vec<crate::models::Track>> {
        let params = limit.map(|l| vec![("limit", l.to_string())]);
        let res = self.get("/me/feed", params).await?;
        Ok(res.json().await?)
    }

    /// Create a new playlist.
    pub async fn create_playlist(&self, body: serde_json::Value) -> Result<crate::models::Playlist> {
        let mut url = Url::parse(&format!("{}/playlists", self.host))?;
        url.query_pairs_mut()
            .append_pair("client_id", &self.client_id);
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            "application/json".parse().unwrap(),
        );
        if let Some(ref token) = self.auth_token {
            headers.insert(
                reqwest::header::AUTHORIZATION,
                format!("OAuth {}", token).parse()?,
            );
        }

        let response = self
            .http_client
            .post(url)
            .headers(headers)
            .json(&body)
            .send()
            .await?;
        let playlist: crate::models::Playlist = response.error_for_status()?.json().await?;
        Ok(playlist)
    }

    /// Update a playlist.
    pub async fn update_playlist(
        &self,
        playlist_urn: &str,
        body: serde_json::Value,
    ) -> Result<crate::models::Playlist> {
        let mut url = Url::parse(&format!("{}/playlists/{}", self.host, playlist_urn))?;
        url.query_pairs_mut()
            .append_pair("client_id", &self.client_id);
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            "application/json".parse().unwrap(),
        );
        if let Some(ref token) = self.auth_token {
            headers.insert(
                reqwest::header::AUTHORIZATION,
                format!("OAuth {}", token).parse()?,
            );
        }

        let response = self
            .http_client
            .put(url)
            .headers(headers)
            .json(&body)
            .send()
            .await?;
        let playlist: crate::models::Playlist = response.error_for_status()?.json().await?;
        Ok(playlist)
    }

    /// Delete a playlist by its URN.
    pub async fn delete_playlist(&self, playlist_urn: &str) -> Result<()> {
        let mut url = Url::parse(&format!("{}/playlists/{}", self.host, playlist_urn))?;
        url.query_pairs_mut()
            .append_pair("client_id", &self.client_id);
        let mut headers = reqwest::header::HeaderMap::new();
        if let Some(ref token) = self.auth_token {
            headers.insert(
                reqwest::header::AUTHORIZATION,
                format!("OAuth {}", token).parse()?,
            );
        }

        self.http_client
            .delete(url)
            .headers(headers)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    /// Returns stream URLs for a track (AAC HLS streams).
    pub async fn track_streams(&self, track_urn: &str) -> Result<crate::models::TrackStreams> {
        let no_params: Option<&[(&str, &str)]> = None;
        let res = self
            .get(&format!("/tracks/{}/streams", track_urn), no_params)
            .await?;
        Ok(res.json().await?)
    }

    /// Returns the authenticated user's profile.
    pub async fn me(&self) -> Result<User> {
        let res = self.get("/me", None::<&[(&str, &str)]>).await?;
        Ok(res.json().await?)
    }

    /// Resolves any SoundCloud URL and returns a typed resource.
    ///
    /// Unlike `resolve()` which returns only the redirect URL, this method
    /// follows the redirect and deserializes the result into a `ResolvedResource`
    /// enum (Track, User, or Playlist).
    pub async fn resolve_typed(&self, url: &str) -> Result<ResolvedResource> {
        let response = self
            .get("/resolve", Some(&[("url", url)]))
            .await?;
        let location = response
            .headers()
            .get(reqwest::header::LOCATION)
            .ok_or_else(|| Error::ApiError("expected location header".to_owned()))?
            .to_str()?
            .to_owned();
        let location_url = Url::parse(&location)?;
        let path = location_url.path().to_owned();

        let no_params: Option<&[(&str, &str)]> = None;
        let res = self.get(&path, no_params).await?;

        if path.contains("/tracks/") {
            Ok(ResolvedResource::Track(Box::new(res.json().await?)))
        } else if path.contains("/users/") {
            Ok(ResolvedResource::User(Box::new(res.json().await?)))
        } else if path.contains("/playlists/") {
            Ok(ResolvedResource::Playlist(Box::new(res.json().await?)))
        } else {
            Err(Error::ApiError(format!("unknown resource type: {}", path)))
        }
    }

    /// Returns chart data (top tracks/playlists).
    pub async fn charts(&self, kind: Option<&str>, genre: Option<&str>) -> Result<Vec<ChartEntry>> {
        let mut params = Vec::new();
        if let Some(k) = kind {
            params.push(("kind", k));
        }
        if let Some(g) = genre {
            params.push(("genre", g));
        }
        let res = self.get("/charts", Some(params)).await?;
        Ok(res.json().await?)
    }

    /// Returns trending tracks.
    pub async fn trending(&self, genre: Option<&str>) -> Result<Vec<Track>> {
        let params = genre.map(|g| vec![("genre", g)]);
        let res = self.get("/trending", params).await?;
        Ok(res.json().await?)
    }

    /// Returns personalized track recommendations for the authenticated user.
    pub async fn recommendations(&self) -> Result<Vec<Track>> {
        let res = self.get("/me/recommendations", None::<&[(&str, &str)]>).await?;
        Ok(res.json().await?)
    }

    /// Returns track insight/stats for the authenticated user's tracks.
    pub async fn track_insights(&self) -> Result<Vec<TrackInsight>> {
        let res = self
            .get("/me/insights/tracks", None::<&[(&str, &str)]>)
            .await?;
        Ok(res.json().await?)
    }

    /// Returns insight/stats for a specific track owned by the authenticated user.
    pub async fn track_insight(&self, track_urn: &str) -> Result<TrackInsight> {
        let res = self
            .get(&format!("/me/insights/tracks/{}", track_urn), None::<&[(&str, &str)]>)
            .await?;
        Ok(res.json().await?)
    }

    /// Checks if the authenticated user follows a given user.
    pub async fn follows_user(&self, user_urn: &str) -> Result<bool> {
        match self
            .get(&format!("/me/followings/{}", user_urn), None::<&[(&str, &str)]>)
            .await
        {
            Ok(_) => Ok(true),
            Err(Error::HttpError(e)) if e.status() == Some(reqwest::StatusCode::NOT_FOUND) => {
                Ok(false)
            }
            Err(e) => Err(e),
        }
    }

    /// Adds tracks to a playlist by their URNs.
    pub async fn add_tracks_to_playlist(
        &self,
        playlist_urn: &str,
        track_urns: &[&str],
    ) -> Result<Playlist> {
        let current = self
            .get(&format!("/playlists/{}", playlist_urn), None::<&[(&str, &str)]>)
            .await?;
        let playlist: Playlist = current.json().await?;
        let mut existing: Vec<String> = playlist
            .tracks
            .as_deref()
            .unwrap_or_default()
            .iter()
            .filter_map(|t| t.urn.as_deref())
            .map(String::from)
            .collect();
        for urn in track_urns {
            if !existing.contains(&urn.to_string()) {
                existing.push(urn.to_string());
            }
        }
        let body = serde_json::json!({ "playlist": { "tracks": existing } });
        let mut url = Url::parse(&format!("{}/playlists/{}", self.host, playlist_urn))?;
        url.query_pairs_mut()
            .append_pair("client_id", &self.client_id);
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            "application/json".parse().unwrap(),
        );
        if let Some(ref token) = self.auth_token {
            headers.insert(
                reqwest::header::AUTHORIZATION,
                format!("OAuth {}", token).parse()?,
            );
        }
        let response = self
            .http_client
            .put(url)
            .headers(headers)
            .json(&body)
            .send()
            .await?;
        Ok(response.error_for_status()?.json().await?)
    }

    /// Removes tracks from a playlist by their URNs.
    pub async fn remove_tracks_from_playlist(
        &self,
        playlist_urn: &str,
        track_urns: &[&str],
    ) -> Result<Playlist> {
        let current = self
            .get(&format!("/playlists/{}", playlist_urn), None::<&[(&str, &str)]>)
            .await?;
        let playlist: Playlist = current.json().await?;
        let keep: Vec<String> = playlist
            .tracks
            .as_deref()
            .unwrap_or_default()
            .iter()
            .filter_map(|t| t.urn.as_deref())
            .filter(|u| !track_urns.contains(u))
            .map(String::from)
            .collect();
        let body = serde_json::json!({ "playlist": { "tracks": keep } });
        let mut url = Url::parse(&format!("{}/playlists/{}", self.host, playlist_urn))?;
        url.query_pairs_mut()
            .append_pair("client_id", &self.client_id);
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            "application/json".parse().unwrap(),
        );
        if let Some(ref token) = self.auth_token {
            headers.insert(
                reqwest::header::AUTHORIZATION,
                format!("OAuth {}", token).parse()?,
            );
        }
        let response = self
            .http_client
            .put(url)
            .headers(headers)
            .json(&body)
            .send()
            .await?;
        Ok(response.error_for_status()?.json().await?)
    }

    /// Reorders tracks in a playlist.
    pub async fn reorder_playlist_tracks(
        &self,
        playlist_urn: &str,
        track_urns: &[&str],
    ) -> Result<Playlist> {
        let body = serde_json::json!({ "playlist": { "tracks": track_urns } });
        let mut url = Url::parse(&format!("{}/playlists/{}", self.host, playlist_urn))?;
        url.query_pairs_mut()
            .append_pair("client_id", &self.client_id);
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            "application/json".parse().unwrap(),
        );
        if let Some(ref token) = self.auth_token {
            headers.insert(
                reqwest::header::AUTHORIZATION,
                format!("OAuth {}", token).parse()?,
            );
        }
        let response = self
            .http_client
            .put(url)
            .headers(headers)
            .json(&body)
            .send()
            .await?;
        Ok(response.error_for_status()?.json().await?)
    }

    /// Parses a string and returns a url with the client_id query parameter set.
    fn parse_url<S: AsRef<str>>(&self, url: S) -> Result<Url> {
        let mut url = Url::parse(url.as_ref())?;
        url.query_pairs_mut()
            .append_pair("client_id", &self.client_id);
        Ok(url)
    }
}

/// "unfold" paginated results of a list of soundcloud entities
fn unfold<T>(
    client: Client,
    first: BoxFuture<'static, Result<Page<T>>>,
    num_pages: u64,
) -> BoxStream<'static, Result<T>>
where
    T: DeserializeOwned + 'static + Send,
{
    Box::pin(
        first
            .map_ok(move |page| {
                let count = 1;
                let mut items = page.collection;
                items.reverse();
                let link = page.next_href;
                stream::try_unfold(
                    (client, link, items, count),
                    move |(client, link, mut items, mut count)| async move {
                        match items.pop() {
                            Some(item) => Ok(Some((item, (client, link, items, count)))),
                            None => {
                                if count == num_pages {
                                    Ok(None)
                                } else {
                                    match link {
                                        Some(url) => {
                                            count += 1;
                                            let page = client.get_pages_url(&url).await?;
                                            let link = page.next_href;
                                            let mut items = page.collection;
                                            items.reverse();
                                            match items.pop() {
                                                Some(item) => {
                                                    Ok(Some((item, (client, link, items, count))))
                                                }
                                                None => Ok(None),
                                            }
                                        }
                                        None => Ok(None),
                                    }
                                }
                            }
                        }
                    },
                )
            })
            .try_flatten_stream(),
    )
}
