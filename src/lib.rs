extern crate clap;
extern crate indicatif;
extern crate md5;
extern crate num_cpus;
extern crate reqwest;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate threadpool;
extern crate url;

pub mod file;
pub mod arg_parser;
pub mod downloader;
