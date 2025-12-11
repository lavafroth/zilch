use serde::{Deserialize, Serialize};

#[derive(Serialize, Clone, Deserialize, Debug)]
pub struct PackageName {
    pub id: String,
    pub name: Option<String>,
}

pub fn many_from(s: &str) -> Vec<(String, String)> {
    s.lines()
        .map(|line| line.strip_prefix("package:").unwrap_or(line).to_string())
        .map(|line| match line.rsplit_once("=") {
            Some((path, id)) => (id.to_string(), path.to_string()),
            None => (line, String::default()),
        })
        .collect()
}
