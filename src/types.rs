use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct Request {
    pub name: Option<String>,
    pub method: String,
    pub body: Value,
}

#[derive(Debug)]
pub struct RequestChainAndRes {
    pub res: HashMap<String, ResponseJson>,
    pub log: Vec<(String, Value)>,
}

impl RequestChainAndRes {
    pub fn new() -> Self {
        Self {
            res: HashMap::new(),
            log: vec![],
        }
    }
}

pub type ResponseJson = Value;
