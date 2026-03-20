use std::io::Read;

use anyhow::Error;
use serde_json::Value;

use flatten::Path;
use flatten::PathComponent;

pub struct PathValue {
    path: Path,
    value: Value,
}

impl PathValue {
    #[must_use]
    fn new(path: Path, value: Value) -> Self {
        Self { path, value }
    }
}

impl From<PathValue> for (Path, Value) {
    fn from(value: PathValue) -> Self {
        (value.path, value.value)
    }
}

impl IntoIterator for PathValue {
    type Item = Self;
    type IntoIter = Box<dyn Iterator<Item = Self>>;

    fn into_iter(self) -> Self::IntoIter {
        match self.value {
            Value::Array(array) => {
                let it = array
                    .into_iter()
                    .enumerate()
                    .flat_map(move |(index, value)| {
                        let new_path =
                            self.path.clone() + Path::new(&[PathComponent::Index(index)]);
                        PathValue::new(new_path, value.clone()).into_iter()
                    });
                Box::new(it)
            }
            Value::Object(map) => {
                let it = map.into_iter().flat_map(move |(key, value)| {
                    let new_path =
                        self.path.clone() + Path::new(&[PathComponent::Key(key.clone())]);
                    PathValue::new(new_path, value.clone()).into_iter()
                });
                Box::new(it)
            }
            _ => Box::new([PathValue::new(self.path.clone(), self.value.clone())].into_iter()),
        }
    }
}

pub struct Stream {
    path_value_iterator: Box<dyn Iterator<Item = PathValue>>,
}

impl Stream {
    pub fn new<R: Read>(reader: R) -> Result<Self, Error> {
        let path: Path = Path::default();
        let value: Value = serde_json::from_reader(reader)?;
        let path_value_iterator = PathValue::new(path, value).into_iter();

        Ok(Self {
            path_value_iterator,
        })
    }
}

impl Iterator for Stream {
    type Item = PathValue;

    fn next(&mut self) -> Option<Self::Item> {
        self.path_value_iterator.next()
    }
}
