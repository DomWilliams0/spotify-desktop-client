use std::io;
use reqwest;
use url;
use log;

error_chain! {

    types {
        Error, ErrorKind, ResultExt, SpotifyResult;
    }

    foreign_links {
        Io(io::Error);
        Reqwest(reqwest::Error);
        Url(url::ParseError);
        Logger(log::SetLoggerError);
    }

    errors {
        AuthMissingCookie(cookie: String) {
            display("Expected cookie '{}' was not present", cookie)
        }

        AuthBadCreds {
            display("bad credentials")
        }

        AuthFailedAccept {
            display("spotify rejected the authorisation request")
        }

        BadTokenCache(reason: &'static str) {
            display("token cache invalid: {}", reason)
        }

        BadResponseStatusCode(code: reqwest::StatusCode) {
            display("unexpected response status code ({:?})", code)
        }

        NotImplemented {
            display("not implemented")
        }
    }
}
