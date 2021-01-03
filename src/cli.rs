use crate::constants;
use clap::{App, Arg, ArgMatches};

#[derive(Debug)]
pub struct CliOptions {
    url: String,
    destination: String,
    no_download: bool,
    verbosity: u64,
    limit: u64,
    skip: u64,
}

pub fn configure_parser(default_path: &str) -> App {
    // The app definition
    let app = App::new(constants::NAME)
        .version(constants::VERSION)
        .author(constants::AUTHOR)
        .about(constants::ABOUT)
        .after_help(constants::LICENSE)
        .args(&[
            // URL
            Arg::with_name("URL")
                .help("The root URL you want to crawl & download")
                .required(true)
                .index(1),
            // Destination
            Arg::with_name("destination")
                .help("The path to which to write the downloaded files to")
                .default_value(default_path)
                .short("d")
                .long("destination"),
            // no download
            Arg::with_name("disable download")
                .help("Crawls without downloading (only writes json)")
                .short("j")
                .long("json-without-download"),
            // verbosity
            Arg::with_name("verbosity")
                .short("v")
                .multiple(true)
                .help("The verbosity level of the application"),
            // limit
            Arg::with_name("limit")
                .help("Limit to n finding(s) to be downloaded")
                .short("l")
                .long("limit")
                .default_value("0"),
            // skip
            Arg::with_name("skip")
                .help("Skip the first n finding(s)")
                .short("s")
                .long("skip")
                .default_value("0"),
        ]);

    app
}

pub fn get_options(matches: ArgMatches) -> Result<CliOptions, anyhow::Error> {
    Ok(CliOptions {
        url: matches.value_of("URL").unwrap().to_owned(),
        destination: matches.value_of("destination").unwrap().to_owned(),
        no_download: matches.is_present("disable download"),
        verbosity: (matches.occurrences_of("verbosity")),
        limit: matches.value_of("limit").unwrap().parse::<u64>()?,
        skip: matches.value_of("skip").unwrap().parse::<u64>()?,
    })
}
