pub mod download_file;
pub use download_file::DownloadFile;
pub mod logger;
pub mod thread_manage;

pub fn main() {
    logger::main();
    thread_manage::main();
    download_file::main();
}

use lazy_static::lazy_static;
use serde_json::Value;

lazy_static! {
    pub static ref VOID_VEC: Vec<Value> = Vec::new();
}
