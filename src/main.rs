extern crate time;
extern crate reqwest;
extern crate url;

mod spotify;
mod auth;
mod error;

use spotify::Spotify;

fn main() {
    let mut spot = Spotify::new();

    spot.authenticate("dummy", "dummy");

}
