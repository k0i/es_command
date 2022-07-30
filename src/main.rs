mod actions;
mod fs;
mod types;
use actions::action_evans;
use anyhow::Result;
use seahorse::App;
use std::env;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let app = App::new(env!("CARGO_PKG_NAME"))
        .description(env!("CARGO_PKG_DESCRIPTION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .version(env!("CARGO_PKG_VERSION"))
        .usage("es [path to json files]")
        .action(action_evans);
    app.run(args);
    Ok(())
}
