extern crate clap;

use clap::App;

fn main() {
    App::new("Async File Downloader")
        .version("0.1")
        .about("Given an URL which responses a JSON array containing a list of target URLs. This \
            program downloads files from the target URLs.")
        .author("cwouyang <cwouyang@protonmail.com>")
        .args_from_usage("<JSON URL> 'JSON array contains a list of target URL to download'")
        .get_matches();
}
