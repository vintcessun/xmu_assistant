use std::thread;

use lazy_static::lazy_static;
use log::debug;
use std::sync::{Arc, Mutex};

lazy_static! {
    static ref pool: Arc<Mutex<Vec<thread::JoinHandle<()>>>> = Arc::new(Mutex::new(Vec::new()));
}

pub fn main() {}

pub fn execute<F>(name: &str, f: F)
where
    F: FnOnce() + Send + 'static,
{
    debug!("创建线程\"{}\"", name);
    let mut lock = pool.lock().unwrap();
    let thread = thread::Builder::new()
        .name(name.to_string())
        .spawn(f)
        .unwrap();
    lock.push(thread);
}
