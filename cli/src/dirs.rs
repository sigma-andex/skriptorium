use directories;

use crate::types;
use std::convert;
use std::error;
use std::fmt;
use std::fs;
use std::io;
use std::path;

const QUALIFIER: &str = "com";
const ORGANIZATION: &str = "skriptorium";
const PROJECT_NAME: &str = "skriptorium-cli";

#[derive(Debug)]
pub enum DirectoryError {
    ProjectDirUnavailable,
    ProjectDirIOError(io::Error),
}

impl error::Error for DirectoryError {}

impl fmt::Display for DirectoryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DirectoryError::ProjectDirUnavailable => {
                write!(f, "Project directory couldn't be determined.")
            }
            DirectoryError::ProjectDirIOError(err) => {
                write!(f, "Project directory couldn't be created.")
            }
        }
    }
}

impl convert::From<io::Error> for DirectoryError {
    fn from(error: io::Error) -> Self {
        DirectoryError::ProjectDirIOError(error)
    }
}

pub fn get_data_dir() -> types::Result<path::PathBuf> {
    let project_dir = directories::ProjectDirs::from(QUALIFIER, ORGANIZATION, PROJECT_NAME)
        .ok_or(DirectoryError::ProjectDirUnavailable)?;
    if !&project_dir.data_dir().exists() {
        fs::create_dir_all(&project_dir.data_dir())?;
    }
    let data_dir = project_dir.data_dir().to_path_buf();
    Ok(data_dir)
}
