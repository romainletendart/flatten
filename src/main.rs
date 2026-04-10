use std::io::BufReader;

use anyhow::{Context, Error, Result};

mod json;
mod json_parser;

fn main() -> Result<(), Error> {
    let mut args = std::env::args();
    let program_name = args.next().context("Missing program name")?;
    let usage = format!("Usage: {program_name} json_file");
    let json_path = args.next().context(usage.clone())?;
    let json_file = std::fs::File::open(&json_path)
        .context(format!("Couldn't open {json_path}", json_path = &json_path))?;

    let json_reader = BufReader::new(json_file);
    let stream = json::Stream::new(json_reader)?;
    for (path, value) in stream {
        println!("{path} = {value}");
    }

    Ok(())
}
