use url::Url;

use futures::prelude::*;
use soundcloud::*;

const USER_ID: usize = 31506117;
const TRACK_ID: usize = 505512390;

fn client() -> Client {
    Client::new(&std::env::var("SOUNDCLOUD_CLIENT_ID").expect("SOUNDCLOUD_CLIENT_ID"))
}

fn authenticated_client() -> Client {
    let mut client = client();
    client.authenticate_with_token(std::env::var("SOUNDCLOUD_AUTH_TOKEN").expect("SOUNDCLOUD_AUTH_TOKEN"));

    client
}

#[tokio::test]
async fn test_fetch_my_playlists() {
    let client = authenticated_client();
    assert!(!client.my_playlists().await.unwrap().is_empty());
}

#[tokio::test]
async fn test_fetch_likes() {
    let client = authenticated_client();
    assert!(!client.likes().await.unwrap().is_empty());
}

#[tokio::test]
async fn test_resolve_track() {
    let result = client()
        .resolve("https://soundcloud.com/djmaksgermany/invites-feat-maks-warm-up-mix")
        .await;

    assert_eq!(
        result.unwrap(),
        Url::parse("https://api.soundcloud.com/tracks/330733497").unwrap()
    );
}

#[tokio::test]
async fn test_search_tracks() {
    let result = client().tracks().query(Some("monstercat")).get().await;

    assert!(!result.unwrap().is_empty());
}

#[tokio::test]
async fn test_get_track() {
    let track = client().tracks().id(263801976).get().await.unwrap();

    assert_eq!(track.id, 263801976);
}

#[tokio::test]
async fn test_get_playlists() {
    let result = client().playlists().query("monstercat").get().await;

    assert!(!result.unwrap().is_empty());
}

#[tokio::test]
async fn test_get_playlist() {
    let playlist = client().playlist(565064082).get().await.unwrap();

    assert_eq!(playlist.id, 565064082);
}

#[tokio::test]
async fn test_download() {
    use tokio::fs::{remove_file, File};
    use tokio_util::compat::TokioAsyncWriteCompatExt;

    let client = client();
    let path = "hi.mp3".to_string();
    let track = client.tracks().id(263801976).get().await.unwrap();
    let mut outfile = File::create(&path).await.unwrap().compat_write();

    let num_bytes = client.download(&track, &mut outfile).await.unwrap();
    assert!(num_bytes > 0);
    let _ = remove_file(path).await;
}

#[tokio::test]
async fn test_stream() {
    use tokio::fs::{remove_file, File};
    use tokio_util::compat::TokioAsyncWriteCompatExt;

    let client = client();
    let path = "test.mp3".to_string();
    let track = client.tracks().id(263801976).get().await.unwrap();
    let mut outfile = File::create(&path).await.unwrap().compat_write();

    let num_bytes = client.stream(&track, &mut outfile).await.unwrap();
    assert!(num_bytes > 0);
    let _ = remove_file(path).await;
}

#[tokio::test]
async fn test_get_user() {
    let user = client().user(8553751).get().await.unwrap();

    assert_eq!(user.id, 8553751);
}

#[tokio::test]
async fn test_get_users() {
    let users = client()
        .users()
        .query(Some("monstercat"))
        .get()
        .await
        .unwrap();

    assert!(!users.is_empty());
}

#[tokio::test]
async fn test_get_user_from_permalink() {
    let user = client()
        .users()
        .permalink("djmaksgermany")
        .await
        .unwrap()
        .get()
        .await
        .unwrap();

    assert_eq!(user.id, USER_ID);
}

#[tokio::test]
async fn test_get_first_page_user_tracks() {
    let tracks = client().user(USER_ID).tracks();
    let tracks: Vec<Track> = tracks
        .get(Default::default(), 1)
        .try_collect()
        .await
        .unwrap();

    assert!(!tracks.is_empty());
}

#[tokio::test]
async fn test_paginate_user_tracks() {
    let tracks = client().user(USER_ID).tracks();
    let tracks: Vec<Track> = tracks.iter(Default::default()).try_collect().await.unwrap();

    assert!(!tracks.is_empty());
}

#[tokio::test]
async fn test_user_web_profile() {
    let profiles = client().user(USER_ID).web_profiles();
    let profiles: Vec<WebProfile> = profiles
        .iter(Default::default())
        .try_collect()
        .await
        .unwrap();

    assert!(!profiles.is_empty());
}

#[tokio::test]
async fn test_user_playlists() {
    let playlists = client().user(USER_ID).playlists();
    let playlists: Vec<Playlist> = playlists
        .iter(Default::default())
        .try_collect()
        .await
        .unwrap();

    assert!(!playlists.is_empty());
}

#[tokio::test]
async fn test_user_followings() {
    let followings = client().user(USER_ID).followings();
    let users: Vec<User> = followings
        .iter(Default::default())
        .take(50)
        .try_collect()
        .await
        .unwrap();

    assert_eq!(50, users.len());
}

#[tokio::test]
async fn test_user_followers() {
    let followers = client().user(USER_ID).followers();
    let users: Vec<User> = followers
        .iter(Default::default())
        .take(50)
        .try_collect()
        .await
        .unwrap();

    assert_eq!(50, users.len());
}

#[tokio::test]
async fn test_user_likes() {
    let likes = client().user(USER_ID).likes();
    let tracks: Vec<Track> = likes
        .iter(Default::default())
        .take(50)
        .try_collect()
        .await
        .unwrap();

    assert_eq!(50, tracks.len());
}

#[tokio::test]
async fn test_track_comments() {
    let comments = client().track(TRACK_ID).comments();
    let comments: Vec<Comment> = comments
        .iter(Default::default())
        .take(50)
        .try_collect()
        .await
        .unwrap();

    assert_eq!(50, comments.len());
}

#[tokio::test]
async fn test_track_likers() {
    let likers = client().track(TRACK_ID).likers();
    let users: Vec<User> = likers
        .iter(Default::default())
        .take(50)
        .try_collect()
        .await
        .unwrap();

    assert_eq!(50, users.len());
}

#[tokio::test]
async fn test_related_tracks() {
    let related = client().track(TRACK_ID).related_tracks();
    let tracks: Vec<Track> = related
        .iter(Default::default())
        .take(30)
        .try_collect()
        .await
        .unwrap();

    assert_eq!(30, tracks.len());
}
