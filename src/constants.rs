// The name of the application
pub const NAME: &'static str = "od-get";

/// The main author of the application
pub const AUTHOR: &'static str = "Bernd-L <git@bernd.pw>";

/// The semantic-version string of the application
pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");

/// Describes the application (i.e. its use cases) in a short phrase
pub const ABOUT: &'static str =
    "A Rust tool for recursively crawling & downloading data from open directories";

/// The licence notice (AGPL 3) of the application
pub const LICENSE: &'static str = concat![
    "Copyright 2021 Bernd-L; All rights reserved.\n",
    "Licensed under the AGPL 3.0 <https://www.gnu.org/licenses/agpl-3.0.en.html>"
];
