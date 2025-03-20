use super::download::get_with_cookie;
use anyhow::Result;
use log::{debug, trace};
use log::{error, warn};
use serde_json::Value;

use crate::public::DownloadFile;
use crate::public::VOID_VEC;

const DOWNLOAD_PATH: &str = "./download/";

#[derive(Debug)]
pub enum Error {
    LoginDataInvalid,
    NetworkFailure,
}

pub fn main() {
    let cookie = "session=V2-1-c4a98c17-fefc-490b-9f34-416ada2efa1a.MTg5NTQ0.1742373752122.V0HGDMwE6v6M8LaCssXSV_qxvkc";
    let course_id = "53248";
    match get_file(course_id, cookie) {
        Ok(_) => {}
        Err(e) => match e {
            Error::LoginDataInvalid => warn!("账号已失效，请重新登录。"),
            Error::NetworkFailure => error!("网络不通，请检查网络。"),
        },
    }
}

pub fn get_file(course_id: &str, cookie: &str) -> Result<(), Error> {
    let resp = get_with_cookie(
        format!(
            "https://lnt.xmu.edu.cn/api/courses/{}/activities",
            course_id
        ),
        cookie,
    )?;
    let json: Value = match resp.json() {
        Ok(v) => v,
        Err(_) => return Err(Error::LoginDataInvalid),
    };
    trace!("课程json = {}", json);
    let elements = json
        .get("activities")
        .unwrap_or(&Value::Null)
        .as_array()
        .unwrap_or(&*VOID_VEC);
    for element in elements {
        let uploads = element
            .get("uploads")
            .unwrap_or(&Value::Null)
            .as_array()
            .unwrap_or(&*VOID_VEC);
        for file in uploads {
            trace!("获取到一个 upload file = {}", file);
            let reference_id = file.get("reference_id").unwrap_or(&Value::Null).to_string();
            debug!("获取到 reference_id = {}", reference_id);
            let name = file
                .get("name")
                .unwrap_or(&Value::Null)
                .as_str()
                .unwrap_or("");
            debug!("获取到 name = {}", name);
            let resp = get_with_cookie(
                format!("https://lnt.xmu.edu.cn/api/uploads/reference/{reference_id}/url"),
                cookie,
            )?;
            let json: Value = resp.json().unwrap_or(Value::Null);
            trace!("获取到 json = {}", json);
            let url = json
                .get("url")
                .unwrap_or(&Value::Null)
                .as_str()
                .unwrap_or("");
            let file = format!("{}/{}", DOWNLOAD_PATH, name);
            let d = DownloadFile::new(url, &file);
            d.run();
        }
    }
    Ok(())
}
