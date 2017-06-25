extern crate time;
extern crate reqwest;
extern crate url;

mod spotify;
mod auth;
mod error;

use spotify::Spotify;

fn main() {
    // temporarily read creds from environment
    const CRED_ENV: &str = "SPOTIFY_CREDS";
    let (user, password) = match std::env::var(CRED_ENV) {
        Err(e) => {
            println!("No creds provided in env var {} ({})", CRED_ENV, e);
            std::process::exit(1);
        }
        Ok(ref creds) => {
            match creds.find(':') {
                None => {
                    println!("Creds must be in format user:password");
                    std::process::exit(2);
                }
                Some(index) => {
                    let user = (&creds[..index]).to_owned();
                    let pass = (&creds[index + 1..]).to_owned();
                    (user, pass)
                }
            }
        }
    };

    let spot = Spotify::new(user, password);
}
