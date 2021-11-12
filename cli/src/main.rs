extern crate clap;

use clap::{App, SubCommand};

fn scribe(matches: &clap::ArgMatches) {
    println!("scribing now...");
}

fn main() {
    let matches = App::new("skriptorium")
        .version("1.0")
        .about("...soon there will be more information here...")
        .subcommand(
            SubCommand::with_name("scribe")
                .about("runs the generation")
                .arg_from_usage("-d, --debug 'Print debug information'"),
        )
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("scribe") {
        scribe(matches);
    }
}
