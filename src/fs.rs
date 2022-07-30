use crate::types::Request;
use anyhow::{Context, Result};
use std::{fs, path::Path};

pub fn open_file<P: AsRef<Path>>(path: P) -> Result<Vec<Request>>
where
    P: std::fmt::Debug,
{
    let file = fs::read_to_string(path.as_ref())
        .with_context(|| format!("Failed to open file: {:?}", path))?;
    let reqs: Vec<Request> = serde_json::from_str(file.as_str())
        .with_context(|| format!("Failed to parse json: {:?}", path))?;
    Ok(reqs)
}
