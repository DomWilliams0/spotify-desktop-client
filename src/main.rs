#![allow(dead_code)]

extern crate time;
extern crate reqwest;
extern crate url;
extern crate json;
extern crate fern;

#[macro_use]
extern crate log;

#[macro_use]
extern crate lazy_static;


mod spotify;
mod auth;
mod error;

use spotify::Spotify;

fn main() {
    if let Err(e) = init_logging() {
        println!("Failed to init logging: {:?}", e);
        std::process::exit(1);
    }

    // temporarily read creds from environment
    const CRED_ENV: &str = "SPOTIFY_CREDS";
    let (user, password) = match std::env::var(CRED_ENV) {
        Err(e) => {
            error!("No creds provided in env var {} ({})", CRED_ENV, e);
            std::process::exit(2);
        }
        Ok(ref creds) => {
            match creds.find(':') {
                None => {
                    error!("Creds must be in format user:password");
                    std::process::exit(3);
                }
                Some(index) => {
                    let user = (&creds[..index]).to_owned();
                    let pass = (&creds[index + 1..]).to_owned();
                    (user, pass)
                }
            }
        }
    };

    let mut spot = Spotify::new(user, password);
    let items = spot.fetch_saved_tracks()
        .expect("Failed to test track fetching");

    let list = &items.albums;
    info!("{} elements:", list.len());
    for t in list {
        info!("{:?}", t);
    }
}

fn init_logging() -> Result<(), log::SetLoggerError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
                    out.finish(format_args!("[{}][{}] {}",
                                            record.level(),
                                            record.target(),
                                            message))
                })
        .level(log::LogLevelFilter::Error)
        .level_for("rust_model", log::LogLevelFilter::Trace)
        .chain(std::io::stderr())
        .apply()
}
