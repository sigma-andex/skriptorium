extern crate clap;

use crate::cmd::util;
use crate::guesslang;
use crate::types::Result;
use itertools::Itertools;
use rust_embed::RustEmbed;
use std::collections;
use std::path;
use tokio;
use tokio::task;

#[derive(RustEmbed)]
#[folder = "assets/"]
struct Asset;

pub async fn multi_language_detection(
    files: Vec<path::PathBuf>,
    guess_lang_settings: guesslang::classification::GuessLangSettings,
) -> Result<collections::HashMap<String, u64>> {
    let settings_box = std::sync::Arc::new(guess_lang_settings);

    let mut tasks: Vec<task::JoinHandle<Option<(String, u64)>>> = Vec::new();

    for file_path in files.iter() {
        let my_box = settings_box.clone();
        let my_path = path::PathBuf::from(file_path);
        tasks.push(tokio::spawn(async move {
            let maybe_file_contents = util::read_utf8_file(my_path.as_path()).await.ok();
            let maybe_file_size = my_path.metadata().ok().map(|md| md.len());
            maybe_file_contents
                .zip(maybe_file_size)
                .and_then(|(file_contents, file_size)| {
                    if let Some(maybe_classifications) =
                        guesslang::classification::classify(&*my_box, file_contents).ok()
                    {
                        maybe_classifications.first().map(|classification| {
                            (classification.identifier.to_string(), file_size)
                        })
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

    let results_map: collections::HashMap<String, u64> = classifications_to_map(&some_results);
    Ok(results_map)
}

pub fn classifications_to_map(
    classifications: &Vec<(String, u64)>,
) -> collections::HashMap<String, u64> {
    let mut results_map: collections::HashMap<String, u64> = collections::HashMap::new();
    for (name, size) in classifications.iter() {
        if let Some(x) = results_map.get_mut(name) {
            *x += size;
        } else {
            results_map.insert(name.to_string(), *size);
        }
    }
    results_map
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
    let guesslang_model_path = guesslang::model_downloader::retrieve_model().await?;
    let guess_lang_settings =
        guesslang::classification::load_settings(guesslang_model_path).await?;
    let languages = multi_language_detection(files, guess_lang_settings).await?;
    let determined_language = get_primary_language(&languages).map(|(k, _)| k);
    Ok(determined_language)
}
