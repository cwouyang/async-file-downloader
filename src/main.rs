extern crate lib;

use lib::{arg_parser, downloader};
use std::process;

fn main() {
    let matches = arg_parser::parse_args();
    let json_url = matches.value_of(arg_parser::ARG_JSON_URL).unwrap();
    match downloader::download_file_list(json_url) {
        Ok(file_list) => downloader::download_files(file_list),
        Err(e) => {
            println!("Download file list fail: {:?}", e);
            process::exit(1);
        }
    }
}
