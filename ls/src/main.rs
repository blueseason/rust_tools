use std::{env, fs};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    let arg1 = env::args().nth(1);

    let path = arg1.expect("usage: ls Path");
    let dirs: fs::ReadDir = fs::read_dir(path)?;
    for dir in dirs {
        let entry = dir?;
        println!("{:?} :{:?}", entry.file_name(), entry.path());
    }

    Ok(())
}
