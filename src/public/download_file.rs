use crate::public::thread_manage;
use anyhow::Result;
use curl::easy::Easy;
use lazy_static::lazy_static;
use log::{debug, info, trace, warn};
use std::collections::VecDeque;
use std::fs::File;
use std::io::Write;
use std::sync::{Arc, Condvar, Mutex};
use threadpool::ThreadPool;

lazy_static! {
    static ref download_queue: Arc<Mutex<VecDeque<DownloadFile>>> =
        Arc::new(Mutex::new(VecDeque::new()));
    static ref error_queue: Arc<Mutex<VecDeque<DownloadFile>>> =
        Arc::new(Mutex::new(VecDeque::new()));
    static ref condvar: Arc<Condvar> = Arc::new(Condvar::new());
    static ref pool: Arc<Mutex<ThreadPool>> =
        Arc::new(Mutex::new(ThreadPool::with_name("下载线程".to_string(), 4)));
}

#[derive(Default, Clone, Debug)]
pub struct DownloadFile {
    pub url: String,
    pub file: String,
}

impl DownloadFile {
    pub fn new(url: &str, file: &str) -> Self {
        Self {
            url: url.to_string(),
            file: file.to_string(),
        }
    }
    pub fn run(self) {
        debug!("放入任务队列 {:?}", &self);
        let mut lock = download_queue.lock().unwrap();
        lock.push_back(self);
        condvar.notify_one();
    }
}

pub fn download_file(task: &DownloadFile) -> Result<()> {
    let mut curl = Easy::new();
    let mut output = File::create(&task.file)?;
    curl.url(&task.url)?;
    curl.progress(true)?;
    curl.progress_function(
        |total_download_bytes, cur_download_bytes, _total_upload_bytes, _cur_upload_bytes| {
            if total_download_bytes > 0.0 {
                trace!("已下载:{}/{}", cur_download_bytes, total_download_bytes);
            }
            true
        },
    )?;
    curl.write_function(move |data: &[u8]| {
        output.write_all(data).unwrap();
        Ok(data.len())
    })?;
    curl.perform()?;
    debug!("完成 {:?}", &task);
    Ok(())
}

pub fn set_num_threads(num_threads: usize) {
    pool.lock().unwrap().set_num_threads(num_threads);
}

pub fn retry_error_tasks() {
    let mut errors = error_queue.lock().unwrap();
    let mut queue = download_queue.lock().unwrap();
    while let Some(item) = errors.pop_front() {
        warn!("移动到任务队列：{:?}", &item);
        queue.push_back(item);
    }
}

pub fn main() {
    thread_manage::execute("下载主线程", move || {
        let mutex_clone = Arc::clone(&download_queue);
        let condvar_clone = Arc::new(&condvar);
        let mut queue = mutex_clone.lock().unwrap();
        loop {
            match queue.pop_front() {
                Some(task) => {
                    info!("新建下载任务");
                    pool.lock().unwrap().execute(move || {
                        match download_file(&task) {
                            Ok(_) => {}
                            Err(_) => {
                                error_queue.lock().unwrap().push_back(task);
                            }
                        };
                    });
                }
                None => {
                    debug!("无任务");
                    queue = condvar_clone.wait(queue).unwrap();
                    debug!("被唤醒");
                }
            };
        }
    });
}
