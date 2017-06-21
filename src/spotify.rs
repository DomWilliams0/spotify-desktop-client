use reqwest::{RedirectPolicy, Client};
use auth::Auth;
use std::env;
use std::path::PathBuf;
use std::fs;

pub struct Spotify {
    client: Client,
    auth: Auth,
}

impl Spotify {
    pub fn new(username: String, password: String) -> Self {
        let client = {
            let mut c = Client::new().unwrap();
            c.redirect(RedirectPolicy::none());
            c
        };

        let mut auth = Auth::new(username, password);

        for _ in 0..3 {
            println!("{:?}", auth.token(&client));
        }

        Spotify {
            client: client,
            auth: auth,
        }
    }
}

pub fn config_dir() -> PathBuf {
    let mut p = PathBuf::from(env::var("XDG_CONFIG_HOME")
                                  .unwrap_or_else(|_| env::var("HOME").unwrap()));
    p.push("spotify_fun");
    fs::create_dir_all(&p);
    p
}
