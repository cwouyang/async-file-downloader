use file::FileInfo;
use indicatif::{MultiProgress, ProgressBar};
use md5;
use num_cpus;
use progressbar;
use reqwest::Client;
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
    let mut response = match client.get(json_url).send() {
        Ok(response) => response,
        Err(_) => return Err(Error::DownloadFail)
    };
    let json_vec = match response.json() {
        Ok(Value::Array(list)) => list,
        Ok(_) | Err(_) => return Err(Error::InvalidResponse)
    };
    return create_file_list(json_vec);
}

fn create_file_list(map_vec: Vec<Value>) -> Result<Vec<FileInfo>> {
    let mut file_list: Vec<FileInfo> = Vec::new();
    for (_, file_info) in map_vec.iter().enumerate() {
        let map = match file_info {
            &Value::Object(ref map) => map,
            _ => continue
        };
        let url = match map.get("url") {
            Some(url) => match url {
                &Value::String(ref url) => url,
                _ => continue
            },
            _ => continue
        };

        let parsed_url = match Url::parse(&url) {
            Ok(url) => url,
            Err(_) => continue
        };
        let paths = parsed_url.path_segments();
        let file_name = paths.unwrap().last().unwrap().to_owned();

        let file_size = match map.get("size") {
            Some(size) => match size {
                &Value::Number(ref size) => size,
                _ => continue
            },
            _ => continue
        }.as_u64().unwrap();
        file_list.push(FileInfo { url: url.clone(), name: file_name, size: file_size });
    }
    if file_list.len() == 0 {
        return Err(Error::InvalidResponse);
    }
    Ok(file_list)
}

pub fn download_files(files: Vec<FileInfo>) {
    if files.len() == 0 {
        println!("Empty file list");
        return;
    }
    let worker_count = num_cpus::get_physical();
    let pool = ThreadPool::new(worker_count);
    let mp = MultiProgress::new();

    println!("Start downloading files with {} threads", worker_count);
    for (_, file) in files.iter().enumerate() {
        let mut bar = progressbar::new(file.name.clone(), file.size);
        bar = mp.add(bar);

        let cloned_file = file.clone();
        pool.execute(move || {
            let file_name = cloned_file.name.clone();
            let file_size = cloned_file.size;
            if download_file(cloned_file, &bar).is_ok() {
                let mut file = File::open(file_name).unwrap();
                let mut data = Vec::with_capacity(file_size as usize);
                if file.read_to_end(&mut data).is_ok() {
                    bar.finish_with_message(&format!("{:x}", md5::compute(data)));
                } else {
                    bar.finish_with_message("Failed to compute MD5");
                }
            }
        });
    }
    mp.join().unwrap();
    pool.join();
}

pub fn download_file(file: FileInfo, bar: &ProgressBar) -> Result<()> {
    let client = Client::new();
    let mut response = client.get(&file.url).send().unwrap();
    bar.set_message("Downloading...");
    let status = response.status();
    if !status.is_success() {
        // It seems to be a constrain that the call of finish() will result in the complete of
        // progress bar, we set the length to 0 to present the failure of download here.
        bar.set_length(0);
        bar.finish_with_message(&format!("Response status: {:?}", status));
        return Err(Error::InvalidResponse);
    }

    let mut file_writer = create_file_with_size(&file.name, file.size)?;
    let mut bytes_buffer = [0; DEFAULT_DOWNLOAD_BUFFER_BYTES];
    let mut bytes_received = 0;
    let mut first_byte_received = false;
    let progress_update_interval_millis: Duration = Duration::from_millis(PROGRESS_UPDATE_INTERVAL_MILLIS);
    let mut last_progress_update_time = Instant::now() - progress_update_interval_millis;
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
        if now.duration_since(last_progress_update_time) > progress_update_interval_millis {
            last_progress_update_time = now;
            bar.set_position(bytes_received);
        }
    }
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

#[cfg(test)]
mod test_create_file_list {
    use super::*;

    #[test]
    fn no_data() {
        let v: Vec<Value> = Vec::new();
        let list = create_file_list(v);
        assert!(list.is_err());
    }

    #[test]
    fn only_url_data() {
        let v = vec![
            json!({"url": "http://aaa.bbb"}),
            json!({"url": "http://ccc.ddd"}),
        ];
        let list = create_file_list(v);
        assert!(list.is_err());
    }

    #[test]
    fn only_size_data() {
        let v = vec![
            json!({"size": 123}),
            json!({"size": 456}),
        ];
        let list = create_file_list(v);
        assert!(list.is_err());
    }

    #[test]
    fn invalid_data() {
        let v = vec![
            json!({"abc": "ddd"}),
            json!({"efg": 123}),
        ];
        let list = create_file_list(v);
        assert!(list.is_err());
    }

    #[test]
    fn normal_data_with_url_and_size() {
        let v = vec![
            json!({"url": "http://aaa.bbb", "size": 123}),
            json!({"url": "http://ccc.ddd", "size": 456}),
        ];
        let list = create_file_list(v);
        assert!(list.is_ok());
        let list = list.unwrap();
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn partial_invalid_data() {
        let v = vec![
            json!({"url": "http://aaa.bbb", "size": 123}),
            json!({"abc": "ddd"}),
            json!({"url": "http://ccc.ddd", "size": 456}),
            json!({"efg": 123}),
        ];
        let list = create_file_list(v);
        assert!(list.is_ok());
        let list = list.unwrap();
        assert_eq!(list.len(), 2);
    }
}
