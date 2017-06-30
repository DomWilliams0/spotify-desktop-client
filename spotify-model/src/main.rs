#![allow(dead_code)]
#![recursion_limit = "1024"]

extern crate time;
extern crate reqwest;
extern crate url;
extern crate json;
extern crate fern;

#[macro_use]
extern crate error_chain;

#[macro_use]
extern crate log;

mod spotify;
mod http;
mod error;

use spotify::Spotify;
use error::*;

fn main() {
    if let Err(ref e) = run() {
        use std::io::Write;
        let stderr = &mut ::std::io::stderr();
        let errmsg = "Error writing to stderr";

        writeln!(stderr, "error: {}", e).expect(errmsg);

        for e in e.iter().skip(1) {
            writeln!(stderr, "caused by: {}", e).expect(errmsg);
        }

        if let Some(backtrace) = e.backtrace() {
            writeln!(stderr, "backtrace: {:?}", backtrace).expect(errmsg);
        }

        ::std::process::exit(1);
    }
}

fn run() -> SpotifyResult<()> {
    init_logging()?;

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

    let spot = Spotify::new(user, password);
    let items = spot.fetch_saved_tracks().chain_err(
        || "Failed to fetch saved tracks",
    )?;

    let list = &items.artists;
    info!("{} elements:", list.len());
    for t in list {
        info!("{:?}", t);
    }
    Ok(())
}

fn init_logging() -> SpotifyResult<()> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}][{}] {}",
                record.level(),
                record.target(),
                message
            ))
        })
        .level(log::LogLevelFilter::Error)
        .level_for("spotify_model", log::LogLevelFilter::Trace)
        .chain(std::io::stderr())
        .apply()
        .chain_err(|| "Failed to init logging")
}
