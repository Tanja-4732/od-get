pub(crate) mod cli;
pub(crate) mod constants;
pub(crate) mod download;

use anyhow::Result;
use download::{crawl, fetch};

#[tokio::main]
async fn main() -> Result<()> {
    // The working directory
    let pwd = std::env::current_dir()
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();

    let matches = cli::configure_parser(&pwd).get_matches();

    // Print the name and version of the application along its license notice
    println!("{} {}", constants::NAME, constants::VERSION);
    println!("{}\n", constants::LICENSE);

    let cli_options = cli::get_options(matches).expect(
        "Invalid arguments (skip & limit must be numbers and the working directory must exist)",
    );

    println!("{:?}", &cli_options);

    // Make a new client
    let client = reqwest::Client::new();

    // Crawl the root directory
    let mut root = crawl::get_root_dir(&cli_options.url, &client).await?;

    // Expand the tree
    crawl::expand_tree(&mut root, &client).await?;

    println!("{:?}", &root);

    fetch::download_recursive(
        &root,
        &cli_options,
        &client,
        &mut download::fetch::LimitCounts::new(),
    )
    .await?;

    Ok(())
}
