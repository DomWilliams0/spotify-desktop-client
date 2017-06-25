use std::fmt;
use std::error::Error;
use std::io;
use toml;
use reqwest;

pub type SpotifyResult<T> = Result<T, SpotifyError>;

#[derive(Debug)]
pub enum SpotifyError {
    AuthMissingCookie(String),
    AuthBadCreds,
    AuthFailedAccept,
    Io(io::Error),
    Serialisation(toml::ser::Error),
    Deserialisation(toml::de::Error),
    Reqwest(reqwest::Error),
    NotImplemented,
}

impl Error for SpotifyError {
    fn description(&self) -> &str {
        match *self {
            SpotifyError::AuthMissingCookie(_) => "expected cookie not present",
            SpotifyError::AuthBadCreds => "bad credentials",
            SpotifyError::AuthFailedAccept => "authorisation denied",
            SpotifyError::Io(_) => "io error",
            SpotifyError::Serialisation(_) => "serialisation error",
            SpotifyError::Deserialisation(_) => "deserialisation error",
            SpotifyError::Reqwest(ref e) => e.description(),
            SpotifyError::NotImplemented => "not implemented",
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            SpotifyError::Io(ref e) => e.cause(),
            SpotifyError::Serialisation(ref e) => e.cause(),
            SpotifyError::Deserialisation(ref e) => e.cause(),
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
            SpotifyError::Io(ref e) => e.fmt(f),
            SpotifyError::Serialisation(ref e) => e.fmt(f),
            SpotifyError::Deserialisation(ref e) => e.fmt(f),
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

impl From<io::Error> for SpotifyError {
    fn from(err: io::Error) -> SpotifyError {
        SpotifyError::Io(err)
    }
}

impl From<toml::ser::Error> for SpotifyError {
    fn from(err: toml::ser::Error) -> SpotifyError {
        SpotifyError::Serialisation(err)
    }
}

impl From<toml::de::Error> for SpotifyError {
    fn from(err: toml::de::Error) -> SpotifyError {
        SpotifyError::Deserialisation(err)
    }
}
