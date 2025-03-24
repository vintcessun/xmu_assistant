use super::download::get_with_cookie;
use anyhow::Result;
use dialoguer::theme::ColorfulTheme;
use dialoguer::Select;
use log::info;
use log::{debug, trace, LevelFilter};
use serde_json::Value;

use crate::login::main::get_session;
use crate::public::logger::Logger;
use crate::public::logger::LoggerData;
use crate::public::DownloadFile;
use crate::public::VOID_VEC;

const DOWNLOAD_PATH: &str = "./download/";
const PAGE_SIZE: usize = 5;
const END_PAGE: usize = PAGE_SIZE + 1;

#[derive(Debug)]
pub enum Error {
    LoginDataInvalid,
    NetworkFailure,
}

impl Logger for Error {
    fn get_logger(&self) -> LoggerData {
        match *self {
            Error::LoginDataInvalid => {
                LoggerData::new(LevelFilter::Warn, "账号已失效，请重新登录。")
            }
            Error::NetworkFailure => LoggerData::new(LevelFilter::Error, "网络不通，请检查网络。"),
        }
    }
}

pub fn main() {
    let cookie = get_session();
    let course_id = match get_course_id(&cookie) {
        Ok(v) => v,
        Err(e) => {
            e.logger();
            return;
        }
    };
    info!("获取到 course_id = {}", course_id);
    match get_file(&course_id, &cookie) {
        Ok(_) => {}
        Err(e) => e.logger(),
    }
}

fn get_course_id(cookie: &Option<String>) -> Result<String, Error> {
    let cookie = match cookie {
        Some(v) => format!("session={}", v),
        None => return Err(Error::LoginDataInvalid),
    };
    let mut page = 1;
    loop {
        let resp = get_with_cookie(format!("https://lnt.xmu.edu.cn/api/my-courses?&page={}&page_size={}&showScorePassedStatus=false",page,PAGE_SIZE), &cookie)?;
        let json: Value = match resp.json() {
            Ok(v) => v,
            Err(_) => return Err(Error::LoginDataInvalid),
        };
        trace!("课程列表 json = {}", &json);
        let mut choices = Vec::new();
        choices.push("上一页".to_string());
        let elements = json
            .get("courses")
            .unwrap_or(&Value::Null)
            .as_array()
            .unwrap_or(&*VOID_VEC);
        for element in elements {
            let name = element
                .get("name")
                .unwrap_or(&Value::Null)
                .as_str()
                .unwrap_or("");
            let instructor = element
                .get("instructors")
                .unwrap_or(&Value::Null)
                .as_array()
                .unwrap_or(&*VOID_VEC)
                .to_owned()
                .iter()
                .map(|x| x.get("name").unwrap_or(&Value::Null).as_str().unwrap_or(""))
                .collect::<Vec<_>>()
                .join(",");
            let semester = element
                .get("semester")
                .unwrap_or(&Value::Null)
                .get("name")
                .unwrap_or(&Value::Null)
                .as_str()
                .unwrap_or("");
            choices.push(format!("{} {} {}", name, instructor, semester));
        }
        choices.push("下一页".to_string());
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("请选择要下载的课程")
            .items(&choices)
            .interact()
            .unwrap_or(0);
        match selection {
            0 => {
                if page >= 1 {
                    page -= 1;
                }
            }
            1..=PAGE_SIZE => {
                return match elements[selection - 1].get("id") {
                    Some(v) => Ok(v.to_string()),
                    None => Err(Error::LoginDataInvalid),
                };
            }
            END_PAGE => page += 1,
            _ => {}
        }
    }
}

fn get_file(course_id: &str, cookie: &Option<String>) -> Result<(), Error> {
    let cookie = match cookie {
        Some(v) => format!("session={}", v),
        None => return Err(Error::LoginDataInvalid),
    };
    let resp = get_with_cookie(
        format!(
            "https://lnt.xmu.edu.cn/api/courses/{}/activities",
            course_id
        ),
        &cookie,
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
                &cookie,
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
