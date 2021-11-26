extern crate clap;

mod api;
mod cmd;
mod dirs;
mod guesslang;
mod types;
use console::style;

use crate::cmd::scribe;

use clap::{crate_version, App, AppSettings, Arg, SubCommand};
use tokio;

use types::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let matches = App::new(format!("{}", style("skriptorium").bold()))
        .version(crate_version!())
        .about("\n...your little helper to write the boring documentation for you!\nSkriptorium analyses your code repo to generate documentation automatically using deep learning (Guesslang, OpenAI and NLPCloud).")
        .setting(AppSettings::ArgRequiredElseHelp)
        .subcommand(
            SubCommand::with_name("scribe")
                .about("generate documentation for the INPUT folder")
                .arg(
                    Arg::with_name("INPUT")
                        .help("The input folder to use")
                        .default_value(".")
                        .index(1),
                ),
        )
        .subcommand(
            SubCommand::with_name("watch")
                .about("watches for file changes to generate a new documentation"),
        )
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("scribe") {
        let result = scribe::scribe(matches).await;
        match result {
            Ok(res) => println!("{}", style("\nDone.").dim().white()),
            Err(err) => println!("{:?}", err),
        }
    }

    Ok(())
}
