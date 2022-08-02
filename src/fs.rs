use crate::types::Request;
use anyhow::{bail, Context, Result};
use std::{collections::HashSet, fs, path::Path, process};

pub fn open_file<P: AsRef<Path>>(path: P) -> Result<Vec<Request>>
where
    P: std::fmt::Debug,
{
    let file = fs::read_to_string(path.as_ref())
        .with_context(|| format!("Failed to open file: {:?}", path))?;
    let reqs: Result<Vec<Request>, serde_json::error::Error> = serde_json::from_str(file.as_str());
    match reqs {
        Ok(r) => {
            let mut dedup = HashSet::new();
            r.iter().filter(|re| re.name.is_some()).for_each(|re| {
                if dedup.get(re.name.as_ref().unwrap()).is_some() {
                    error!(
                        "name fields are duplicate in one file. Please use unique name:
file: {},
name: {}",
                        Path::display(path.as_ref()),
                        re.name.as_ref().unwrap()
                    );
                    process::exit(1)
                };
                dedup.insert(re.name.as_ref().unwrap());
            });
            Ok(r)
        }
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
