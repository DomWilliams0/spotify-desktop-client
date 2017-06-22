use reqwest::{RedirectPolicy, Client, Url};
use reqwest::header::{Authorization, Bearer};
use auth::Auth;
use error::*;
use json::{parse, JsonValue};

use std::env;
use std::path::PathBuf;
use std::fs;
use std::io::Read;

lazy_static! {
    static ref CLIENT: Client = {
            let mut c = Client::new().unwrap();
            c.redirect(RedirectPolicy::none());
            c
    };
}

pub struct Spotify {
    auth: Auth,
}

impl Spotify {
    pub fn new(username: String, password: String) -> Self {
        let auth = Auth::new(username, password);

        Spotify { auth: auth }
    }

    pub fn fetch_saved_tracks(&mut self) -> SpotifyResult<Vec<Track>> {
        let it = PageIterator::new(self, ApiEndpoint::SavedTracks)?;
        let tracks = it.map(|o| {
                let track = &o["track"];

                let album = &track["album"]["id"];
                let artists = (&track["artists"])
                    .members()
                    .map(|o| o["id"].as_str().unwrap().to_owned())
                    .collect::<Vec<SpotifyId>>();

                Track {
                    album_id: album.as_str().unwrap().to_owned(),
                    artist_ids: artists,
                    disc: track["disc_number"].as_u8().unwrap(),
                    track_no: track["track_number"].as_u16().unwrap(),
                    duration_ms: track["duration_ms"].as_u32().unwrap(),
                    name: track["name"].as_str().unwrap().to_owned(),
                }
            })
            .collect::<Vec<Track>>();

        Ok(tracks)
    }

    fn send_api_request(&mut self, url: Url) -> SpotifyResult<JsonValue> {
        // TODO avoid allocation with token
        let mut response = CLIENT
            .get(url)
            .header(Authorization(Bearer { token: self.auth.token(&CLIENT)?.to_owned() }))
            .send()?;

        if !response.status().is_success() {
            return Err(SpotifyError::BadResponseStatusCode(*response.status()));
        }

        // TODO use etag header for caching
        // https://developer.spotify.com/web-api/user-guide/#conditional-requests

        let mut raw = String::new();
        response.read_to_string(&mut raw)?;
        Ok(parse(&raw).unwrap())
    }
}

type SpotifyId = String;

#[derive(Debug)]
pub struct Track {
    album_id: SpotifyId,
    artist_ids: Vec<SpotifyId>,
    disc: u8,
    track_no: u16,
    duration_ms: u32,
    name: String,
}


#[derive(Debug, Copy, Clone)]
enum ApiEndpoint {
    SavedTracks,
}

fn get_uri(endpoint: ApiEndpoint) -> &'static str {
    match endpoint {
        ApiEndpoint::SavedTracks => "https://api.spotify.com/v1/me/tracks",
    }
}

pub fn config_dir() -> PathBuf {
    let mut p = PathBuf::from(env::var("XDG_CONFIG_HOME")
                                  .unwrap_or_else(|_| env::var("HOME").unwrap()));
    p.push("spotify_fun");
    fs::create_dir_all(&p);
    p
}

struct PageIterator<'a> {
    spotify: &'a mut Spotify,
    endpoint: ApiEndpoint,
    limit: usize,
    total: u32,
    next: Option<Url>,
    buffer: Vec<JsonValue>,
}

impl<'a> PageIterator<'a> {
    fn new(spotify: &'a mut Spotify, endpoint: ApiEndpoint) -> SpotifyResult<Self> {
        const LIMIT: usize = 50;
        const LIMIT_STR: &str = "50"; // pff why not

        let mut it = PageIterator {
            spotify: spotify,
            endpoint: endpoint,
            limit: LIMIT,
            total: 0,
            next: Some({
                           let params = [("limit", LIMIT_STR), ("offset", "0")];
                           Url::parse_with_params(get_uri(endpoint), &params)?
                       }),
            buffer: Vec::with_capacity(LIMIT),
        };

        it.fetch()?;

        Ok(it)
    }

    fn fetch(&mut self) -> SpotifyResult<()> {
        let url = match self.next.take() {
            Some(s) => s,
            None => return Ok(()), // end reached
        };

        let mut response = self.spotify.send_api_request(url)?;

        self.buffer.clear();
        self.buffer
            .extend((response["items"]).members_mut().map(|o| o.take()));

        self.total = response["total"].as_u32().unwrap();
        self.next = match response["next"] {
            JsonValue::String(ref url) => Some(Url::parse(url)?),
            _ => None,
        };
        trace!("Next href in pagination of {} items is {:?}",
               self.total,
               self.next);

        Ok(())
    }
}

impl<'a> Iterator for PageIterator<'a> {
    type Item = JsonValue;

    fn next(&mut self) -> Option<Self::Item> {
        self.buffer
            .pop()
            .or_else(|| match self.fetch() {
                         Err(e) => {
                             warn!("Failed to get next in iterator: {:?}", e);
                             None
                         }
                         _ => self.buffer.pop(),
                     })
    }
}
