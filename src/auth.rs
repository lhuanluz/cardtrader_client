use dotenv::dotenv;
use reqwest::header::{HeaderMap, HeaderValue};
use std::env;

pub fn get_auth_headers() -> HeaderMap {
    dotenv().ok();
    let mut headers = HeaderMap::new();
    let auth_token = env::var("CARD_TRADER_AUTH").expect("CARD_TRADER_AUTH must be set");
    let cookie = env::var("CARD_TRADER_COOKIE").expect("CARD_TRADER_COOKIE must be set");
    headers.insert("Authorization", HeaderValue::from_str(&auth_token).unwrap());
    headers.insert("Cookie", HeaderValue::from_str(&cookie).unwrap());
    headers
}
