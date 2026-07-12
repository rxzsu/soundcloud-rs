use crate::client::Client;
use crate::error::Result;
use crate::models::Track;
use url::Url;

#[derive(Debug)]
pub struct TrackUploadBuilder<'a> {
    client: &'a Client,
    title: String,
    asset_data: Vec<u8>,
    genre: Option<String>,
    tags: Option<String>,
    description: Option<String>,
    artwork_data: Option<Vec<u8>>,
    sharing: Option<String>,
    downloadable: Option<bool>,
    streamable: Option<bool>,
    license: Option<String>,
    purchase_url: Option<String>,
    purchase_title: Option<String>,
    release_day: Option<u32>,
    release_month: Option<u32>,
    release_year: Option<u32>,
    bpm: Option<u32>,
    key_signature: Option<String>,
    isrc: Option<String>,
}

impl<'a> TrackUploadBuilder<'a> {
    pub fn new(client: &'a Client, title: &str, asset_data: Vec<u8>) -> Self {
        TrackUploadBuilder {
            client,
            title: title.to_owned(),
            asset_data,
            genre: None,
            tags: None,
            description: None,
            artwork_data: None,
            sharing: None,
            downloadable: None,
            streamable: None,
            license: None,
            purchase_url: None,
            purchase_title: None,
            release_day: None,
            release_month: None,
            release_year: None,
            bpm: None,
            key_signature: None,
            isrc: None,
        }
    }

    pub fn genre(mut self, genre: &str) -> Self {
        self.genre = Some(genre.to_owned());
        self
    }

    pub fn tags(mut self, tags: &str) -> Self {
        self.tags = Some(tags.to_owned());
        self
    }

    pub fn description(mut self, description: &str) -> Self {
        self.description = Some(description.to_owned());
        self
    }

    pub fn artwork(mut self, artwork_data: Vec<u8>) -> Self {
        self.artwork_data = Some(artwork_data);
        self
    }

    pub fn sharing(mut self, sharing: &str) -> Self {
        self.sharing = Some(sharing.to_owned());
        self
    }

    pub fn downloadable(mut self, downloadable: bool) -> Self {
        self.downloadable = Some(downloadable);
        self
    }

    pub fn streamable(mut self, streamable: bool) -> Self {
        self.streamable = Some(streamable);
        self
    }

    pub fn license(mut self, license: &str) -> Self {
        self.license = Some(license.to_owned());
        self
    }

    pub fn purchase_url(mut self, url: &str) -> Self {
        self.purchase_url = Some(url.to_owned());
        self
    }

    pub fn purchase_title(mut self, title: &str) -> Self {
        self.purchase_title = Some(title.to_owned());
        self
    }

    pub fn release_date(mut self, day: u32, month: u32, year: u32) -> Self {
        self.release_day = Some(day);
        self.release_month = Some(month);
        self.release_year = Some(year);
        self
    }

    pub fn bpm(mut self, bpm: u32) -> Self {
        self.bpm = Some(bpm);
        self
    }

    pub fn key_signature(mut self, key: &str) -> Self {
        self.key_signature = Some(key.to_owned());
        self
    }

    pub fn isrc(mut self, isrc: &str) -> Self {
        self.isrc = Some(isrc.to_owned());
        self
    }

    pub async fn send(self) -> Result<Track> {
        let mut url = Url::parse(&format!("{}/tracks", self.client.host))?;
        url.query_pairs_mut()
            .append_pair("client_id", &self.client.client_id);

        let mut headers = reqwest::header::HeaderMap::new();
        if let Some(ref token) = self.client.auth_token {
            headers.insert(
                reqwest::header::AUTHORIZATION,
                format!("OAuth {}", token).parse()?,
            );
        }

        let mut form = reqwest::multipart::Form::new()
            .text("track[title]", self.title)
            .part(
                "track[asset_data]",
                reqwest::multipart::Part::bytes(self.asset_data),
            );

        if let Some(genre) = self.genre {
            form = form.text("track[genre]", genre);
        }
        if let Some(tags) = self.tags {
            form = form.text("track[tag_list]", tags);
        }
        if let Some(description) = self.description {
            form = form.text("track[description]", description);
        }
        if let Some(artwork) = self.artwork_data {
            let part = reqwest::multipart::Part::bytes(artwork)
                .file_name("artwork.jpg")
                .mime_str("image/jpeg")?;
            form = form.part("track[artwork_data]", part);
        }
        if let Some(sharing) = self.sharing {
            form = form.text("track[sharing]", sharing);
        }
        if let Some(downloadable) = self.downloadable {
            form = form.text("track[downloadable]", downloadable.to_string());
        }
        if let Some(streamable) = self.streamable {
            form = form.text("track[streamable]", streamable.to_string());
        }
        if let Some(license) = self.license {
            form = form.text("track[license]", license);
        }
        if let Some(url) = self.purchase_url {
            form = form.text("track[purchase_url]", url);
        }
        if let Some(title) = self.purchase_title {
            form = form.text("track[purchase_title]", title);
        }
        if let Some(day) = self.release_day {
            form = form.text("track[release_day]", day.to_string());
        }
        if let Some(month) = self.release_month {
            form = form.text("track[release_month]", month.to_string());
        }
        if let Some(year) = self.release_year {
            form = form.text("track[release_year]", year.to_string());
        }
        if let Some(bpm) = self.bpm {
            form = form.text("track[bpm]", bpm.to_string());
        }
        if let Some(key) = self.key_signature {
            form = form.text("track[key_signature]", key);
        }
        if let Some(isrc) = self.isrc {
            form = form.text("track[isrc]", isrc);
        }

        let response = self
            .client
            .http_client
            .post(url)
            .headers(headers)
            .multipart(form)
            .send()
            .await?;
        let track: Track = response.error_for_status()?.json().await?;
        Ok(track)
    }
}
