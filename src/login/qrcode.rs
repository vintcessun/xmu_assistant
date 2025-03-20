use std::fs::remove_file;

use super::main::{get_timestamp, Error};
use crate::public::download_file::download_file;
use crate::public::DownloadFile;
use anyhow::Result;
use bardecoder;
use qrcode::QrCode;
use reqwest::blocking::Client;

const QRCODE_PATH: &str = "qrcode.png";

pub enum State {
    Waiting,
    Success,
    Scanned,
    Outdated,
}

pub struct UrlConsoleQRCode {
    qrcode_id: String,
    data: Option<String>,
}

impl UrlConsoleQRCode {
    pub fn new(qrcode_id: &str) -> Self {
        Self {
            qrcode_id: qrcode_id.to_string(),
            data: None,
        }
    }
    pub fn download_file(&mut self) -> Result<(), Error> {
        let url = format!(
            "https://ids.xmu.edu.cn/authserver/qrCode/getCode?uuid={}",
            self.qrcode_id
        );
        match download_file(&DownloadFile::new(&url, QRCODE_PATH)) {
            Ok(_) => {}
            Err(_) => return Err(Error::Network),
        };

        let img = match image::open(QRCODE_PATH) {
            Ok(e) => e,
            Err(_) => return Err(Error::OpenQRCode),
        };

        let decoder = bardecoder::default_decoder();
        let results = decoder.decode(&img);

        let first_ok = results.into_iter().flatten().next();

        self.data = Some(match first_ok {
            Some(e) => e,
            None => return Err(Error::OpenQRCode),
        });

        Ok(())
    }
    pub fn show(&mut self) -> Result<(), Error> {
        if self.data.is_none() {
            self.download_file()?;
        }
        let code = QrCode::new(self.data.as_ref().unwrap_or(&String::new()).as_bytes()).unwrap();
        let string = code
            .render::<char>()
            .dark_color('█')
            .light_color(' ')
            .quiet_zone(false)
            .module_dimensions(2, 1)
            .build();
        println!("{}", string);
        Ok(())
    }
    pub fn renew(&mut self, qrcode_id: &str) {
        self.qrcode_id = qrcode_id.to_string();
        self.data = None;
    }
    pub fn get_id(&self) -> &str {
        &self.qrcode_id
    }
    pub fn get_data(&self) -> Option<&str> {
        match &self.data {
            Some(e) => Some(e),
            None => None,
        }
    }
    pub fn get_state(&self) -> Result<Option<State>, Error> {
        if self.data.is_none() {
            return Ok(None);
        }
        let url = format!(
            "https://ids.xmu.edu.cn/authserver/qrCode/getStatus.htl?ts={}&uuid={}",
            get_timestamp(),
            self.qrcode_id,
        );

        let response = Client::new().get(&url).send()?;
        let state = response.text()?;
        match state.as_str() {
            "0" => Ok(Some(State::Waiting)),
            "1" => Ok(Some(State::Success)),
            "2" => Ok(Some(State::Scanned)),
            "3" => Ok(Some(State::Outdated)),
            _ => Err(Error::Service),
        }
    }
}

impl Drop for UrlConsoleQRCode {
    fn drop(&mut self) {
        if self.data.is_some() {
            remove_file(QRCODE_PATH).unwrap_or_default();
        }
    }
}
