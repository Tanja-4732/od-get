use crate::constants;
use clap::{App, Arg, ArgMatches};
use core::panic;
use regex::Regex;
use reqwest::Url;

#[derive(Debug)]
pub struct CliOptions {
    pub url: Url,
    pub destination: String,
    pub no_download: bool,
    pub verbosity: u64,
    pub limit_count: Option<u64>,
    pub skip_count: Option<u64>,
    pub recursion_limit: Option<u64>,
    pub file_filter: Option<Regex>,
    pub path_filter: Option<Regex>,
    pub file_matcher: Option<Regex>,
    pub path_matcher: Option<Regex>,
    pub state_store_path: Option<String>,
}

pub fn configure_parser(default_path: &str) -> App {
    let app = App::new(constants::NAME)
        .version(constants::VERSION)
        .author(constants::AUTHOR)
        .about(constants::ABOUT)
        .after_help(constants::LICENSE)
        .args(&[
            Arg::with_name("URL")
                .help("The root URL you want to crawl & download")
                .required(true)
                .index(1),
            Arg::with_name("destination")
                .help("The path to which to write the downloaded files to")
                .default_value(default_path)
                .short("d")
                .long("destination")
                .value_name("path"),
            Arg::with_name("disable download")
                .help("Crawls without downloading (you mut also use -S)")
                .short("n")
                .long("no-download"),
            Arg::with_name("verbosity")
                .short("v")
                .multiple(true)
                .help("The verbosity level of the application"),
            Arg::with_name("limit")
                .help("Limit to n finding(s) to be downloaded")
                .short("l")
                .long("limit")
                .value_name("integer")
                .default_value("0"),
            Arg::with_name("skip")
                .help("Skip the first n finding(s)")
                .short("s")
                .long("skip")
                .value_name("integer")
                .default_value("0"),
            Arg::with_name("max_depth")
                .help("Maximum recursion depth (0 is unlimited)")
                .short("r")
                .long("recursive-depth")
                .value_name("integer")
                .default_value("0"),
            Arg::with_name("file_filter")
                .takes_value(true)
                .help("Regex filter to exclude matching file names")
                .short("f")
                .long("file-filter")
                .value_name("regex"),
            Arg::with_name("path_filter")
                .takes_value(true)
                .help("Regex filter to exclude matching paths names")
                .short("p")
                .long("path-filter")
                .value_name("regex"),
            Arg::with_name("file_matcher")
                .takes_value(true)
                .help("Regex filter to exclude non-matching file names")
                .short("F")
                .long("file-matcher")
                .value_name("regex"),
            Arg::with_name("path_matcher")
                .takes_value(true)
                .help("Regex filter to exclude non-matching paths names")
                .short("P")
                .long("path-matcher")
                .value_name("regex"),
            Arg::with_name("state_store")
                .takes_value(true)
                .help("Store progress in a file (and resume when possible)")
                .short("S")
                .long("store-state")
                .value_name("path"),
        ]);

    app
}

pub fn get_options(matches: ArgMatches) -> Result<CliOptions, anyhow::Error> {
    let make_regex = |name: &str| {
        matches.value_of(name).and_then(|v| match Regex::new(v) {
            Ok(regex) => Some(regex),
            Err(err) => panic!(&format!("{:?}", err)),
        })
    };

    Ok(CliOptions {
        url: Url::parse(matches.value_of("URL").unwrap())?,
        destination: matches.value_of("destination").unwrap().to_owned(),
        no_download: matches.is_present("disable download"),
        verbosity: matches.occurrences_of("verbosity"),
        limit_count: make_option(matches.value_of("limit").unwrap().parse::<u64>()),
        skip_count: make_option(matches.value_of("skip").unwrap().parse::<u64>()),
        recursion_limit: make_option(matches.value_of("max_depth").unwrap().parse::<u64>()),
        file_filter: make_regex("file_filter"),
        path_filter: make_regex("path_filter"),
        file_matcher: make_regex("file_matcher"),
        path_matcher: make_regex("path_matcher"),
        state_store_path: matches.value_of("state_store").map(|path| path.to_owned()),
    })
}

/// Converts a number (which has to be greater than zero) to an option, or None (in case of zero)
fn make_option(number: Result<u64, std::num::ParseIntError>) -> Option<u64> {
    match number
        .unwrap_or_else(|err| panic!("Invalid number (must be a positive integer): {:?}", err))
    {
        0 => None,
        n => Some(n),
    }
}
