pub mod download_file;
pub use download_file::DownloadFile;
pub mod logger;
pub mod thread_manage;

pub fn main() {
    logger::main();
    thread_manage::main();
    download_file::main();
}
