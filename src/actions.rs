use crate::{
    fs::open_file,
    types::{Request, RequestChainAndRes, ResponseJson},
};
use anyhow::{bail, Context, Result};
use serde_json::Value;
use std::{
    path::Path,
    process::{self, Command, Stdio},
    str::from_utf8,
    thread,
};

pub fn action_evans(c: &seahorse::Context) {
    let mut handles = vec![];
    c.args.clone().into_iter().for_each(|s| {
        let handle = thread::spawn(move || {
            process(&s).unwrap_or_else(|e| {
                eprintln!("Error: {}", e);
                process::exit(1);
            })
        });
        handles.push(handle);
    });
    for h in handles {
        let output = h.join().unwrap();
        println!("output: {:?}", output.0);
        println!("errors: {:?}", output.1);
    }
}

fn process<P: AsRef<Path>>(path: P) -> Result<(Vec<Vec<String>>, Vec<anyhow::Error>)>
where
    P: std::fmt::Debug,
{
    let reqs = open_file(path)?;
    let mut chain = RequestChainAndRes::new();
    let mut logs = vec![];
    let mut errs = vec![];
    reqs.into_iter().for_each(|r| match exec(r, &mut chain) {
        Ok(res) => logs.push(res),
        Err(e) => errs.push(e),
    });
    Ok((logs, errs))
}

fn exec(req: Request, chain: &mut RequestChainAndRes) -> Result<Vec<String>> {
    let body = refine_body(req.body, chain);
    let bd = Command::new("echo")
        .arg(body.to_string())
        .stdout(Stdio::piped())
        .spawn()
        .with_context(|| format!("Failed to echo request body: {}", body))?
        .stdout;
    if bd.is_none() {
        bail!("stdout is empty: something went wrong!")
    }
    let p = Command::new("evans")
        .args(["-r", "cli", "call", req.method.as_str()])
        .stdin(bd.unwrap())
        .output();
    let s: ResponseJson = serde_json::from_str(from_utf8(&p.unwrap().stdout).unwrap()).unwrap();
    if req.name.is_some() {
        chain.res.insert(req.name.unwrap(), s.clone());
    } else {
        chain.res.insert(req.method, s.clone());
    }
    chain.log.push(s.to_string());
    Ok(chain.log.clone())
}

fn refine_body(body: Value, chain: &RequestChainAndRes) -> Value {
    if let Value::Object(mut obj) = body {
        let c = obj.clone();
        c.into_iter().for_each(|(k, v)| {
            let new_val = refine_body(v, chain);
            obj.remove_entry(&k);
            obj.insert(k, new_val);
        });
        return Value::Object(obj);
    }
    if let Value::Array(arr) = body {
        return Value::Array(arr.into_iter().map(|a| refine_body(a, chain)).collect());
    }
    if let Value::String(st) = body {
        return resolve(st, chain);
    }
    body
}
fn resolve(s: String, chain: &RequestChainAndRes) -> Value {
    match s.get(..2) {
        Some("$$") => {
            let variables: Vec<_> = s.get(2..).unwrap().split('.').collect();
            if variables.len() <= 2 {
                let mut res_messages = chain.res.get(variables[0]).unwrap().clone();
                for key in &variables {
                    let temp = res_messages.get(&key.to_string()).unwrap().clone();
                    res_messages = temp;
                }
                res_messages
            } else {
                let mut res_messages = chain.res.get(variables[0]).unwrap().clone();
                for key in variables.into_iter().skip(1) {
                    match res_messages {
                        Value::Object(obj) => {
                            res_messages = obj.get(key).unwrap().clone();
                        }
                        Value::Array(arr) => {
                            res_messages = arr[key.to_string().parse::<usize>().unwrap()].clone();
                        }
                        _ => {}
                    };
                }
                res_messages
            }
        }
        _ => Value::String(s),
    }
}
