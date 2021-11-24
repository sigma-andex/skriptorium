extern crate clap;

use crate::api::classification;
use crate::guesslang;
use crate::types::Result;
use crate::cmd::directory_listing;
use console::style;
use console::Emoji;
use indicatif::ProgressBar;
use indicatif::ProgressStyle;
use std::env;
use std::fmt;
use std::future;
use std::path;
use std::time::Duration;
use tokio;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc;

#[cfg(not(windows))]
const TICK_SETTINGS: (&str, u64) = ("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏ ", 80);

#[cfg(windows)]
const TICK_SETTINGS: (&str, u64) = (r"+-x| ", 200);

static PEN: Emoji<'_, '_> = Emoji("🖋", "=>");
static CROSS_MARK: Emoji<'_, '_> = Emoji("🔥", "X");
static CLASSIFIED: Emoji<'_, '_> = Emoji("🗄️ ", "C");
static LANGUAGE: Emoji<'_, '_> = Emoji("🌍", "L");
static FILES: Emoji<'_, '_> = Emoji("🗂", "L");


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
                        style(&running).dim().white()
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

pub async fn language_detection(snippet: String) -> Result<Option<(String, String, f32)>> {
    let guesslang_model_path = guesslang::model_downloader::retrieve_model().await?;
    let guess_lang_settings = guesslang::classification::load_settings(guesslang_model_path)?;
    let classification_result = guesslang::classification::classify(&guess_lang_settings, snippet)?;
    Ok(classification_result.first().map(|t| t.clone()))
}

#[derive(Debug)]
pub enum ScribeError {
    MissingInputParameter,
}

impl std::error::Error for ScribeError {}

impl fmt::Display for ScribeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ScribeError::MissingInputParameter => write!(f, "Input file or folder missing."),
        }
    }
}

pub async fn scribe<'a>(matches: &clap::ArgMatches<'a>) -> Result<()> {

    let input_file = matches
        .value_of("INPUT")
        .ok_or(ScribeError::MissingInputParameter)?;
    println!("{} {}", PEN, style("Scribing now...").bold().white());

    let running = format!("{}", style("Collecting relevant files...").dim().white());

    let success = |files: &Vec<path::PathBuf>| if !files.is_empty() {
        format!(
            "{}  {} {}",
            FILES,
            style(format!("{}",files.len())).dim().white()
            ,
            style("revelant files found!").dim().white()
        )
    } else {
        format!(
            "{}  {}",
            FILES,
            style("Got zero relevant files 🤷‍♀️").dim().white()
        )
    };

    let failure = |e: &Box<dyn std::error::Error + Send + Sync>| {
        format!(
            "{} {}",
            CROSS_MARK,
            style("Unable to get relevant files 😢").dim().white()
        )
    };
    let relevant_files = create_task(
        directory_listing::list_directories_async(),
        running,
        success,
        failure,
    )
    .await?;



    let snippet = read_utf8_file(input_file.to_string()).await?;

    let running = format!("{}", style("Running language detection...").dim().white());

    let success = |maybe_language: &Option<(String, String, f32)>| match maybe_language {
        Some((_, language, _)) => format!(
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
            "{} {}",
            CROSS_MARK,
            style("Unable to detect language 😢").dim().white()
        )
    };
    let detected_language = create_task(
        language_detection(snippet.clone()),
        running,
        success,
        failure,
    )
    .await?;

    let running = format!("{}", style("Running classification...").dim().white());

    let success = |classification: &classification::Classification| {
        format!(
            "{} {}\n      {} {}\n      {} {}",
            CLASSIFIED,
            style("Classification successful:").dim().white(),
            style("- name").dim().white(),
            style(classification.classification.to_string())
                .dim()
                .blue(),
            style("- tldr").dim().white(),
            style(classification.tldr.to_string()).dim().blue(),
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
        classification::classify(detected_language.map(|(abbr, _, _)| abbr), snippet.clone()),
        running,
        success,
        failure,
    )
    .await?;

    let markdown = format!("# {}\n\n{}", result.classification, result.tldr);
    write_utf8_file("docs/README.md".to_owned(), markdown).await?;
    Ok(())
}
