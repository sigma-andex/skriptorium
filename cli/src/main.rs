extern crate clap;

mod api;

use clap::{App, SubCommand};
use std::io::Error;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

async fn scribe<'a>(matches: &clap::ArgMatches<'a>) -> Result<()> {
    println!("scribing now...");
    let snippet = 
"```
from cryptography.fernet import Fernet
key = Fernet.generate_key()
f = Fernet(key)
token = f.encrypt(b\"A really secret message. Not for prying eyes.\")
```";
    let result = api::api::classify(snippet.to_owned()).await;
    match result {
        Ok(res) => println!("Got result {:?}", res),
        Err(err) => println!("Got err {:?}", err),
    }
    Ok(())
}

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
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("scribe") {
        let result = scribe(matches).await;
    }

    Ok(())
}
