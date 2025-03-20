use super::main::Error;
use reqwest::blocking::{Client, Response};
use reqwest::header::{HeaderMap, REFERER, USER_AGENT};
use reqwest::IntoUrl;

pub struct SessionClient {
    client: Client,
    headers: HeaderMap,
}

impl SessionClient {
    pub fn new() -> Self {
        let client = Client::builder().cookie_store(true).build().unwrap();
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT,"Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.3".parse().unwrap());
        Self { client, headers }
    }
    pub fn get<U: IntoUrl>(&mut self, url: U) -> Result<Response, Error> {
        let ret = self.client.get(url).headers(self.headers.clone()).send()?;
        self.headers
            .insert(REFERER, ret.url().as_str().parse().unwrap());
        Ok(ret)
    }
    pub fn post<U: IntoUrl, T: serde::ser::Serialize + ?core::marker::Sized>(
        &mut self,
        url: U,
        data: &T,
    ) -> Result<Response, Error> {
        let ret = self
            .client
            .post(url)
            .headers(self.headers.clone())
            .form(data)
            .send()?;
        self.headers
            .insert(REFERER, ret.url().as_str().parse().unwrap());
        Ok(ret)
    }
}
