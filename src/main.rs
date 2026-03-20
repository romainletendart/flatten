use std::vec::Vec;
use std::{fmt::Display, io::BufReader};

use anyhow::{Context, Error, Result};
use serde_json::Value;

#[derive(Clone, Debug)]
enum PathComponent {
    Key(String),
    Index(usize),
}

impl Display for PathComponent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PathComponent::Index(index) => write!(f, "[{index}]"),
            PathComponent::Key(key) => write!(f, "\"{key}\""),
        }
    }
}

#[derive(Clone)]
struct Path {
    components: Vec<PathComponent>,
}

impl Path {
    fn new() -> Self {
        Self {
            components: Vec::new(),
        }
    }

    fn from_components(components: &[PathComponent]) -> Self {
        Self {
            components: components.to_vec(),
        }
    }
}

impl Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(".")?;
        f.write_str(&itertools::join(self.components.iter(), "."))
    }
}

impl std::ops::Add for Path {
    type Output = Path;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            components: [self.components, rhs.components].concat(),
        }
    }
}

fn to_path_iter(path: Path, json_value: &Value) -> Box<dyn Iterator<Item = (Path, Value)> + '_> {
    match json_value {
        Value::Array(array) => {
            let it = array.iter().enumerate().flat_map(move |(index, value)| {
                let new_path = path.clone() + Path::from_components(&[PathComponent::Index(index)]);
                to_path_iter(new_path, value)
            });
            Box::new(it)
        }
        Value::Object(map) => {
            let it = map.into_iter().flat_map(move |(key, value)| {
                let new_path =
                    path.clone() + Path::from_components(&[PathComponent::Key(key.clone())]);
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
    let path: Path = Path::new();
    let json_value: Value = serde_json::from_reader(json_reader)?;
    let it = to_path_iter(path, &json_value);
    for (path, value) in it {
        println!("{path} = {value}");
    }

    Ok(())
}
