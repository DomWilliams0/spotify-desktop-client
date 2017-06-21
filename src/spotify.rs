use reqwest::{RedirectPolicy, Client};
use auth::Auth;

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

        Spotify {
            client: client,
            auth: Auth::new(username, password),
        }
    }
}
