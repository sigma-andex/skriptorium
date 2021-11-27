use crate::api::classification;
use crate::cmd::util;
use crate::types;
use rand::prelude::*;
use std::path;
use tokio;
use tokio::task;

// [TODO] Make this more intelligent by using a local DL model.
pub async fn select_files(
    detected_language: &Option<String>,
    relevant_files: &Vec<path::PathBuf>,
) -> types::Result<Vec<path::PathBuf>> {
    let file_selection: Vec<String> = relevant_files
        .iter()
        .take(30)
        .filter_map(|path| path.as_path().to_str())
        .map(|s| s.to_string())
        .collect();

    let selected_files = classification::select(detected_language.clone(), file_selection).await?;
    if !selected_files.files.is_empty() {
        let selected_paths: Vec<path::PathBuf> = selected_files
            .files
            .iter()
            .map(|file| path::PathBuf::from(file))
            .collect();
        let candidate_paths: Vec<path::PathBuf> = relevant_files
            .iter()
            .filter(|p| selected_paths.contains(p))
            .map(|pb| pb.clone())
            .collect();
        Ok(candidate_paths)
    } else {
        Ok(relevant_files.clone())
    }
}

pub async fn classify(
    detected_language: Option<String>,
    relevant_files: Vec<path::PathBuf>,
) -> types::Result<classification::Classification> {
    let selected_files = select_files(&detected_language, &relevant_files).await?;

    let mut tasks: Vec<task::JoinHandle<types::Result<(Option<String>, String)>>> = Vec::new();
    for file_path_buf in selected_files.iter() {
        let my_path = file_path_buf.clone();
        let my_path2 = file_path_buf.clone();
        tasks.push(tokio::spawn(async move {
            let content = util::read_utf8_file(my_path.as_path()).await?;
            Ok((my_path2.as_path().to_str().map(|p| p.to_string()), content))
        }));
    }

    let file_contents: Vec<
        std::result::Result<types::Result<(Option<String>, String)>, task::JoinError>,
    > = futures::future::join_all(tasks).await;
    let result: types::Result<Vec<(Option<String>, String)>> =
        file_contents.into_iter().flatten().collect();
    let files = result?;
    let classification = classification::classify(detected_language, files).await?;
    Ok(classification)
}
