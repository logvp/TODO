#![deny(
    unsafe_code,
    clippy::correctness,
    clippy::suspicious,
    unused_must_use,
    unfulfilled_lint_expectations
)]
#![warn(clippy::complexity, clippy::perf, clippy::style)]
#![warn(clippy::pedantic)]
#![allow(
    clippy::missing_panics_doc,
    clippy::wildcard_imports,
    clippy::semicolon_if_nothing_returned,
    clippy::uninlined_format_args,
    clippy::missing_errors_doc,
    clippy::match_same_arms,
    clippy::must_use_candidate
)]

use std::{env, path};
use std::fs::{self, File};
use std::io::Write;
use std::iter::Peekable;
use std::path::PathBuf;

use anyhow::{Context, Result, anyhow, bail, ensure};
use chrono::{DateTime, Local};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

const FILENAME: &str = "todofile.json";
const TIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

#[derive(Serialize, Deserialize, Debug)]
struct TodoItem {
    timestamp: DateTime<Local>,
    path: PathBuf,
    message: String,
    completed: Option<DateTime<Local>>,
}

struct Cli {
    verbose: bool,
    paths: Vec<PathBuf>,
    command: Command,
}

enum Command {
    AddItem { message: String },
    DeleteItem { index: usize },
    CompleteItem { index: usize },
    ShowItems,
}

fn parse_message_command(args: &mut Peekable<env::Args>) -> Command {
    Command::AddItem { message: args.into_iter().collect::<Vec<_>>().join(" ") }
}

fn parse_index(args: &mut Peekable<env::Args>) -> Result<usize> {
    if let Some(arg) = args.next() {
        arg.parse().map_err(|_| anyhow!("Could not parse \"{}\" as integer INDEX", arg))
    } else {
        bail!("Expected positional argument INDEX")
    }
}

fn parse_delete_command(args: &mut Peekable<env::Args>) -> Result<Command> {
    let index = parse_index(args)?;
    ensure!(args.peek().is_none(), "Unknown arguments: {:?}", args.collect::<Vec<_>>());
    Ok(Command::DeleteItem { index })
}

fn parse_complete_command(args: &mut Peekable<env::Args>) -> Result<Command> {
    let index = parse_index(args)?;
    ensure!(args.peek().is_none(), "Unknown arguments: {:?}", args.collect::<Vec<_>>());
    Ok(Command::CompleteItem { index })
}

fn parse_show_command() -> Command {
    Command::ShowItems
}

fn parse_args() -> Result<Cli> {
    let mut args = env::args().peekable();
    args.next().expect("Expected executable as first argument");

    let mut verbose = false;
    let mut paths = Vec::new();

    let command = loop {
        match args.peek().map(String::as_str) {
            Some("--verbose" | "-v") => {
                args.next();
                verbose = true;
            }
            Some("--path" | "-p") => {
                args.next();
                if let Some(path) = args.next() {
                    paths.push(fs::canonicalize(path)?);
                } else {
                    bail!("Expected argument following --filter");
                }
            }
            Some("--delete" | "-d") => {
                args.next();
                break parse_delete_command(&mut args)?;
            }
            Some("--complete" | "-c") => {
                args.next();
                break parse_complete_command(&mut args)?;
            }
            Some(flag) if flag.starts_with("--") => {
                bail!("Unknown argument \"{}\"", flag)
            }
            Some(_) => break parse_message_command(&mut args),
            None => break parse_show_command(),
        }
    };

    Ok(Cli { verbose, paths, command })
}

fn main() -> anyhow::Result<()> {
    let cli = parse_args()?;

    let project_dir = ProjectDirs::from("io.github", "logvp", "todoapp")
        .ok_or(anyhow!("Could not get project root"))?;
    let storage = project_dir.data_local_dir();
    let datafile = storage.join(FILENAME);
    if cli.verbose {
        println!("{}", storage.display());
        println!("{}", datafile.display());
    }
    fs::create_dir_all(storage)?;
    let text = fs::read_to_string(&datafile).ok();
    let all_data: Vec<TodoItem> = if let Some(text) = text {
        if cli.verbose {
            println!("{}", text);
        }
        serde_json::from_str(&text).context("While reading todo list from disk")?
    } else {
        Vec::new()
    };

    let mut data = Vec::new();
    let mut rest = Vec::new();
    if cli.paths.is_empty() {
        data = all_data;
    } else {
        for item in all_data {
            let item_path = fs::canonicalize(&item.path)?;
            for p in &cli.paths {
                if item_path.starts_with(p) {
                    data.push(item);
                    break;
                } else {
                    rest.push(item);
                    break;
                }
            }
        }
    }

    match cli.command {
        Command::AddItem { message } => {
            if cli.verbose {
                println!("{}", message);
            }
            data.push(TodoItem {
                timestamp: Local::now(),
                path: path::absolute(env::current_dir()?)?,
                message,
                completed: None,
            });
        }
        Command::DeleteItem { index } => {
            data.remove(index.checked_sub(1).unwrap());
        }
        Command::CompleteItem { index } => {
            data[index.checked_sub(1).unwrap()].completed = Some(Local::now());
        }
        Command::ShowItems => {
            for (i, item) in data.iter().enumerate() {
                println!(
                    "{} [{}] {} ({})",
                    i + 1,
                    if item.completed.is_some() { "x" } else { " " },
                    item.message,
                    item.timestamp.format(TIME_FORMAT)
                );
            }
            // No need to update the list
            return Ok(());
        }
    }

    // Finally, combine the selected and unselected items and save to disk
    data.append(&mut rest);
    data.sort_by_key(|item| item.timestamp);
    let mut f = File::options().create(true).write(true).truncate(true).open(&datafile)?;
    writeln!(&mut f, "{}", serde_json::to_string(&data)?)?;
    Ok(())
}
