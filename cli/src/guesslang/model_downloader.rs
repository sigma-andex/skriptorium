use crate::dirs;
use crate::types;
use std::error;
use std::fmt;
use std::fs;
use std::io;
use std::path;

#[derive(Debug)]
pub enum DownloadError {
    FileCreationFailed(path::PathBuf),
    DownloadError(io::Error),
}

impl error::Error for DownloadError {}

impl fmt::Display for DownloadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DownloadError::FileCreationFailed(path) => write!(f, "Failed to create path {:?}", path.to_str()),
            DownloadError::DownloadError(err) => write!(f, "Failed to download file"),
        }
    }
}

pub fn get_models_guesslang_path() -> types::Result<path::PathBuf> {
    let mut models_guesslang_dir_buf = dirs::get_data_dir()?;

    models_guesslang_dir_buf.push("models");
    models_guesslang_dir_buf.push("guesslang");

    let models_guesslang_dir = models_guesslang_dir_buf.as_path();
    if !&models_guesslang_dir_buf.exists() {
        fs::create_dir_all(&models_guesslang_dir)?;
    }

    Ok(models_guesslang_dir_buf)
}

pub async fn get_or_download_file(
    base_path: &path::PathBuf,
    file: &path::Path,
    base_url: &reqwest::Url,
    file_url: &str,
) -> types::Result<path::PathBuf> {
    let path_buf = base_path.clone();
    let absolute_path = base_path.join(file);
    if absolute_path.exists() {
        Ok(path_buf)
    } else {
        let absolute_url = base_url.join(file_url)?;
        
        let response = reqwest::get(absolute_url).await?;
        let bytes = response.bytes().await?;
        let mut cursor = io::Cursor::new(bytes);

        let parent_directory = &absolute_path.parent().ok_or(DownloadError::FileCreationFailed(absolute_path.clone()))?;
        fs::create_dir_all(parent_directory)?;

        let mut dest = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(absolute_path)?;
        io::copy(&mut cursor, &mut dest)?;
        Ok(path_buf)
    }
}

pub async fn retrieve_model() -> types::Result<path::PathBuf> {
    let models_guesslang_dir_buf = get_models_guesslang_path()?;

    let base_url = reqwest::Url::parse(
        "https://raw.githubusercontent.com/sigma-andex/guesslang/master/guesslang/data/",
    )?;
    let languages_file = get_or_download_file(
        &models_guesslang_dir_buf,
        path::Path::new("languages.json"),
        &base_url,
        "languages.json",
    )
    .await?;

    let model_file = get_or_download_file(
        &models_guesslang_dir_buf,
        path::Path::new("model/saved_model.pb"),
        &base_url,
        "model/saved_model.pb",
    )
    .await?;


    let variables_index_file = get_or_download_file(
        &models_guesslang_dir_buf,
        path::Path::new("model/variables/variables.index"),
        &base_url,
        "model/variables/variables.index",
    )
    .await?;

    let variables_data_file = get_or_download_file(
        &models_guesslang_dir_buf,
        path::Path::new("model/variables/variables.data-00000-of-00001"),
        &base_url,
        "model/variables/variables.data-00000-of-00001",
    )
    .await?;

    Ok(models_guesslang_dir_buf)
}
