extern crate time;
extern crate reqwest;
extern crate url;

mod spotify;

use spotify::Spotify;

fn main() {
    let mut spot = Spotify::new();

    spot.authenticate("dummy", "dummy");

}
