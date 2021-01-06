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
    pub limit_count: u64,
    pub skip_count: u64,
    pub recursion_limit: u64,
    pub file_filter: Option<Regex>,
    pub path_filter: Option<Regex>,
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
                .long("destination"),
            Arg::with_name("disable download")
                .help("Crawls without downloading (only writes json)")
                .short("j")
                .long("json-without-download"),
            Arg::with_name("verbosity")
                .short("v")
                .multiple(true)
                .help("The verbosity level of the application"),
            Arg::with_name("limit")
                .help("Limit to n finding(s) to be downloaded")
                .short("l")
                .long("limit")
                .default_value("0"),
            Arg::with_name("skip")
                .help("Skip the first n finding(s)")
                .short("s")
                .long("skip")
                .default_value("0"),
            Arg::with_name("max_depth")
                .help("Maximum recursion depth (0 is unlimited)")
                .short("r")
                .long("recursive-depth")
                .default_value("0"),
            Arg::with_name("file_filter")
                .takes_value(true)
                .help("Regex filter to exclude file names")
                .short("f")
                .long("file-filter"),
            Arg::with_name("path_filter")
                .takes_value(true)
                .help("Regex filter to exclude paths names")
                .short("p")
                .long("path-filter"),
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
        limit_count: matches.value_of("limit").unwrap().parse::<u64>()?,
        skip_count: matches.value_of("skip").unwrap().parse::<u64>()?,
        recursion_limit: matches.value_of("max_depth").unwrap().parse::<u64>()?,
        file_filter: make_regex("file_filter"),
        path_filter: make_regex("path_filter"),
    })
}
