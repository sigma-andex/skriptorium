extern crate clap;

use crate::api::classification;
use crate::cmd::directory_listing;
use crate::guesslang;
use crate::types::Result;
use console::style;
use console::Emoji;
use indicatif::ProgressBar;
use indicatif::ProgressStyle;
use itertools::Itertools;
use rust_embed::RustEmbed;
use std::collections;
use std::env;
use std::fmt;
use std::future;
use std::path;
use std::time::Duration;
use tokio;
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc;
use tokio::task;

#[cfg(not(windows))]
const TICK_SETTINGS: (&str, u64) = ("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏ ", 80);

#[cfg(windows)]
const TICK_SETTINGS: (&str, u64) = (r"+-x| ", 200);

static PEN: Emoji<'_, '_> = Emoji("🖋", "=>");
static CROSS_MARK: Emoji<'_, '_> = Emoji("🔥", "X");
static CLASSIFIED: Emoji<'_, '_> = Emoji("🗄️ ", "C");
static LANGUAGE: Emoji<'_, '_> = Emoji("🌍", "L");
static FILES: Emoji<'_, '_> = Emoji("🗂", "L");

#[derive(RustEmbed)]
#[folder = "assets/"]
struct Asset;

pub async fn read_utf8_file(file_name: &path::Path) -> Result<String> {
    let mut file = fs::File::open(file_name).await?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).await?;

    let utf8_file = String::from_utf8(buffer)?;
    Ok(utf8_file)
}

pub async fn write_utf8_file(file_name: String, content: String) -> Result<()> {
    let mut buffer = fs::File::create(file_name).await?;

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
                    pb.set_message(format!("{}", style(&running).dim().white()));
                    pb.inc(1);
                    std::thread::sleep(Duration::from_millis(10));
                }
            };
        }
    });

    let (result, _) = tokio::try_join!(classification_handle, result_handle)?;
    result
}

pub async fn multi_language_detection(
    files: Vec<path::PathBuf>,
) -> Result<collections::HashMap<String, u64>> {
    let guesslang_model_path = guesslang::model_downloader::retrieve_model().await?;

    let guess_lang_settings =
        guesslang::classification::load_settings(guesslang_model_path).await?;

    let settings_box = std::sync::Arc::new(guess_lang_settings);

    let mut tasks: Vec<task::JoinHandle<Option<(String, u64)>>> = Vec::new();

    for file_path in files.iter() {
        let my_box = settings_box.clone();
        let my_path = path::PathBuf::from(file_path);
        tasks.push(tokio::spawn(async move {
            let maybe_file_contents = read_utf8_file(my_path.as_path()).await.ok();
            let maybe_file_size = my_path.metadata().ok().map(|md| md.len());
            maybe_file_contents
                .zip(maybe_file_size)
                .and_then(|(file_contents, file_size)| {
                    if let Some(maybe_classifications) =
                        guesslang::classification::classify(&*my_box, file_contents).ok()
                    {
                        maybe_classifications
                            .first()
                            .map(|(name, _, _)| (name.to_string(), file_size))
                    } else {
                        None
                    }
                })
        }))
    }

    let results: Vec<std::result::Result<Option<(String, u64)>, task::JoinError>> =
        futures::future::join_all(tasks).await;

    let successful_results: Vec<Option<(String, u64)>> = results.into_iter().flatten().collect();
    let some_results: Vec<(String, u64)> = successful_results.into_iter().flatten().collect();
    let mut results_map: collections::HashMap<String, u64> = collections::HashMap::new();
    for (name, size) in some_results.iter() {
        if let Some(x) = results_map.get_mut(name) {
            *x += size;
        } else {
            results_map.insert(name.to_string(), *size);
        }
    }
    Ok(results_map)
}
pub fn language_display_name_or_default(language: &str) -> String {
    let mappings: collections::HashMap<String, String> = Asset::get("languages.json")
        .and_then(|mappings_file| {
            std::str::from_utf8(mappings_file.data.as_ref())
                .ok()
                .map(|s| s.to_owned())
        })
        .and_then(|json| serde_json::from_str(json.as_str()).ok())
        .unwrap_or(collections::HashMap::new());

    mappings
        .get(language)
        .unwrap_or(&language.to_owned())
        .to_string()
}

pub fn get_primary_language(
    classifications: &collections::HashMap<String, u64>,
) -> Option<(String, u64)> {
    let sorted: Vec<(&String, &u64)> = classifications
        .iter()
        .sorted_by_key(|tuple| tuple.1)
        .rev()
        .collect();
    sorted
        .first()
        .map(|tuple| (tuple.0.clone(), tuple.1.clone()))
}

pub async fn language_detection(files: Vec<path::PathBuf>) -> Result<Option<String>> {
    let languages = multi_language_detection(files).await?;
    let determined_language = get_primary_language(&languages).map(|(k, _)| k);
    Ok(determined_language)
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
    println!("{}  {}", PEN, style("Scribing now...").bold().white());

    let running = format!(
        "{}",
        style("Scanning repo for source files...").dim().white()
    );

    let success = |files: &Vec<path::PathBuf>| {
        if !files.is_empty() {
            format!(
                "{}  {} {}",
                FILES,
                style("Repo analysis:").dim().white(),
                style(format!("{} revelant source files found.", files.len()))
                    .blue()
            )
        } else {
            format!(
                "{}  {} {}",
                FILES,
                style("Repo analysis:").dim().white(),
                style("Got zero relevant source files 🤷‍♀️").dim().white()
            )
        }
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
    let snippet = read_utf8_file(path::Path::new("snippet.txt")).await?;

    let running = format!("{}", style("Running language detection...").dim().white());

    let success = |maybe_language: &Option<String>| match maybe_language {
        Some(language) => format!(
            "{} {} {}",
            LANGUAGE,
            style("Detected language:").dim().white(),
            style(language_display_name_or_default(language))
                .blue(),
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
        language_detection(relevant_files),
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
                .blue(),
            style("- tldr").dim().white(),
            style(classification.tldr.to_string()).blue(),
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
        classification::classify(detected_language, snippet.clone()),
        running,
        success,
        failure,
    )
    .await?;

    let markdown = format!("# {}\n\n{}", result.classification, result.tldr);
    write_utf8_file("docs/README.md".to_owned(), markdown).await?;
    Ok(())
}
