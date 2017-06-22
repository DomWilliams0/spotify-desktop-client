use reqwest::{RedirectPolicy, Client, Url};
use reqwest::header::{Authorization, Bearer};
use auth::Auth;
use error::*;
use json;

use std::env;
use std::path::PathBuf;
use std::fs;
use std::io::Read;

pub struct Spotify {
    client: Client,
    auth: Auth,
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

enum ApiEndpoint {
    SavedTracks,
}

fn get_uri(endpoint: &ApiEndpoint) -> &'static str {
    match *endpoint {
        ApiEndpoint::SavedTracks => "https://api.spotify.com/v1/me/tracks",
    }
}


impl Spotify {
    pub fn new(username: String, password: String) -> Self {
        let client = {
            let mut c = Client::new().unwrap();
            c.redirect(RedirectPolicy::none());
            c
        };

        let auth = Auth::new(username, password);

        Spotify {
            client: client,
            auth: auth,
        }
    }

    fn send_api_request(&mut self,
                        endpoint: &ApiEndpoint,
                        params: &[(&str, &str)])
                        -> SpotifyResult<json::JsonValue> {
        let uri = Url::parse_with_params(get_uri(endpoint), params)?;

        // TODO avoid allocation with token
        let mut response = self.client
            .get(uri)
            .header(Authorization(Bearer { token: self.auth.token(&self.client)?.to_owned() }))
            .send()?;

        if !response.status().is_success() {
            return Err(SpotifyError::BadResponseStatusCode(*response.status()));
        }

        // TODO use etag header for caching
        // https://developer.spotify.com/web-api/user-guide/#conditional-requests

        let mut raw = String::new();
        response.read_to_string(&mut raw)?;
        Ok(json::parse(&raw).unwrap())
    }

    pub fn fetch_saved_tracks(&mut self) -> SpotifyResult<Vec<Track>> {
        let params = [("limit", "50"), ("offset", "0")];
        let response = self.send_api_request(&ApiEndpoint::SavedTracks, &params)?;
        let tracks = (&response["items"])
            .members()
            .map(|o| {
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
}

pub fn config_dir() -> PathBuf {
    let mut p = PathBuf::from(env::var("XDG_CONFIG_HOME")
                                  .unwrap_or_else(|_| env::var("HOME").unwrap()));
    p.push("spotify_fun");
    fs::create_dir_all(&p);
    p
}
