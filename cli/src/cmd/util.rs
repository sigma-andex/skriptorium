use crate::types::Result;
use std::path;
use tokio;
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

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

pub fn limit_string(text: &str, length: usize) -> String {
    if (text.len() > length) {
        let text_trimmed: String = text.chars().take(length).collect();
        let mut trimmed = String::new();
        trimmed.push_str(text_trimmed.as_str());
        trimmed.push_str("...");
        trimmed
    } else {
        text.to_string()
    }
}
