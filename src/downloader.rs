use file::FileInfo;
use num_cpus;
use reqwest::{Client, Response};
use serde_json::Value;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::result;
use threadpool::ThreadPool;
use url::Url;

/// Constant to represent the length of the buffer to download the remote content
const DEFAULT_DOWNLOAD_BUFFER_BYTES: usize = 1024 * 64;

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
        let size = match file_info["size"] {
            Value::Number(ref size) => size,
            _ => unreachable!()
        }.as_u64().unwrap();
        file_list.push(FileInfo { url: cloned_url, size });
    }
    return Ok(file_list);
}

pub fn download_files(files: Vec<FileInfo>) {
    let worker_count = num_cpus::get_physical();
    let pool = ThreadPool::new(worker_count);
//    let (tx, rx) = channel();

    println!("Start downloading files with {} threads", worker_count);
    for (_, file) in files.iter().enumerate() {
        let url = file.url.clone();
        let size = file.size;
        pool.execute(move || {
            match download_file(url, size) {
                Ok(_) => {
                    // Download complete, calculate MD5
                }
                Err(e) => {
                    println!("Download file failed: {:?}", e);
                }
            }
        });
    }
    pool.join();
}

pub fn download_file(url: String, size: u64) -> Result<()> {
    let parsed_url = match Url::parse(&url) {
        Ok(url) => url,
        Err(_) => return Err(Error::InvalidUrl)
    };
    let paths = parsed_url.path_segments();
    let file_name = paths.unwrap().last().unwrap();
    let mut file_writer = create_file_with_size(file_name, size)?;

    let client = Client::new();
    let mut response = client.get(&url).send().unwrap();
    let mut sum_bytes = 0;
    let mut bytes_buffer = [0; DEFAULT_DOWNLOAD_BUFFER_BYTES];
    while let Ok(n) = response.read(&mut bytes_buffer) {
        if n == 0 {
            return Err(Error::DownloadFail);
        }
        if file_writer.write(&bytes_buffer[0..n]).is_err() {
            return Err(Error::IoError);
        }
        sum_bytes += n as u64;
    }
    Ok(())
}

fn create_file_with_size(file_path: &str, size: u64) -> Result<File> {
    let path = Path::new(file_path);
    if path.exists() {
        println!("The path to store {} already exists! Overwrite it.", file_path);
    }
    match File::create(path) {
        Ok(file) => {
            file.set_len(size).expect("Cannot extend file to download size!");
            Ok(file)
        }
        Err(_) => Err(Error::IoError)
    }
}
