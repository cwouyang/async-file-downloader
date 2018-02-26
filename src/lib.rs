extern crate clap;
extern crate num_cpus;
extern crate reqwest;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate threadpool;
extern crate url;

pub mod file;
pub mod arg_parser;
pub mod downloader;