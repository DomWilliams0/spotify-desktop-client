use std::fmt;
use std::error::Error;
use reqwest;

pub type SpotifyResult<T> = Result<T, SpotifyError>;

#[derive(Debug)]
pub enum SpotifyError {
    AuthMissingCookie(String),
    AuthBadCreds,
    AuthFailedAccept,
    Reqwest(reqwest::Error),
    NotImplemented,
}

impl Error for SpotifyError {
    fn description(&self) -> &str {
        match *self {
            SpotifyError::AuthMissingCookie(_) => "expected cookie not present",
            SpotifyError::AuthBadCreds => "bad credentials",
            SpotifyError::AuthFailedAccept => "authorisation denied",
            SpotifyError::Reqwest(ref e) => e.description(),
            SpotifyError::NotImplemented => "not implemented",
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            SpotifyError::Reqwest(ref e) => e.cause(),
            _ => None,
        }
    }
}

impl fmt::Display for SpotifyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SpotifyError::AuthMissingCookie(ref c) => {
                write!(f, "Expected cookie '{}' was not present", c)
            }
            SpotifyError::AuthBadCreds => write!(f, "Bad credentials"),
            SpotifyError::AuthFailedAccept => {
                write!(f, "Spotity rejected the authorisation request")
            }
            SpotifyError::Reqwest(ref e) => e.fmt(f),
            SpotifyError::NotImplemented => write!(f, "Not currently implemented"),
        }
    }
}

impl From<reqwest::Error> for SpotifyError {
    fn from(err: reqwest::Error) -> SpotifyError {
        SpotifyError::Reqwest(err)
    }
}
