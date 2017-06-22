use std::fmt;
use std::error::Error;
use std::io;
use reqwest;

pub type SpotifyResult<T> = Result<T, SpotifyError>;

#[derive(Debug)]
pub enum SpotifyError {
    AuthMissingCookie(String),
    AuthBadCreds,
    AuthFailedAccept,
    BadTokenCache(&'static str),
    Io(io::Error),
    Reqwest(reqwest::Error),
    BadResponseStatusCode(reqwest::StatusCode),
    NotImplemented,
}

impl Error for SpotifyError {
    fn description(&self) -> &str {
        match *self {
            SpotifyError::AuthMissingCookie(_) => "expected cookie not present",
            SpotifyError::AuthBadCreds => "bad credentials",
            SpotifyError::AuthFailedAccept => "authorisation denied",
            SpotifyError::Io(_) => "io error",
            SpotifyError::BadTokenCache(_) => "invalid token cache",
            SpotifyError::Reqwest(ref e) => e.description(),
            SpotifyError::BadResponseStatusCode(_) => "bad response status code",
            SpotifyError::NotImplemented => "not implemented",
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            SpotifyError::Io(ref e) => e.cause(),
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
            SpotifyError::BadTokenCache(reason) => {
                write!(f, "The token cache file is invalid: {}", reason)
            }
            SpotifyError::Io(ref e) => e.fmt(f),
            SpotifyError::Reqwest(ref e) => e.fmt(f),
            SpotifyError::BadResponseStatusCode(ref code) => {
                write!(f, "Bad response status code: {:?}", code)
            }
            SpotifyError::NotImplemented => write!(f, "Not currently implemented"),
        }
    }
}

impl From<reqwest::Error> for SpotifyError {
    fn from(err: reqwest::Error) -> SpotifyError {
        SpotifyError::Reqwest(err)
    }
}

impl From<io::Error> for SpotifyError {
    fn from(err: io::Error) -> SpotifyError {
        SpotifyError::Io(err)
    }
}
