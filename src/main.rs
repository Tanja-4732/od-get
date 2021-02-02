pub(crate) mod cli;
pub(crate) mod constants;
pub(crate) mod download;

use anyhow::Result;
use download::{
    crawl,
    fetch::{self, DownloadRecursiveStatus},
    types::Node,
};

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

    // Make a new client
    let client = reqwest::Client::new();

    // Crawl the root directory
    let mut root = crawl::get_root_dir(&cli_options.url, &client).await?;

    // Expand the tree
    if let Node::CrawledDir(_, ref mut children) = root {
        crawl::expand_node(children, &client).await?;
    } else {
        panic!("Cannot expand root node")
    }

    let mut counters = download::fetch::LimitCounts::new();

    let mut res = { fetch::download_recursive(&root, &cli_options, &client, &mut counters).await? };

    let mut counters = counters.clone();

    while let DownloadRecursiveStatus::Do(ref to_do) = res {
        for task in to_do {
            let (node, options, client) = task;
            res = fetch::download_recursive(node, options, client, &mut counters).await?;
        }
    }

    Ok(())
}
