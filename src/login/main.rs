use crate::public::logger::{Logger, LoggerData};

use super::qrcode::{State, UrlConsoleQRCode};
use super::session::SessionClient;
use base64::Engine;
use crossterm::cursor::{MoveRight, MoveUp};
use crossterm::execute;
use dialoguer::theme::ColorfulTheme;
use dialoguer::Select;
use lazy_static::lazy_static;
use log::{info, trace, LevelFilter};
use rand::seq::IndexedRandom;
use regex::Regex;
use serde_json::Value;
use soft_aes::aes::aes_enc_cbc;
use std::collections::HashMap;
use std::io::{stdin, stdout, Write};
use std::sync::Arc;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

const AES_CHARS: &[u8] = b"ABCDEFGHJKMNPQRSTWXYZabcdefhijkmnprstwxyz2345678";

lazy_static! {
    static ref SESSION: Mutex<Option<String>> = Mutex::new(None);
}
lazy_static! {
    static ref REGEX_EXECUTION: Arc<Regex> = Arc::new(
        Regex::new("<input[^>]*?name=\"execution\"[^>]*?value=\"([^\"]*)\"[^>]*?>").unwrap()
    );
    static ref REGEX_PWD_SALT: Arc<Regex> = Arc::new(
        Regex::new("<input[^>]*?id=\"pwdEncryptSalt\"[^>]*?value=\"([^\"]*)\"[^>]*?>").unwrap()
    );
}
lazy_static! {
    static ref TEMPLATE_QR_LOGIN: HashMap<String, String> = [
        ("lt".to_string(), "".to_string()),
        ("uuid".to_string(), "".to_string()),
        ("cllt".to_string(), "qrLogin".to_string()),
        ("dllt".to_string(), "generalLogin".to_string()),
        ("execution".to_string(), "".to_string()),
        ("_eventId".to_string(), "submit".to_string()),
        ("rmShown".to_string(), "1".to_string())
    ]
    .into_iter()
    .collect();
    static ref TEMPLATE_PWD_LOGIN: HashMap<String, String> = [
        ("username".to_string(), "".to_string()),
        ("password".to_string(), "".to_string()),
        ("captcha".to_string(), "".to_string()),
        ("_eventId".to_string(), "submit".to_string()),
        ("cllt".to_string(), "userNameLogin".to_string()),
        ("dllt".to_string(), "generalLogin".to_string()),
        ("lt".to_string(), "".to_string()),
        ("execution".to_string(), "".to_string())
    ]
    .into_iter()
    .collect();
}

pub enum Error {
    Network,
    OpenQRCode,
    Service,
    ContentGet,
    ParseKey,
    Account,
    Input,
    Encrypt,
}

impl Logger for Error {
    fn get_logger(&self) -> LoggerData {
        match *self {
            Error::Network => LoggerData::new(LevelFilter::Error, "网络不通，请检查网络。"),
            Error::OpenQRCode => {
                LoggerData::new(LevelFilter::Error, "无法生成二维码，可能是网络问题")
            }
            Error::Service => LoggerData::new(LevelFilter::Warn, "学校服务遇到异常，请重试"),
            Error::ContentGet => LoggerData::new(LevelFilter::Warn, "无法获得页面内容，请重试"),
            Error::ParseKey => LoggerData::new(LevelFilter::Error, "获取页面Execution或Salt异常"),
            Error::Account => LoggerData::new(LevelFilter::Warn, "账号可能被风控，请使用扫码登录"),
            Error::Input => LoggerData::new(LevelFilter::Error, "输入异常"),
            Error::Encrypt => LoggerData::new(LevelFilter::Error, "密码加密失败"),
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        if e.is_request() {
            Error::Network
        } else {
            Error::ContentGet
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(_: std::io::Error) -> Self {
        Error::Input
    }
}

impl From<Box<dyn core::error::Error>> for Error {
    fn from(_: Box<dyn core::error::Error>) -> Self {
        Error::Encrypt
    }
}

pub fn main() {
    let by = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("选择登录方式")
        .default(0)
        .item("二维码登录")
        .item("密码登录")
        .interact()
        .unwrap_or(3);
    let target = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("选择登录目标")
        .default(0)
        .item("课程中心 https://lnt.xmu.edu.cn/")
        .item("教务系统 https://jw.xmu.edu.cn/")
        .interact()
        .unwrap_or(3);
    let target_url = match target {
        0 => "https://lnt.xmu.edu.cn/",
        1 => panic!("此功能未实现"), //"https://jw.xmu.edu.cn/login?service=https://jw.xmu.edu.cn/new/index.html",
        _ => "",
    };
    let ret = match by {
        0 => qr_login(target_url),
        1 => password_login(target_url),
        _ => Ok(()),
    };
    match ret {
        Ok(_) => {}
        Err(e) => e.logger(),
    }
    info!("获取到session = {:?}", get_session());
}

fn password_login(target: &str) -> Result<(), Error> {
    let mut session = SessionClient::new();

    let mut username = String::with_capacity(30);
    print!("请输入学号：");
    stdout().flush()?;
    stdin().read_line(&mut username)?;
    let username = username.trim();
    trace!("获取到 username = {:?}", username);

    let response = session.get(format!(
        "https://ids.xmu.edu.cn/authserver/checkNeedCaptcha.htl?username={}_={}",
        username,
        get_timestamp()
    ))?;
    let json: Value = response.json()?;
    let state = json
        .get("isNeed")
        .unwrap_or(&Value::Null)
        .as_bool()
        .unwrap_or(true);
    if state {
        return Err(Error::Account);
    }

    let mut password = String::with_capacity(100);
    print!("请输入密码：");
    stdout().flush()?;
    stdin().read_line(&mut password)?;
    let password = password.trim();
    trace!("获取到 password = {:?}", password);

    if crate::public::logger::LEVEL == log::LevelFilter::Trace {
        execute! {stdout(),MoveUp(1)}?;
    }
    execute! {stdout(),MoveUp(1),MoveRight(12)}?;
    for _ in 0..password.len() {
        print!("*");
    }
    println!();

    let service = get_service(&mut session, target)?;

    let response = session.get(format!(
        "https://ids.xmu.edu.cn/authserver/login?type=userNameLogin&service={}",
        service
    ))?;
    let text = response.text()?;

    let execution = get_execution(&text)?;
    let salt = get_salt(&text)?;

    let random_password = random_string(64) + password;
    let iv = random_string(16);
    trace!("random_password = {}", random_password);
    trace!("iv = {}", iv);
    trace!("service = {}", service);

    let random_password_u8 = random_password.as_bytes();
    let salt_u8 = salt.as_bytes();
    let iv_u8 = iv.as_bytes().try_into().unwrap_or(b"ABCDEFGHJKMNPQRS");
    let encrypted_password_u8 = aes_enc_cbc(random_password_u8, salt_u8, iv_u8, Some("PKCS7"))?;
    let encrypted_password =
        base64::engine::general_purpose::STANDARD.encode(encrypted_password_u8);

    info!("获取到 encrypted_password = {}", encrypted_password);

    let data = get_pwd_data(username, &encrypted_password, execution);

    let response = session.post(
        format!(
            "https://ids.xmu.edu.cn/authserver/login?type=service={}",
            service
        ),
        &data,
    )?;

    for e in response.cookies() {
        info!("获取到 cookie {}={}", e.name(), e.value());
        match e.name() {
            "session" => {
                let mut lock = SESSION.lock().unwrap();
                *lock = Some(e.value().to_string());
                return Ok(());
            }
            "asessionid" => {}
            _ => {}
        }
    }

    Err(Error::Account)
}

fn qr_login(target: &str) -> Result<(), Error> {
    let mut session = SessionClient::new();
    let service = get_service(&mut session, target)?;
    let login_page = session.get(format!(
        "https://ids.xmu.edu.cn/authserver/login?type=qrLogin&service={}",
        service
    ))?;
    let login_text = login_page.text()?;
    let execution = get_execution(&login_text)?;
    let mut qrcode = UrlConsoleQRCode::new(&get_qrcode_id(&mut session)?);
    qrcode.show()?;
    trace!("二维码的data = {:?}", qrcode.get_data());
    loop {
        match qrcode.get_state()? {
            Some(State::Waiting) => trace!("等待扫描二维码"),
            Some(State::Scanned) => trace!("扫描成功，等待确认"),
            Some(State::Success) => break,
            Some(State::Outdated) => {
                qrcode.renew(&get_qrcode_id(&mut session)?);
                qrcode.show()?;
            }
            None => trace!("请求太频繁"),
        }
    }
    let data = get_qrcode_data(qrcode.get_id(), execution);
    let response = session.post(
        format!(
            "https://ids.xmu.edu.cn/authserver/login?display=qrLogin&service={}",
            service
        ),
        &data,
    )?;
    for e in response.cookies() {
        info!("获取到 cookie {}={}", e.name(), e.value());
        match e.name() {
            "session" => {
                let mut lock = SESSION.lock().unwrap();
                *lock = Some(e.value().to_string());
                return Ok(());
            }
            "asessionid" => {}
            _ => {}
        }
    }

    Err(Error::Account)
}

fn random_string(len: usize) -> String {
    let mut rng = rand::rng();
    let mut result = String::new();
    for _ in 0..len {
        result.push(AES_CHARS.choose(&mut rng).unwrap().to_owned() as char)
    }
    result
}

pub fn get_session() -> Option<String> {
    SESSION.lock().unwrap().as_ref().map(|x| x.clone())
}

fn get_qrcode_data(qrcode_id: &str, execution: &str) -> HashMap<String, String> {
    let mut ret = TEMPLATE_QR_LOGIN.clone();
    ret.insert("uuid".to_string(), qrcode_id.to_string());
    ret.insert("execution".to_string(), execution.to_string());
    trace!("从模板 TEMPLATE_QR_LOGIN 新建 {:?}", &ret);
    ret
}

fn get_pwd_data(username: &str, salt_passwd: &str, execution: &str) -> HashMap<String, String> {
    let mut ret = TEMPLATE_PWD_LOGIN.clone();
    ret.insert("username".to_string(), username.to_string());
    ret.insert("password".to_string(), salt_passwd.to_string());
    ret.insert("execution".to_string(), execution.to_string());
    trace!("从模板 TEMPLATE_PWD_LOGIN 新建 {:?}", &ret);
    ret
}

fn get_qrcode_id(session: &mut SessionClient) -> Result<String, Error> {
    Ok(session
        .get(format!(
            "https://ids.xmu.edu.cn/authserver/qrCode/getToken?ts={}",
            get_timestamp()
        ))?
        .text()?)
}

pub fn get_timestamp() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

fn get_service(session: &mut SessionClient, target: &str) -> Result<String, Error> {
    let redirect_to_login_response = session.get(target)?;
    let redirect_to_login_query = match redirect_to_login_response.url().query() {
        Some(e) => e,
        None => return Err(Error::Service),
    };
    let service = &redirect_to_login_query[8..];
    trace!("获取到service = {}", service);
    Ok(service.to_string())
}

fn regex_get_first<'a>(re: &'a Regex, content: &'a str) -> Result<&'a str, Error> {
    let mut results: Vec<&str> = vec![];
    for (_, [s]) in re.captures_iter(content).map(|c| c.extract()) {
        results.push(s);
    }
    trace!("使用 regex = {}", re);
    trace!("匹配结果为 results = {:?}", results);
    match results.first() {
        Some(&e) => Ok(e),
        None => Err(Error::ParseKey),
    }
}

fn get_execution(content: &str) -> Result<&str, Error> {
    regex_get_first(&REGEX_EXECUTION, content)
}

fn get_salt(content: &str) -> Result<&str, Error> {
    regex_get_first(&REGEX_PWD_SALT, content)
}
