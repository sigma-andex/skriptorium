extern crate clap;

use crate::api::classification;
use crate::types::Result;
use console::style;
use console::Emoji;
use indicatif::ProgressBar;
use indicatif::ProgressStyle;
use std::time::Duration;
use tokio;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::sync::mpsc;
use tokio::task::yield_now;

#[cfg(not(windows))]
const TICK_SETTINGS: (&str, u64) = ("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏ ", 80);

#[cfg(windows)]
const TICK_SETTINGS: (&str, u64) = (r"+-x| ", 200);

static PEN: Emoji<'_, '_> = Emoji("🖋 ", "=>");
static CROSS_MARK: Emoji<'_, '_> = Emoji("❌ ", "X");
static CLASSIFIED: Emoji<'_, '_> = Emoji("🗄️ ", "!");

pub async fn read_utf8_file(file_name: String) -> Result<String> {
    let mut file = File::open(file_name).await?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).await?;

    let utf8_file = String::from_utf8(buffer)?;
    Ok(utf8_file)
}

enum ClassificationStatus {
    Done,
    Failed,
}

pub async fn classification_task<'a>(snippet: String) -> Result<classification::Classification> {
    let (tx, mut rx) = mpsc::channel(32);

    let count = 30 * 100;
    let pb = ProgressBar::new(count);
    let progressbar_style = ProgressStyle::default_spinner()
        .tick_chars(TICK_SETTINGS.0)
        .template(" {spinner:.blue} {msg:<30} ");
    pb.set_style(progressbar_style);

    let classification_handle = tokio::spawn(async move {
        let classification_result = classification::classify(snippet.to_owned()).await;
        match classification_result {
            Ok(classification) => {
                tx.send(ClassificationStatus::Done).await;
                Ok(classification)
            }
            Err(err) => {
                tx.send(ClassificationStatus::Done).await;
                Err(err)
            }
        }
    });

    let result_handle = tokio::spawn(async move {
        loop {
            if let Ok(result) = rx.try_recv() {
                let msg = match result {
                    ClassificationStatus::Done => {
                        format!(
                            " {} {}",
                            CLASSIFIED,
                            style("Classification successful").dim().white()
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

    let (first, _) = tokio::try_join!(classification_handle, result_handle)?;
    first
}

pub async fn scribe<'a>(matches: &clap::ArgMatches<'a>) -> Result<()> {
    println!("{} {}", PEN, style("Scribing now...").bold().white());

    let snippet = read_utf8_file("snippet.txt".to_owned()).await?;
    let result = classification_task(snippet).await?;

    Ok(())
}
