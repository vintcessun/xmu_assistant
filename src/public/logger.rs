use ansi_term::Colour;
use chrono::Local;
use env_logger::Builder;
use log::{Level, LevelFilter};
use std::io::Write;
use std::thread;

pub const LEVEL: LevelFilter = LevelFilter::Info;

pub fn main() {
    let main_id = thread::current().id();
    Builder::new()
        .format(move |buf, record| {
            let current = thread::current();
            let thread_name = if current.id() != main_id {
                current.name().unwrap_or("未知线程")
            } else {
                "主线程"
            };
            writeln!(
                buf,
                "[{}][{}][{}][{}][{}] {}",
                Local::now().format("%Y-%m-%dT%H:%M:%S"),
                thread_name,
                record.file().unwrap_or("未知文件"),
                record.module_path().unwrap_or("未知模块"),
                match record.level() {
                    Level::Error => Colour::Red.paint("ERROR"),
                    Level::Warn => Colour::Yellow.paint("WARNING"),
                    Level::Info => Colour::Blue.paint("INFO"),
                    Level::Debug => Colour::Purple.paint("DEBUG"),
                    Level::Trace => Colour::Cyan.paint("TRACE"),
                },
                record.args(),
            )
        })
        .filter(None, LEVEL)
        .init();
}
