use std::fmt::Display;

pub struct Location {
    name: String,
    description: String,
    things: Vec<String>,
    paths: Vec<Location>,
}

impl Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "== {} ==", self.name)?;
        write!(f, "{}", self.description)
    }
}

impl Location {
    pub fn new(name: String, description: String) -> Self {
        Self {
            name,
            description,
            things: vec![],
            paths: vec![],
        }
    }
}
