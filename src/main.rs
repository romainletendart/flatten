use std::io::BufReader;

use anyhow::{Context, Error, Result};
use serde_json::Value;

use flatten::Path;
use flatten::PathComponent;

fn to_path_iter(path: Path, json_value: &Value) -> Box<dyn Iterator<Item = (Path, Value)> + '_> {
    match json_value {
        Value::Array(array) => {
            let it = array.iter().enumerate().flat_map(move |(index, value)| {
                let new_path = path.clone() + Path::new(&[PathComponent::Index(index)]);
                to_path_iter(new_path, value)
            });
            Box::new(it)
        }
        Value::Object(map) => {
            let it = map.into_iter().flat_map(move |(key, value)| {
                let new_path = path.clone() + Path::new(&[PathComponent::Key(key.clone())]);
                to_path_iter(new_path, value)
            });
            Box::new(it)
        }
        _ => Box::new([(path.clone(), json_value.clone())].into_iter()),
    }
}

fn main() -> Result<(), Error> {
    let mut args = std::env::args();
    let program_name = args.next().context("Missing program name")?;
    let usage = format!("Usage: {program_name} json_file");
    let json_path = args.next().context(usage.clone())?;
    let json_file = std::fs::File::open(&json_path)
        .context(format!("Couldn't open {json_path}", json_path = &json_path))?;
    let json_reader = BufReader::new(json_file);
    let path: Path = Path::default();
    let json_value: Value = serde_json::from_reader(json_reader)?;
    let it = to_path_iter(path, &json_value);
    for (path, value) in it {
        println!("{path} = {value}");
    }

    Ok(())
}
