use reqwest::{RedirectPolicy, Client};
use auth::Auth;

pub struct Spotify {
    client: Client,
    auth: Option<Auth>,
}

impl Spotify {
    pub fn new() -> Self {
        Spotify {
            client: {
                let mut c = Client::new().unwrap();
                c.redirect(RedirectPolicy::none());
                c
            },
            auth: None,
        }
    }

    pub fn authenticate(&mut self, username: &str, password: &str) {
        self.auth = Auth::new(&self.client, username, password);
        //        println!("{:?}", self.auth);
    }

    fn authenticated(&self) -> bool {
        self.auth.as_ref().map_or(false, Auth::is_valid)
    }
}
