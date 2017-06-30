use reqwest::Url;
use reqwest::header::{Authorization, Bearer};
use error::*;
use json::{parse, JsonValue};

use std::io::Read;
use std::slice::Chunks;

use http::auth::*;

#[derive(Debug, Copy, Clone)]
pub enum ApiEndpoint {
    SavedTracks,
    Albums,
    Artists,
}

fn get_uri_with_params(endpoint: ApiEndpoint, params: &[(&str, &str)]) -> SpotifyResult<Url> {
    Url::parse_with_params(get_uri(endpoint), params).chain_err(|| "Failed to parse uri")
}

fn get_uri(endpoint: ApiEndpoint) -> &'static str {
    match endpoint {
        ApiEndpoint::SavedTracks => "https://api.spotify.com/v1/me/tracks",
        ApiEndpoint::Albums => "https://api.spotify.com/v1/albums",
        ApiEndpoint::Artists => "https://api.spotify.com/v1/artists",
    }
}

pub fn send_api_request(auth: &Auth, url: Url) -> SpotifyResult<JsonValue> {
    // TODO avoid allocation with token
    debug!("Sending HTTP request to {:?}", url);
    let client = auth.client();
    let mut response = client
        .get(url)
        .header(Authorization(Bearer { token: auth.token(client)? }))
        .send()?;

    if !response.status().is_success() {
        bail!(ErrorKind::BadResponseStatusCode(*response.status()));
    }

    // TODO use etag header for caching
    // https://developer.spotify.com/web-api/user-guide/#conditional-requests

    let mut raw = String::new();
    response.read_to_string(&mut raw)?;
    Ok(parse(&raw).unwrap())
}

pub struct SeveralIterator<'a> {
    auth: &'a Auth,
    endpoint: ApiEndpoint,
    limit: usize,
    buffer: Vec<JsonValue>,
    in_vec: &'a [String],
    in_chunks: Chunks<'a, String>,
}

impl<'a> SeveralIterator<'a> {
    pub fn new(auth: &'a Auth, endpoint: ApiEndpoint, what: &'a [String]) -> SpotifyResult<Self> {
        let limit = SeveralIterator::get_limit(endpoint);
        let it = SeveralIterator {
            auth: auth,
            endpoint: endpoint,
            limit: limit,
            buffer: Vec::with_capacity(limit),
            in_vec: what,
            in_chunks: what.chunks(limit),
        };
        Ok(it)
    }

    fn fetch(&mut self) -> SpotifyResult<()> {
        // init chunks because it's apparently impossible to do in the constructor
        if let Some(ids) = self.in_chunks.next() {
            let url = {
                // repeated parameters not supported!
                let uri = get_uri(self.endpoint);
                let joined = ids.join(",");
                let prefix = "?ids=";
                let mut qs = String::with_capacity(uri.len() + prefix.len() + joined.len());
                qs.push_str(uri);
                qs.push_str(prefix);
                qs.push_str(&joined);
                Url::parse(&qs)?
            };
            let mut response = send_api_request(self.auth, url)?;
            if let JsonValue::Object(mut obj) = response.take() {
                let mut arr = obj.iter_mut()
                    .map(|(_k, mut v)| v.take())
                    .collect::<Vec<JsonValue>>()
                    .pop()
                    .unwrap();
                if let JsonValue::Array(ref mut vec) = arr.take() {
                    self.buffer.append(vec);
                }
            }
        }
        Ok(())
    }

    fn get_limit(endpoint: ApiEndpoint) -> usize {
        match endpoint {
            ApiEndpoint::Albums => 20,
            ApiEndpoint::Artists => 50,
            _ => 0,
        }
    }
}

impl<'a> Iterator for SeveralIterator<'a> {
    type Item = JsonValue;

    fn next(&mut self) -> Option<Self::Item> {
        self.buffer.pop().or_else(|| match self.fetch() {
            Err(e) => {
                warn!("Failed to get next in iterator: {:?}", e);
                None
            }
            _ => self.buffer.pop(),
        })
    }
}

pub struct PageIterator<'a> {
    auth: &'a Auth,
    endpoint: ApiEndpoint,
    limit: usize,
    total: u32,
    next: Option<Url>,
    buffer: Vec<JsonValue>,
}

impl<'a> PageIterator<'a> {
    pub fn new(auth: &'a Auth, endpoint: ApiEndpoint) -> SpotifyResult<Self> {
        const LIMIT: usize = 50;
        const LIMIT_STR: &str = "50"; // pff why not

        let mut it = PageIterator {
            auth: auth,
            endpoint: endpoint,
            limit: LIMIT,
            total: 0,
            next: Some({
                let params = [("limit", LIMIT_STR), ("offset", "0")];
                get_uri_with_params(endpoint, &params)?
            }),
            buffer: Vec::with_capacity(LIMIT),
        };

        it.fetch()?;

        Ok(it)
    }

    fn fetch(&mut self) -> SpotifyResult<()> {
        let url = match self.next.take() {
            Some(s) => s,
            None => return Ok(()), // end reached
        };

        let mut response = send_api_request(self.auth, url)?;

        self.buffer.clear();
        self.buffer.extend((response["items"]).members_mut().map(
            |o| o.take(),
        ));

        self.total = response["total"].as_u32().unwrap();
        self.next = match response["next"] {
            JsonValue::String(ref url) => Some(Url::parse(url)?),
            _ => None,
        };
        trace!(
            "Next href in pagination of {} items is {:?}",
            self.total,
            self.next
        );

        Ok(())
    }
}

impl<'a> Iterator for PageIterator<'a> {
    type Item = JsonValue;

    fn next(&mut self) -> Option<Self::Item> {
        self.buffer.pop().or_else(|| match self.fetch() {
            Err(e) => {
                warn!("Failed to get next in iterator: {:?}", e);
                None
            }
            _ => self.buffer.pop(),
        })
    }
}
