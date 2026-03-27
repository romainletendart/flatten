use std::fmt::Display;

#[derive(Clone, Debug, PartialEq)]
pub enum PathComponent {
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

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Path {
    components: Vec<PathComponent>,
}

impl Path {
    #[must_use]
    pub fn new(components: &[PathComponent]) -> Self {
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
