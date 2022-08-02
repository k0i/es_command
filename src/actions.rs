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
                error!("{}", e);
                process::exit(1);
            })
        });
        handles.push(handle);
    });
    for h in handles {
        if let Err(e) = h.join() {
            error!("{:?}", e);
        };
    }
}

fn process<P: AsRef<Path>>(path: P) -> Result<()>
where
    P: std::fmt::Debug,
{
    let reqs = open_file(path)?;
    let mut chain = RequestChainAndRes::new();
    let mut errs = vec![];
    let len = reqs.len();
    reqs.into_iter()
        .enumerate()
        .for_each(|(i, r)| match exec(r, &mut chain) {
            Ok(res) => {
                if i + 1 == len {
                    for (k, v) in res.iter() {
                        println!("\x1b[32mName\x1b[m: \x1b[35m{}\x1b[m", k);
                        println!();
                        println!("\x1b[34mResponse\x1b[m: \x1b[36m{}\x1b[m", v);
                        println!();
                    }
                }
            }
            Err(e) => errs.push(e),
        });
    if !errs.is_empty() {
        errs.into_iter().for_each(|e| error!("{}", e))
    }
    Ok(())
}

fn exec(req: Request, chain: &mut RequestChainAndRes) -> Result<&Vec<(String, Value)>> {
    let body = refine_body(req.body, chain);
    let bd = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", "echo", &body.to_string()])
            .stdout(Stdio::piped())
            .spawn()
            .with_context(|| format!("Failed to echo request body: {}", body))?
            .stdout
    } else {
        Command::new("echo")
            .arg(body.to_string())
            .stdout(Stdio::piped())
            .spawn()
            .with_context(|| format!("Failed to echo request body: {}", body))?
            .stdout
    };
    if bd.is_none() {
        bail!("Stdout is empty: something went wrong.");
    }
    let p = Command::new("evans")
        .args([
            "--host",
            "localhost",
            "-r",
            "cli",
            "call",
            req.method.as_str(),
        ])
        .stdin(bd.unwrap())
        .output()
        .with_context(|| "Failed to execute Evans.")?;
    if !p.stderr.is_empty() {
        error!(
            "
            \x1b[31mFailed to execute Evans. All following requests were canceled: \x1b[m
            {:?}

            Req:{:?}

            Body:{:?}",
            from_utf8(&p.stderr).unwrap(),
            req.method,
            body.to_string()
        );
        process::exit(1);
    }
    let s: ResponseJson = serde_json::from_str(
        from_utf8(&p.stdout)
            .with_context(|| " \x1b[31mFailed \x1b[mto convert response to string.")?,
    )
    .with_context(|| "\x1b[31mFailed \x1b[mto parse response strings to json.")?;
    if req.name.is_some() {
        chain.res.insert(req.name.clone().unwrap(), s.clone());
        chain.log.push((req.name.unwrap(), s));
    } else if chain.res.get(&req.method).is_none() {
        chain.res.insert(req.method.clone(), s.clone());
        chain.log.push((req.method, s));
    } else {
        let mut dedup = 2;
        while chain.res.get(&format!("{}{}", req.method, dedup)).is_some() {
            dedup += 1;
        }
        chain
            .res
            .insert(format!("{}{}", req.method, dedup), s.clone());
        chain.log.push((format!("{}{}", req.method, dedup), s));
    }
    Ok(&chain.log)
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
                let mut res_messages = chain
                    .res
                    .get(variables[0])
                    .unwrap_or_else(|| {
                        error!(
                            "
                            \x1b[31mFailed \x1b[mto find request by Name : {}
                            {:?} 
                            ",
                            variables[0], chain.res
                        );
                        process::exit(1);
                    })
                    .clone();
                for key in variables.iter().skip(1) {
                    let temp = res_messages
                        .get(&key.to_string())
                        .unwrap_or_else(|| {
                            error!(
                                "
                                \x1b[31mFailed \x1b[mto find key : {}
message: {}
                                ",
                                key, res_messages
                            );
                            process::exit(1);
                        })
                        .clone();
                    res_messages = temp;
                }
                res_messages
            } else {
                let mut res_messages = chain
                    .res
                    .get(variables[0])
                    .unwrap_or_else(|| {
                        error!(
                            "
                            \x1b[31mFailed \x1b[mto get variable from response: {},
response: {:?}
                            ",
                            variables[0], chain.res
                        );
                        process::exit(1);
                    })
                    .clone();
                for key in variables.into_iter().skip(1) {
                    match res_messages {
                        Value::Object(obj) => {
                            res_messages = obj
                                .get(key)
                                .unwrap_or_else(|| {
                                    error!(
                                        "
                                        \x1b[31mFailed \x1b[mto get key: {}
message: {:?}
                                        ",
                                        key, obj
                                    );
                                    process::exit(1);
                                })
                                .clone();
                        }
                        Value::Array(arr) => {
                            res_messages =
                                arr[key.to_string().parse::<usize>().unwrap_or_else(|e| {
                                    error!(
                                        "
                                        \x1b[31mFailed \x1b[mto access array: 
                                        expected index but got: 
                                        {}
                                        {}",
                                        key, e
                                    );
                                    process::exit(1);
                                })]
                                .clone();
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
