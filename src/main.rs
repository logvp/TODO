use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

use anyhow::anyhow;
use chrono::{DateTime, Local};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

const VERBOSE: bool = true;
const FILENAME: &str = "todofile.json";

#[derive(Serialize, Deserialize, Debug)]
struct TodoItem {
    timestamp: DateTime<Local>,
    path: PathBuf,
    message: String,
}

fn main() -> anyhow::Result<()> {
    let project_dir = ProjectDirs::from("io.github", "logvp", "todoapp")
        .ok_or(anyhow!("Could not get project root"))?;
    let storage = project_dir.data_local_dir();
    let datafile = storage.join(FILENAME);
    if VERBOSE {
        println!("{:?}", storage);
        println!("{:?}", datafile);
    }
    fs::create_dir_all(storage)?;
    let text = fs::read_to_string(&datafile).ok();
    let data: Vec<TodoItem> = if let Some(text) = text {
        if VERBOSE {
            println!("{}", text);
        }
        serde_json::from_str(&text)?
    } else {
        Vec::new()
    };
    let message = env::args().skip(1).collect::<Vec<String>>().join(" ");
    if message.is_empty() {
        for item in data {
            println!("{:?}", item);
        }
    } else {
        println!("{}", message);
        let mut data = data;
        data.push(TodoItem {
            timestamp: Local::now(),
            path: env::current_dir()?,
            message,
        });
        let mut f = File::options().append(true).create(true).open(&datafile)?;
        writeln!(&mut f, "{}", serde_json::to_string(&data)?)?;
    }
    Ok(())
}
