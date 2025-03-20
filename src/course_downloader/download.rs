use super::main::Error;
use anyhow::Result;
use reqwest::blocking::{Client, Response};
use reqwest::header::COOKIE;
use reqwest::IntoUrl;

pub fn get_with_cookie<U: IntoUrl>(url: U, cookie: &str) -> Result<Response, Error> {
    match Client::new()
        .get(url)
        .header(COOKIE, cookie)
        .header("User-Agent","Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
        .send(){
            Ok(e)=>Ok(e),
            Err(_)=>Err(Error::NetworkFailure),
        }
}
