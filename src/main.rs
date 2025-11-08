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

use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::{env, path};

use anyhow::{Context, anyhow};
use chrono::{DateTime, Local};
use clap::{ArgAction, Parser};
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

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    verbose: bool,

    #[arg(short, long = "path", action = ArgAction::Append)]
    paths: Vec<PathBuf>,

    #[arg(short, long, group = "command")]
    delete: Option<usize>,

    #[arg(short, long, group = "command")]
    complete: Option<usize>,

    #[arg(group = "command")]
    message: Option<Vec<String>>,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

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
        if cli.verbose {
            println!("No todo list file found");
        }
        Vec::new()
    };

    let mut data = Vec::new();
    let mut rest = Vec::new();
    if cli.paths.is_empty() {
        data = all_data;
    } else {
        let mut paths = Vec::new();
        for p in &cli.paths {
            paths.push(fs::canonicalize(p)?);
        }
        'items: for item in all_data {
            let item_path = fs::canonicalize(&item.path)?;
            for p in &paths {
                if item_path.starts_with(p) {
                    data.push(item);
                    continue 'items;
                }
            }
            rest.push(item);
        }
    }

    if let Some(message) = cli.message {
        let message = message.into_iter().collect::<Vec<_>>().join(" ");
        if cli.verbose {
            println!("{}", message);
        }
        data.push(TodoItem {
            timestamp: Local::now(),
            path: path::absolute(env::current_dir()?)?,
            message,
            completed: None,
        });
    } else if let Some(index) = cli.delete {
        data.remove(index.checked_sub(1).unwrap());
    } else if let Some(index) = cli.complete {
        data[index.checked_sub(1).unwrap()].completed = Some(Local::now());
    } else {
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

    // Finally, combine the selected and unselected items and save to disk
    data.append(&mut rest);
    data.sort_by_key(|item| item.timestamp);
    let mut f = File::options()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&datafile)?;
    writeln!(&mut f, "{}", serde_json::to_string(&data)?)?;
    Ok(())
}
