extern crate clap;

use crate::api::classification;
use console::style;
use console::Emoji;
use indicatif::MultiProgress;
use indicatif::ProgressBar;
use indicatif::ProgressStyle;
use std::time::Duration;
use tokio;
use tokio::sync::mpsc;
use tokio::task::yield_now;

#[cfg(not(windows))]
const TICK_SETTINGS: (&str, u64) = ("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏ ", 80);

#[cfg(windows)]
const TICK_SETTINGS: (&str, u64) = (r"+-x| ", 200);

static PEN: Emoji<'_, '_> = Emoji("🖋 ", "=>");
static CROSS_MARK: Emoji<'_, '_> = Emoji("❌ ", "X");
static CLASSIFIED: Emoji<'_, '_> = Emoji("🗄️ ", "!");

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub async fn scribe<'a>(matches: &clap::ArgMatches<'a>) -> Result<()> {
    println!("{} {}", PEN, style("Scribing now...").bold().white());

    let snippet = "```
from cryptography.fernet import Fernet
key = Fernet.generate_key()
f = Fernet(key)
token = f.encrypt(b\"A really secret message. Not for prying eyes.\")
```";

    let (tx, mut rx) = mpsc::channel(32);

    let multi_progress = MultiProgress::new();

    let count = 30 * 100;
    let pb = multi_progress.add(ProgressBar::new(count));
    let progressbar_style = ProgressStyle::default_spinner()
        .tick_chars(TICK_SETTINGS.0)
        .template(" {spinner:.blue} {msg:<30} ");
    pb.set_style(progressbar_style);

    let classification_handle = tokio::spawn(async move {
        let classification_result = classification::classify(snippet.to_owned()).await;
        tx.send(classification_result).await
    });
    let result_handle = tokio::spawn(async move {
        loop {
            if let Ok(result) = rx.try_recv() {
                let msg = match result {
                    Ok(classification::Classification { classification })
                        if !String::is_empty(&classification) =>
                    {
                        format!(
                            " {} {} {}",
                            CLASSIFIED,
                            style("Classfication sucessful:").dim().white(),
                            style(classification).bold().white()
                        )
                    }
                    _ => format!(
                        " {} {}",
                        CROSS_MARK,
                        style("Classification failed").dim().white()
                    ),
                };

                pb.finish_with_message(msg);
                break;
            }
            pb.set_message(format!(
                "{}",
                style("Running classification...").dim().white()
            ));
            pb.inc(1);
            std::thread::sleep(Duration::from_millis(10));
        }

        yield_now().await;
    });

    multi_progress.join();
    let result = tokio::try_join!(classification_handle, result_handle);

    match result {
        Ok((first, second)) => {
            println!("Got result {:?}", first);
        }
        Err(err) => {
            println!("processing failed; error = {}", err);
        }
    }
    Ok(())
}