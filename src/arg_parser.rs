use clap::App;
use clap::ArgMatches;

pub const ARG_JSON_URL: &str = "JSON URL";
pub const ARG_USAGE: &str = "<JSON URL> JSON array contains a list of target URL to download";

pub fn parse_args() -> ArgMatches<'static> {
    return App::new("Async File Downloader")
        .version("0.1")
        .about("Given an URL which responses a JSON array containing a list of target URLs. This \
            program downloads files from the target URLs.")
        .author("cwouyang <cwouyang@protonmail.com>")
        .args_from_usage(&ARG_USAGE)
        .get_matches();
}
