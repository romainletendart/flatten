use std::io::BufRead;

use serde_json::Value;
use thiserror::Error;

use flatten::Path;
use flatten::PathComponent;
use flatten::Scalar;

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
    type Item = (Path, Scalar);
    type IntoIter = Box<dyn Iterator<Item = (Path, Scalar)>>;

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
            _ => Box::new([(self.path, Scalar(self.value.to_string()))].into_iter()),
        }
    }
}

#[derive(Error, Debug)]
pub enum StreamError {
    #[error("IO error")]
    Io(#[from] std::io::Error),

    #[error("Could not parse input JSON")]
    Parsing(#[from] serde_json::Error),
}

pub struct Stream {
    path_value_iterator: Box<dyn Iterator<Item = (Path, Scalar)>>,
}

impl Stream {
    pub fn new<R: BufRead>(mut reader: R) -> Result<Self, StreamError> {
        let path: Path = Path::default();
        let path_value_iterator = {
            if reader.fill_buf()?.is_empty() {
                // An empty stream should result in an empty iterator.
                Box::new(Vec::new().into_iter())
            } else {
                let value: Value = serde_json::from_reader(reader)?;
                PathValue::new(path, value).into_iter()
            }
        };

        Ok(Self {
            path_value_iterator,
        })
    }
}

impl Iterator for Stream {
    type Item = (Path, Scalar);

    fn next(&mut self) -> Option<Self::Item> {
        self.path_value_iterator.next()
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;
    use std::io::Cursor;

    use test_case::test_case;

    use crate::json::{Path, Scalar, Stream};
    use flatten::PathComponent::{Index, Key};

    macro_rules! key {
        ($e:expr) => {
            Key($e.to_string())
        };
    }

    macro_rules! idx {
        ($e:expr) => {
            Index($e)
        };
    }

    macro_rules! path {
        ($($e:expr),*) => {
            Path::new(&[$($e,)*])
        };
    }

    macro_rules! scalar {
        ($e:expr) => {
            Scalar($e.to_string())
        };
    }

    #[test]
    fn test_empty_json_string_results_in_empty_stream() {
        let reader = Cursor::new("");
        let stream = Stream::new(reader);

        assert!(stream.is_ok());
        assert_eq!(stream.unwrap().collect::<Vec<(Path, Scalar)>>(), Vec::new());
    }

    #[test_case("null")]
    #[test_case("42")]
    #[test_case("6.022e+23")]
    #[test_case("true")]
    #[test_case("false")]
    #[test_case("\"\""; "empty string")]
    #[test_case("\"Hello world\"")]
    fn test_valid_bare_scalar_results_in_valid_stream(json_string: &str) {
        let reader = Cursor::new(json_string);
        let stream = Stream::new(reader);
        let expected: Vec<(Path, Scalar)> = vec![(path![], scalar!(json_string))];

        assert!(stream.is_ok());
        assert_eq!(stream.unwrap().collect::<Vec<(Path, Scalar)>>(), expected);
    }

    #[test]
    fn test_invalid_bare_scalar_results_in_error() {
        let reader = Cursor::new("Invalid JSON");
        let stream = Stream::new(reader);

        assert!(stream.is_err());
    }

    #[test]
    fn test_heterogeneous_value_results_in_valid_stream() {
        let json_string = r#"
{"employees": [
    {"id": 1, "name": "John Doe", "is_manager": true},
    {"id": 2, "name": "Jean Dupont", "is_manager": false}
]}
        "#;
        let reader = Cursor::new(json_string);
        let stream = Stream::new(reader);
        let expected: Vec<(Path, Scalar)> = vec![
            (path![key!("employees"), idx!(0), key!("id")], scalar!("1")),
            (
                path![key!("employees"), idx!(0), key!("name")],
                scalar!("\"John Doe\""),
            ),
            (
                path![key!("employees"), idx!(0), key!("is_manager")],
                scalar!("true"),
            ),
            (path![key!("employees"), idx!(1), key!("id")], scalar!("2")),
            (
                path![key!("employees"), idx!(1), key!("name")],
                scalar!("\"Jean Dupont\""),
            ),
            (
                path![key!("employees"), idx!(1), key!("is_manager")],
                scalar!("false"),
            ),
        ];

        assert!(stream.is_ok());
        assert_eq!(stream.unwrap().collect::<Vec<(Path, Scalar)>>(), expected);
    }
}
