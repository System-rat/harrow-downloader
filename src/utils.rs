use std::path::Path;

use anyhow::{Context, Result};
use reqwest::Url;

pub fn filename_from_url(url: &str) -> Result<String> {
    let url = Url::parse(url)?;
    Path::new(url.path())
        .file_name()
        .context("Could not get filename")?
        .to_os_string()
        .into_string()
        .map_err(|_| anyhow::Error::msg("Could not convert to string"))
}
