use crate::types;
use std::env;
use std::fmt;
use std::path;
use walkdir;

#[derive(Debug)]
pub enum DirectoryListingError {
    GitIgnoreNotFound,
}

impl std::error::Error for DirectoryListingError {}

impl fmt::Display for DirectoryListingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DirectoryListingError::GitIgnoreNotFound => write!(f, "Couldn't determine gitignore."),
        }
    }
}

fn is_hidden(entry: &walkdir::DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

fn is_directory(entry: &walkdir::DirEntry) -> bool {
    entry.path().is_dir()
}

fn is_excluded_extension(entry: &walkdir::DirEntry) -> bool {
    let excluded_extensions: Vec<&str> = vec!["lock"];
    entry
        .file_name()
        .to_str()
        .and_then(|s| {
            let p = path::Path::new(s);
            p.extension()
                .and_then(|e| e.to_str())
                .map(|e| excluded_extensions.contains(&e))
        })
        .unwrap_or(false)
}

pub async fn list_directories_async() -> types::Result<Vec<path::PathBuf>> {
    let handle = tokio::spawn(async { list_directories() });
    handle.await?
}

pub fn list_directories() -> types::Result<Vec<path::PathBuf>> {
    let current_dir = env::current_dir().map_err(|e| DirectoryListingError::GitIgnoreNotFound)?;
    let gitignore_path = path::Path::new(".gitignore");
    let gitignore_abs_path = current_dir.join(gitignore_path);
    let gitignore = gitignore::File::new(gitignore_abs_path.as_path())
        .map_err(|e| DirectoryListingError::GitIgnoreNotFound)?;

    let cur_dir = env::current_dir().unwrap();

    let results: Vec<path::PathBuf> = walkdir::WalkDir::new(".")
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| !is_hidden(e) && !is_excluded_extension(e) && !is_directory(e))
        .filter_map(|entry| {
            let entry_path = path::PathBuf::from(entry.path());
            let entry_path2 = entry_path.clone();
            let abs_path = cur_dir.join(entry_path);
            abs_path.canonicalize().ok().and_then(|used_path| {
                match gitignore.is_excluded(used_path.as_path()) {
                    Err(err) => None,
                    Ok(false) => Some(entry_path2),
                    Ok(true) => None,
                }
            })
        })
        .collect();
    Ok(results)
}
