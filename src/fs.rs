use crate::types::Request;
use anyhow::{bail, Context, Result};
use std::{fs, path::Path};

pub fn open_file<P: AsRef<Path>>(path: P) -> Result<Vec<Request>>
where
    P: std::fmt::Debug,
{
    let file = fs::read_to_string(path.as_ref())
        .with_context(|| format!("Failed to open file: {:?}", path))?;
    let reqs: Result<Vec<Request>, serde_json::error::Error> = serde_json::from_str(file.as_str());
    match reqs {
        Ok(r) => Ok(r),
        _ => {
            let req: Result<Request, serde_json::error::Error> =
                serde_json::from_str(file.as_str());
            if let Ok(r) = req {
                Ok(vec![r])
            } else {
                bail!("Failed to parse json: {}", file)
            }
        }
    }
}
