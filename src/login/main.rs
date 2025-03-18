use reqwest::blocking::{Client, Response};
use reqwest::IntoUrl;

pub fn main() {}

#[derive(Debug)]
pub enum Error {
    NetworkFailure,
}

fn get_with_ua<U: IntoUrl>(url: U) -> Result<Response, Error> {
    match Client::new()
        .get(url)
        .header("User-Agent","Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
        .send(){
            Ok(e)=>Ok(e),
            Err(_)=>Err(Error::NetworkFailure),
        }
}
