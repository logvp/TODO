use std::env::args;
use std::fs;
use std::io::Write;

use anyhow::anyhow;
use directories::ProjectDirs;

const FILENAME: &str = "todofile.txt";

fn main() -> anyhow::Result<()> {
    let project_dir = ProjectDirs::from("io.github", "logvp", "todoapp").ok_or(anyhow!("Could not get project root"))?;
    let storage = project_dir.data_local_dir();
    let datafile = storage.join(FILENAME);
    fs::create_dir_all(storage)?;
    println!("{:?}", storage);
    println!("{:?}", datafile);
    let string = args().skip(1).collect::<Vec<String>>().join(" ");
    if string.is_empty() {
        let data = fs::read_to_string(datafile).unwrap_or(String::new());
        println!("{}", data);
    } else {
        let mut f = fs::File::options().append(true).create(true).open(datafile)?;
        writeln!(&mut f, "{}", string)?;
        println!("{}", string);
    }
    Ok(())
}
