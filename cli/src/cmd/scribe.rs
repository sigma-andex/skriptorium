extern crate clap;

use crate::api::classification;
use crate::guesslang;
use crate::types::Result;
use console::style;
use console::Emoji;
use indicatif::ProgressBar;
use indicatif::ProgressStyle;
use std::future;
use std::time::Duration;
use tokio;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc;
use tokio::task::yield_now;

#[cfg(not(windows))]
const TICK_SETTINGS: (&str, u64) = ("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏ ", 80);

#[cfg(windows)]
const TICK_SETTINGS: (&str, u64) = (r"+-x| ", 200);

static PEN: Emoji<'_, '_> = Emoji("🖋 ", "=>");
static CROSS_MARK: Emoji<'_, '_> = Emoji("❌ ", "X");
static CLASSIFIED: Emoji<'_, '_> = Emoji("🗄️ ", "C");
static LANGUAGE: Emoji<'_, '_> = Emoji("🌍 ", "L");

pub async fn read_utf8_file(file_name: String) -> Result<String> {
    let mut file = File::open(file_name).await?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).await?;

    let utf8_file = String::from_utf8(buffer)?;
    Ok(utf8_file)
}

pub async fn write_utf8_file(file_name: String, content: String) -> Result<()> {
    let mut buffer = File::create(file_name).await?;

    buffer.write_all(content.as_bytes()).await?;

    Ok(())
}

pub async fn create_task<F, Out>(
    task: F,
    running: String,
    success: fn(&Out) -> String,
    failure: fn(&Box<dyn std::error::Error + Send + Sync>) -> String,
) -> Result<Out>
where
    F: future::Future<Output = Result<Out>> + Send + 'static,
    Out: Send + Sync + std::fmt::Debug + 'static,
{
    let (tx, mut rx) = mpsc::channel(1);

    let count = 30 * 100;
    let pb = ProgressBar::new(count);
    let progressbar_style = ProgressStyle::default_spinner()
        .tick_chars(TICK_SETTINGS.0)
        .template(" {spinner:.blue} {msg:<30} ");
    pb.set_style(progressbar_style);

    let classification_handle = tokio::spawn(async move {
        let result = task.await;
        let msg = match &result {
            Ok(inner_result) => success(&inner_result),
            Err(err) => failure(err),
        };
        tx.send(msg).await?;
        result
    });

    let result_handle = tokio::spawn(async move {
        loop {
            match rx.try_recv() {
                Ok(msg) => {
                    pb.finish_with_message(msg);
                    break;
                }
                Err(err) => {
                    pb.set_message(format!(
                        "{}",
                        style("Running classification...").dim().white()
                    ));
                    pb.inc(1);
                    std::thread::sleep(Duration::from_millis(10));
                }
            };
        }
    });

    let (result, _) = tokio::try_join!(classification_handle, result_handle)?;
    result
}

pub async fn classification_task(snippet: String) -> Result<classification::Classification> {
    let (tx, mut rx) = mpsc::channel(1);

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
                tx.send(Ok(classification.clone())).await?;
                Ok(classification)
            }
            Err(err) => {
                tx.send(Err(
                    classification::ClassificationError::ClassificationFailed,
                ))
                .await?;
                Err(err)
            }
        }
    });

    let result_handle = tokio::spawn(async move {
        loop {
            if let Ok(result) = rx.try_recv() {
                let msg = match result {
                    Ok(classification::Classification { classification })
                        if !classification.is_empty() =>
                    {
                        format!(
                            " {} {} {}",
                            CLASSIFIED,
                            style("Classification successful: ").dim().white(),
                            style(classification).dim().blue(),
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

        yield_now().await
    });

    let (first, _) = tokio::try_join!(classification_handle, result_handle)?;
    first
}

pub async fn language_detection(snippet: String) -> Result<Option<String>> {
    let guesslang_model_path = guesslang::model_downloader::retrieve_model().await?;
    let guess_lang_settings = guesslang::classification::load_settings(guesslang_model_path)?;
    let classification_result = guesslang::classification::classify(&guess_lang_settings, snippet)?;
    Ok(classification_result
        .first()
        .map(|(name, _)| name.to_string()))
}

pub async fn scribe<'a>(matches: &clap::ArgMatches<'a>) -> Result<()> {
    println!("{} {}", PEN, style("Scribing now...").bold().white());

    let snippet = read_utf8_file("snippet.txt".to_owned()).await?;

    let running = format!("{}", style("Running language detection...").dim().white());

    let success = |maybe_language: &Option<String>| match maybe_language {
        Some(language) => format!(
            "{} {} {}",
            LANGUAGE,
            style("Detected language:").dim().white(),
            style(language.to_string()).dim().blue(),
        ),
        None => format!(
            "{} {}",
            LANGUAGE,
            style("Unsure which language that is 🧐").dim().white()
        ),
    };
    let failure = |e: &Box<dyn std::error::Error + Send + Sync>| {
        format!(
            " {} {}",
            CROSS_MARK,
            style("Unable to detect language 😢").dim().white()
        )
    };
    let result = create_task(
        language_detection(snippet.clone()),
        running,
        success,
        failure,
    )
    .await?;

    let running = format!("{}", style("Running classification...").dim().white());

    let success = |classification: &classification::Classification| {
        format!(
            "{} {} {}",
            CLASSIFIED,
            style("Classification successful: ").dim().white(),
            style(classification.classification.to_string())
                .dim()
                .blue(),
        )
    };
    let failure = |e: &Box<dyn std::error::Error + Send + Sync>| {
        format!(
            "{} {}",
            CROSS_MARK,
            style("Unable to classify code 😭").dim().white()
        )
    };
    let result: classification::Classification = create_task(
        classification::classify(snippet.clone()),
        running,
        success,
        failure,
    )
    .await?;

    let markdown = format!("This is a library for {}", result.classification);
    write_utf8_file("docs/README.md".to_owned(), markdown).await?;
    Ok(())
}
