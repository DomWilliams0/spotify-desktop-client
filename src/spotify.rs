use reqwest::Url;
use error::*;
use json::JsonValue;

use std::env;
use std::path::PathBuf;
use std::fs;
use std::collections::HashSet;

use http::auth::Auth;
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

    pub fn fetch_saved_tracks(&self) -> SpotifyResult<SavedItems> {
        let mut album_ids = HashSet::<String>::new();
        let mut artist_ids = HashSet::new();

        let tracks = PageIterator::new(&self.auth, ApiEndpoint::SavedTracks)?
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
        let albums = SeveralIterator::new(&self.auth, ApiEndpoint::Albums, &ids)?
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
        let artists = SeveralIterator::new(&self.auth, ApiEndpoint::Artists, &ids)?
            .map(|mut o| {
                let genres = o["genres"]
                    .members_mut()
                    .map(|g| g.take_string().unwrap())
                    .collect();

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

#[derive(Debug, PartialEq)]
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
    fs::create_dir_all(&p).ok();
    p
}

#[cfg(test)]
mod test {
    use json;
    use spotify::*;
    use reqwest::Url;

    #[test]
    fn artist_collection() {
        assert_eq!(collect_artist_ids(json::parse(ARTISTS_JSON).unwrap()),
                   vec!["0oSGxfWSnnOXhD2fKuz2Gy", "3dBVyJ7JuOMt4GE9607Qin"]);
        assert_eq!(collect_artist_ids(json::parse("[]").unwrap()),
                   Vec::<String>::new());
        assert_eq!(collect_artist_ids(json::parse("null").unwrap()),
                   Vec::<String>::new());
    }

    #[test]
    fn image_collection() {
        let expected =
            vec![Image {
                     width: 1000,
                     height: 1000,
                     url: Url::parse("https://i.scdn.co/image/32bd9707b42a2c081482ec9cd3ffa8879f659f95",)
                         .unwrap(),
                 },
                 Image {
                     width: 640,
                     height: 640,
                     url: Url::parse("https://i.scdn.co/image/865f24753e5e4f40a383bf24a9cdda598a4559a8",)
                         .unwrap(),
                 }];
        assert_eq!(collect_images(json::parse(IMAGES_JSON).unwrap()), expected);
        assert_eq!(collect_images(json::parse("[]").unwrap()),
                   Vec::<Image>::new());
        assert_eq!(collect_images(json::parse("null").unwrap()),
                   Vec::<Image>::new());
    }

    // ugly constants

    const ARTISTS_JSON: &'static str = r#"
[ {
    "external_urls" : {
      "spotify" : "https://open.spotify.com/artist/0oSGxfWSnnOXhD2fKuz2Gy"
    },
    "followers" : {
      "href" : null,
      "total" : 633494
    },
    "genres" : [ "art rock", "glam rock", "permanent wave" ],
    "href" : "https://api.spotify.com/v1/artists/0oSGxfWSnnOXhD2fKuz2Gy",
    "id" : "0oSGxfWSnnOXhD2fKuz2Gy",
    "images" : [ {
      "height" : 1000,
      "url" : "https://i.scdn.co/image/32bd9707b42a2c081482ec9cd3ffa8879f659f95",
      "width" : 1000
    }, {
      "height" : 640,
      "url" : "https://i.scdn.co/image/865f24753e5e4f40a383bf24a9cdda598a4559a8",
      "width" : 640
    }, {
      "height" : 200,
      "url" : "https://i.scdn.co/image/7ddd6fa5cf78aee2f2e8b347616151393022b7d9",
      "width" : 200
    }, {
      "height" : 64,
      "url" : "https://i.scdn.co/image/c8dc28c191432862afce298216458a6f00bbfbd8",
      "width" : 64
    } ],
    "name" : "David Bowie",
    "popularity" : 77,
    "type" : "artist",
    "uri" : "spotify:artist:0oSGxfWSnnOXhD2fKuz2Gy"
  }, {
    "external_urls" : {
      "spotify" : "https://open.spotify.com/artist/3dBVyJ7JuOMt4GE9607Qin"
    },
    "followers" : {
      "href" : null,
      "total" : 52338
    },
    "genres" : [ "glam rock", "protopunk" ],
    "href" : "https://api.spotify.com/v1/artists/3dBVyJ7JuOMt4GE9607Qin",
    "id" : "3dBVyJ7JuOMt4GE9607Qin",
    "images" : [ {
      "height" : 1300,
      "url" : "https://i.scdn.co/image/5515a710c94ccd4edd8b9a0587778ed5e3f997da",
      "width" : 1000
    }, {
      "height" : 832,
      "url" : "https://i.scdn.co/image/c990e667b4ca8240c73b0db06e6d76a3b27ce929",
      "width" : 640
    }, {
      "height" : 260,
      "url" : "https://i.scdn.co/image/de2fa1d11c59e63143117d44ec9990b9e40451a2",
      "width" : 200
    }, {
      "height" : 83,
      "url" : "https://i.scdn.co/image/b39638735adb4a4a54621293b99ab65c546f605e",
      "width" : 64
    } ],
    "name" : "T. Rex",
    "popularity" : 58,
    "type" : "artist",
    "uri" : "spotify:artist:3dBVyJ7JuOMt4GE9607Qin"
  } ]
        "#;

    const IMAGES_JSON: &'static str = r#"
    [ {
      "height" : 1000,
      "url" : "https://i.scdn.co/image/32bd9707b42a2c081482ec9cd3ffa8879f659f95",
      "width" : 1000
    }, {
      "height" : 640,
      "url" : "https://i.scdn.co/image/865f24753e5e4f40a383bf24a9cdda598a4559a8",
      "width" : 640
    } ]
        "#;
}
