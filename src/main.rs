extern crate clap;
extern crate reqwest;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use clap::App;
use clap::ArgMatches;
use reqwest::{Client, Response};
use serde_json::Value;
use std::process;

const ARG_JSON_URL: &str = "JSON URL";
const ARG_USAGE: &str = "<JSON URL> JSON array contains a list of target URL to download";

#[derive(Deserialize, Debug)]
struct File {
    url: String,
    file_size: u64,
}

fn main() {
    let matches = parse_args();
    let json_url = matches.value_of(ARG_JSON_URL).unwrap();
    let client = Client::new();
    let file_list = download_file_list(client, json_url);
}

fn parse_args() -> ArgMatches<'static> {
    return App::new("Async File Downloader")
        .version("0.1")
        .about("Given an URL which responses a JSON array containing a list of target URLs. This \
            program downloads files from the target URLs.")
        .author("cwouyang <cwouyang@protonmail.com>")
        .args_from_usage(&ARG_USAGE)
        .get_matches();
}

fn download_file_list(client: Client, json_url: &str) -> Vec<File> {
    let response = match client.get(json_url).send() {
        Ok(response) => response,
        Err(e) => {
            println!("Invalid {}: {}\n{}", ARG_JSON_URL, json_url, e);
            process::exit(1);
        }
    };
    return create_file_list(response);
}

fn create_file_list(mut response: Response) -> Vec<File> {
    let json_vec = match response.json() {
        Ok(Value::Array(list)) => list,
        Ok(_) => unreachable!(),
        Err(e) => {
            println!("Invalid JSON response: {:?}\n{}", response.text(), e);
            process::exit(1);
        }
    };

    let mut file_list: Vec<File> = Vec::new();
    for (_, file_info) in json_vec.iter().enumerate() {
        let url = match file_info["url"] {
            Value::String(ref url) => url,
            _ => unreachable!()
        };
        let file_size = match file_info["size"] {
            Value::Number(ref size) => size,
            _ => unreachable!()
        }.as_u64().unwrap();
        file_list.push(File { url: url.clone(), file_size });
        println!("{} {}", url, file_size);
    }
    return file_list;
}
