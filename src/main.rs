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

use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

use anyhow::{Context, anyhow};
use chrono::{DateTime, Local};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

const VERBOSE: bool = true;
const FILENAME: &str = "todofile.json";
const TIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

#[derive(Serialize, Deserialize, Debug)]
struct TodoItem {
    timestamp: DateTime<Local>,
    path: PathBuf,
    message: String,
    completed: Option<DateTime<Local>>,
}

fn main() -> anyhow::Result<()> {
    let project_dir = ProjectDirs::from("io.github", "logvp", "todoapp")
        .ok_or(anyhow!("Could not get project root"))?;
    let storage = project_dir.data_local_dir();
    let datafile = storage.join(FILENAME);
    if VERBOSE {
        println!("{}", storage.display());
        println!("{}", datafile.display());
    }
    fs::create_dir_all(storage)?;
    let text = fs::read_to_string(&datafile).ok();
    let data: Vec<TodoItem> = if let Some(text) = text {
        if VERBOSE {
            println!("{}", text);
        }
        serde_json::from_str(&text).context("While reading todo list from disk")?
    } else {
        Vec::new()
    };
    let message = env::args().skip(1).collect::<Vec<String>>().join(" ");
    if message.is_empty() {
        for (i, item) in data.iter().enumerate() {
            println!(
                "{} - {} ({})",
                i + 1,
                item.message,
                item.timestamp.format(TIME_FORMAT)
            );
        }
    } else {
        println!("{}", message);
        let mut data = data;
        data.push(TodoItem {
            timestamp: Local::now(),
            path: env::current_dir()?,
            message,
            completed: None,
        });
        let mut f = File::options().create(true).open(&datafile)?;
        writeln!(&mut f, "{}", serde_json::to_string(&data)?)?;
    }
    Ok(())
}
