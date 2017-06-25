use reqwest::{RedirectPolicy, Client, Url};
use reqwest::header::{Authorization, Bearer};
use error::*;
use json::{parse, JsonValue};

use std::env;
use std::path::PathBuf;
use std::fs;
use std::io::Read;
use std::collections::HashSet;
use std::slice::Chunks;

use http::auth::{Auth, AuthState};
use http::request::*;

pub struct Spotify {
    auth: Auth,
}

pub struct SavedItems {
    pub tracks: Vec<Track>,
    pub albums: Vec<Album>,
    pub artists: Vec<Artist>,
}

impl Spotify {
    pub fn new(username: String, password: String) -> Self {
        let auth = Auth::new(username, password);

        Spotify { auth: auth }
    }

    pub fn fetch_saved_tracks(&mut self) -> SpotifyResult<SavedItems> {
        let mut album_ids = HashSet::<String>::new();
        let mut artist_ids = HashSet::new();

        let tracks = PageIterator::new(&mut self.auth, ApiEndpoint::SavedTracks)?
            .map(|mut o| {
                let mut track = o["track"].take();

                let album = track["album"]["id"].take_string().unwrap();
                let artists = collect_artist_ids(track["artists"].take());

                album_ids.insert(album.clone());
                artist_ids.extend(artists.clone());

                Track {
                    album_id: album,
                    artist_ids: artists,
                    disc: track["disc_number"].as_u8().unwrap(),
                    track_no: track["track_number"].as_u16().unwrap(),
                    duration_ms: track["duration_ms"].as_u32().unwrap(),
                    name: track["name"].take_string().unwrap(),
                }
            })
            .collect::<Vec<Track>>();

        let ids = album_ids.into_iter().collect::<Vec<String>>();
        let albums = SeveralIterator::new(&mut self.auth, ApiEndpoint::Albums, &ids)?
            .map(|mut o| {
                // TODO parse out of strings
                let release_date =
                    SpotifyDate::from(o["release_date"].take_string().unwrap(),
                                      o["release_date_precision"].take_string().unwrap());

                Album {
                    album_id: o["id"].take_string().unwrap(),
                    artist_ids: collect_artist_ids(o["artists"].take()),
                    images: collect_images(o["images"].take()),
                    release_date: release_date,
                    name: o["name"].take_string().unwrap(),
                }
            })
            .collect();

        let ids = artist_ids.into_iter().collect::<Vec<String>>();
        let artists = SeveralIterator::new(&mut self.auth, ApiEndpoint::Artists, &ids)?
            .map(|mut o| {
                //            artist_id: SpotifyId,
                //            images: Vec<Image>,
                //            genres: Vec<String>,
                //            name: String,

                let genres = o["genres"]
                    .members_mut()
                    .map(|g| g.take_string().unwrap())
                    .collect();

                // TODO take instead of mut in collect_..
                Artist {
                    artist_id: o["id"].take_string().unwrap(),
                    images: collect_images(o["images"].take()),
                    genres: genres,
                    name: o["name"].take_string().unwrap(),
                }
            })
            .collect();


        Ok(SavedItems {
               tracks: tracks,
               albums: albums,
               artists: artists,
           })
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

#[derive(Debug)]
pub struct Image {
    width: u32,
    height: u32,
    url: Url,
}

#[derive(Debug)]
pub struct SpotifyDate {
    date: String,
    precision: String,
}

#[derive(Debug)]
pub struct Album {
    album_id: SpotifyId,
    artist_ids: Vec<SpotifyId>,
    images: Vec<Image>,
    release_date: SpotifyDate,
    // TODO get genres from artist
    // genres: Vec<String>,
    name: String,
}

#[derive(Debug)]
pub struct Artist {
    artist_id: SpotifyId,
    images: Vec<Image>,
    genres: Vec<String>,
    name: String,
}

impl SpotifyDate {
    fn from(date: String, precision: String) -> Self {
        SpotifyDate {
            date: date,
            precision: precision,
        }
    }
}


fn collect_artist_ids(mut artists: JsonValue) -> Vec<SpotifyId> {
    artists
        .members_mut()
        .map(move |o| o["id"].take_string().unwrap())
        .collect::<Vec<SpotifyId>>()
}

fn collect_images(mut images: JsonValue) -> Vec<Image> {
    images
        .members_mut()
        .map(move |i| {
                 Image {
                     width: i["width"].as_u32().unwrap(),
                     height: i["height"].as_u32().unwrap(),
                     url: Url::parse(i["url"].as_str().unwrap()).unwrap(),
                 }
             })
        .collect()
}

pub fn config_dir() -> PathBuf {
    let mut p = PathBuf::from(env::var("XDG_CONFIG_HOME")
                                  .unwrap_or_else(|_| env::var("HOME").unwrap()));
    p.push("spotify_fun");
    fs::create_dir_all(&p);
    p
}
