extern crate clap;

mod api;
mod cmd;
mod types;

use crate::cmd::scribe;

use clap::{App, SubCommand};
use tokio;

use types::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let matches = App::new("skriptorium")
        .version("1.0")
        .about("...soon there will be more information here...")
        .subcommand(
            SubCommand::with_name("scribe")
                .about("runs the generation")
                .arg_from_usage("-d, --debug 'Print debug information'"),
        )
        .subcommand(
            SubCommand::with_name("record")
                .about("listens for file changes to generate a new documentation")
                .arg_from_usage("-d, --debug 'Print debug information'"),
        )
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("scribe") {
        let result = scribe::scribe(matches).await;
    }

    Ok(())
}
