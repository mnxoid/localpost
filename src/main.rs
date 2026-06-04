use anyhow::Result;
use clap::{Command, arg};

use crate::config::Config;
mod commands;
mod config;
mod constants;

fn cli() -> Command {
    Command::new("localpost")
        .about("A simple cli file sharing tool")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("upload")
                .about("Upload file")
                .arg_required_else_help(true)
                .arg(arg!(file: <PATH>)),
        )
        .subcommand(
            Command::new("stop")
                .about("Stop serving a file")
                .arg_required_else_help(true)
                .arg(arg!(--all "Stop serving all files"))
                .arg(arg!(key: [KEY]).exclusive(true)),
        )
        .subcommand(Command::new("list").about("List currently served fiels"))
        .subcommand(
            Command::new("explore")
                .about("List files served on the local network or match patterns"),
        )
        .subcommand(
            Command::new("download")
                .about("Download a file by its key")
                .arg_required_else_help(true)
                .arg(arg!(key: <KEY> "The key that identifies the file"))
                .arg(arg!(--output <PATH> "Download location")),
        )
        .subcommand(
            Command::new("daemon")
                .arg(arg!(file: <PATH>))
                .arg(arg!(key: <KEY>))
                .hide(true),
        )
}

fn main() -> Result<()> {
    let matches = cli().get_matches();

    // Load the config file
    let config = Config::load();

    // Handle subcommands
    match matches.subcommand() {
        Some(("upload", sub)) => {
            let file = sub
                .get_one::<String>("file")
                .expect("File argument is required");
            commands::upload(&config, file)
        }
        Some(("stop", sub)) => commands::stop(sub.get_one::<String>("key")),
        Some(("list", _)) => commands::list(),
        Some(("explore", _)) => commands::explore(),
        Some(("download", sub)) => commands::download(
            sub.get_one::<String>("key")
                .expect("Key argument is required"),
            sub.get_one::<String>("output"),
        ),
        Some(("daemon", sub)) => commands::daemon(
            sub.get_one::<String>("file")
                .expect("File argument is required"),
            sub.get_one::<String>("key")
                .expect("Key argument is required"),
        ),
        _ => unreachable!(),
    }
}
