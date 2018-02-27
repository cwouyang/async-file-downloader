use file::FileInfo;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use num_cpus;
use reqwest::{Client, Response};
use serde_json::Value;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::result;
use std::time::{Duration, Instant};
use threadpool::ThreadPool;
use url::Url;

/// Constant to represent the length of the buffer to download the remote content
const DEFAULT_DOWNLOAD_BUFFER_BYTES: usize = 1024 * 64;
/// Constant to represent the refresh interval (in milliseconds) for the CLI
const PROGRESS_UPDATE_INTERVAL_MILLIS: u64 = 1000;

#[derive(Debug)]
pub enum Error {
    InvalidUrl,
    DownloadFail,
    InvalidResponse,
    IoError,
}

pub type Result<T> = result::Result<T, Error>;

pub fn download_file_list(json_url: &str) -> Result<Vec<FileInfo>> {
    let client = Client::new();
    let response = match client.get(json_url).send() {
        Ok(response) => response,
        Err(_) => return Err(Error::DownloadFail)
    };
    return create_file_list(response);
}

fn create_file_list(mut response: Response) -> Result<Vec<FileInfo>> {
    let json_vec = match response.json() {
        Ok(Value::Array(list)) => list,
        Ok(_) => unreachable!(),
        Err(_) => return Err(Error::InvalidResponse)
    };

    let mut file_list: Vec<FileInfo> = Vec::new();
    for (_, file_info) in json_vec.iter().enumerate() {
        let url = match file_info["url"] {
            Value::String(ref url) => url,
            _ => unreachable!()
        };
        let cloned_url = url.clone();

        let parsed_url = match Url::parse(&url) {
            Ok(url) => url,
            Err(_) => continue;
        };
        let paths = parsed_url.path_segments();
        let file_name = paths.unwrap().last().unwrap().to_owned();

        let file_size = match file_info["size"] {
            Value::Number(ref size) => size,
            _ => unreachable!()
        }.as_u64().unwrap();
        file_list.push(FileInfo { url: cloned_url, name: file_name, size: file_size });
    }
    return Ok(file_list);
}

pub fn download_files(files: Vec<FileInfo>) {
    let worker_count = num_cpus::get_physical();
    let pool = ThreadPool::new(worker_count);
    let mp = MultiProgress::new();

    println!("Start downloading files with {} threads", worker_count);
    for (_, file) in files.iter().enumerate() {
        let mut bar = ProgressBar::new(file.size);
        let style = ProgressStyle::default_bar()
            .template(&format!("{} [{{elapsed_precise}}] {{bar:.{}}} {{bytes:>8}}/{{total_bytes:>8}} eta:{{eta:>4}} {{msg}}", file.name, "yello"))
            .progress_chars("=> ");
        bar.set_style(style);
        bar = mp.add(bar);

        let cloned_file = file.clone();
        pool.execute(move || {
            match download_file(cloned_file, &bar) {
                Ok(_) => {
                    // Download complete, calculate MD5
                }
                Err(e) => {
                    println!("Download file failed: {:?}", e);
                }
            }
        });
    }
    mp.join();
    pool.join();
}

pub fn download_file(file: FileInfo, bar: &ProgressBar) -> Result<()> {
    let mut file_writer = create_file_with_size(&file.name, file.size)?;

    let mut bytes_buffer = [0; DEFAULT_DOWNLOAD_BUFFER_BYTES];
    let mut bytes_received = 0;
    let mut first_byte_received = false;
    let progress_update_interval_millis: Duration = Duration::from_millis(PROGRESS_UPDATE_INTERVAL_MILLIS);
    let mut last_progress_time = Instant::now() - progress_update_interval_millis;

    let client = Client::new();
    let mut response = client.get(&file.url).send().unwrap();
    bar.set_message("Downloading...");
    while let Ok(n) = response.read(&mut bytes_buffer) {
        if n == 0 {
            if !first_byte_received {
                bar.finish_with_message(&"Download failed".to_owned());
                return Err(Error::DownloadFail);
            }
            break;
        }
        first_byte_received = true;
        if file_writer.write(&bytes_buffer[0..n]).is_err() {
            bar.finish_with_message(&"Writing file failed".to_owned());
            return Err(Error::IoError);
        }
        bytes_received = bytes_received + n as u64;

        let now = Instant::now();
        if now.duration_since(last_progress_time) > progress_update_interval_millis {
            last_progress_time = now;
            bar.set_position(bytes_received);
        }
        bar.inc(n as u64);
    }
    bar.finish_with_message(&"Downloaded".to_owned());
    Ok(())
}

fn create_file_with_size(file_path: &str, size: u64) -> Result<File> {
    let path = Path::new(file_path);
    if path.exists() {
//        println!("The path to store {} already exists! Overwrite it.", file_path);
    }
    match File::create(path) {
        Ok(file) => {
            file.set_len(size).expect("Cannot extend file to download size!");
            Ok(file)
        }
        Err(_) => Err(Error::IoError)
    }
}