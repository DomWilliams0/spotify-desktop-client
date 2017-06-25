use time;

use url;
use error::*;
use reqwest::*;
use reqwest::header::*;

#[derive(Debug)]
pub struct Auth {
    token: String,
    expiry_time: time::Tm,
}

const CSRF: &str = "csrf_token";

fn extract_from_flattened_list<'a>(src: &'a str, key: &str, sep: char) -> Option<&'a str> {
    if let Some(start) = src.find(key) {

        let end = src[start..].find(sep).unwrap_or_else(|| src.len());
        return Some(&src[start + key.len() + 1..end]); // +1 for =
    }

    None
}

fn extract_cookie_value<'a>(response: &'a Response, key: &str) -> SpotifyResult<&'a str> {
    //    let &SetCookie(ref values) = response.headers().get::<SetCookie>()
    response
        .headers()
        .get::<SetCookie>()
        .and_then(|&SetCookie(ref values)| {
                      values
                          .iter()
                          .find(|&x| x.starts_with(key))
                          .and_then(|c| extract_from_flattened_list(c, key, ';'))
                  })
        .ok_or_else(|| SpotifyError::AuthMissingCookie(key.to_owned()))

}

fn create_cookie(pairs: &Vec<(&str, &str)>) -> Cookie {
    Cookie(pairs.iter().map(|&(k, v)| format!("{}={}", k, v)).collect())
}

impl Auth {
    pub fn new(client: &Client, username: &str, password: &str) -> SpotifyResult<Auth> {
        const AUTHORIZE: &str = "https://accounts.spotify.com/authorize?";
        const LOGIN: &str = "https://accounts.spotify.com/api/login";
        const ACCEPT: &str = "https://accounts.spotify.com/en/authorize/accept";
        const SPOTIFY_CLIENT_ID: &str = "a4a869822602493c828f424d7552379c";
        const REDIRECT_URI: &str = "http://localhost";
        const SCOPE: &str = "user-library-read";

        // initial authorise attempt
        let query_params = vec![("client_id", String::from(SPOTIFY_CLIENT_ID)),
                                ("response_type", String::from("token")),
                                ("redirect_uri", String::from(REDIRECT_URI)),
                                ("scope", String::from(SCOPE)),
                                ("show_dialog", String::from("true"))];

        let mut headers = {
            let mut h = Headers::new();
            h.set(UserAgent(
                "Mozilla/5.0 (X11; Linux x86_64; rv:54.0) Gecko/20100101 Firefox/54.0".to_owned()
            ));
            h.set(Connection::keep_alive());
            h
        };

        let original_url = url::form_urlencoded::Serializer::new(String::from(AUTHORIZE))
            .extend_pairs(&query_params)
            .finish();
        println!("Sending GET to /authorize");
        let resp = client
            .get(original_url.as_str())
            .headers(headers.clone())
            .send()?;

        let csrf = extract_cookie_value(&resp, CSRF)?;

        let login_data = vec![("remember", "false"),
                              ("username", username),
                              ("password", password),
                              ("csrf_token", csrf)];
        let login_cookies = create_cookie(&vec![(CSRF, csrf),
                                                ("__bon",
                                                 "MHwwfDYyODMzMzc0OHwyNjM5MDAxNzQxNnwxfDF8MXww"),
                                                ("fb_continue", &original_url),
                                                ("remember", username)]);

        headers.set(Referer(original_url.clone()));
        headers.set(login_cookies);

        println!("Sending POST to /login");
        let resp = client
            .post(LOGIN)
            .headers(headers.clone())
            .form(&login_data)
            .send()?;

        if !resp.status().is_success() {
            return Err(SpotifyError::AuthBadCreds);
        }

        println!("Authenticated!");

        let csrf = extract_cookie_value(&resp, CSRF)?;
        let accept_data = {
            let mut pairs = query_params;
            pairs.push((CSRF, csrf.to_owned()));
            pairs
        };
        let accept_cookies = create_cookie(&vec![("sp_ac", extract_cookie_value(&resp, "sp_ac")?),
                                                 ("sp_dc", extract_cookie_value(&resp, "sp_dc")?),
                                                 (CSRF, extract_cookie_value(&resp, CSRF)?)]);
        headers.remove::<Cookie>();
        headers.set(accept_cookies);

        println!("Sending POST to /accept");
        let resp = client
            .post(ACCEPT)
            .headers(headers)
            .form(&accept_data)
            .send()?;

        resp.headers()
            .get::<Location>()
            .and_then(|&Location(ref loc)| {
                let e = extract_from_flattened_list(loc, "expires_in", '&');
                let t = extract_from_flattened_list(loc, "access_token", '&');

                match (e, t) {
                    (Some(e), Some(t)) => {
                        Some(Auth {
                                 token: t.to_owned(),
                                 expiry_time: time::now() +
                                              time::Duration::seconds(e.parse().unwrap()),
                             })
                    }
                    _ => None,
                }
            })
            .ok_or(SpotifyError::AuthFailedAccept)
    }

    pub fn is_valid(&self) -> bool {
        self.expiry_time > time::now()
    }
}
