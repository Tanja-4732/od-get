pub(crate) mod cli;
pub(crate) mod constants;

pub fn main() {
    // The working directory
    let pwd = std::env::current_dir()
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();

    let matches = cli::configure_parser(&pwd).get_matches();

    let cli_options = cli::get_options(matches).expect(
        "Invalid arguments (skip & limit must be numbers and the working directory must exist",
    );

    // Print the name and version of the application along its license notice
    println!("{} {}", constants::NAME, constants::VERSION);
    println!("{}\n", constants::LICENSE);

    // TODO do stuff
    println!("{:?}", cli_options);
}
