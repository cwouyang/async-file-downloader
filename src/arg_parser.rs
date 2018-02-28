use clap::App;
use clap::ArgMatches;

/// Constant to represent the name of supported argument for JSON URL
pub const ARG_JSON_URL: &str = "JSON URL";
/// Constant to represent the usage explanation for JSON URL
pub const ARG_USAGE: &str = "<JSON URL> JSON array contains a list of target URL to download";

/// parse_args parses and determines the correctness of arguments passed in. If arguments are all
/// correct, it returns the matched result. Otherwise, it exits the process.
pub fn parse_args() -> ArgMatches<'static> {
    return App::new("Async File Downloader")
        .version("0.1")
        .about("Given an URL which responses a JSON array containing a list of target URLs. This \
            program downloads files from the target URLs.")
        .author("cwouyang <cwouyang@protonmail.com>")
        .args_from_usage(&ARG_USAGE)
        .get_matches();
}
