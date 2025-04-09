use serde::{Deserialize, Serialize};

#[derive(Serialize, Clone, Deserialize, Debug)]
pub struct Package {
    pub id: String,
    #[serde(skip)]
    pub path: String,
    pub name: Option<String>,
}

impl PartialEq for Package {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Package {}

impl PartialOrd for Package {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Package {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

impl Package {
    pub fn many_from(s: &str) -> Vec<Self> {
        s.lines()
            .map(|line| line.strip_prefix("package:").unwrap_or(line).to_string())
            .map(|line| match line.rsplit_once("=") {
                Some((path, id)) => Package {
                    name: None,
                    id: id.to_string(),
                    path: path.to_string(),
                },
                None => Package {
                    name: None,
                    id: line,
                    path: String::default(),
                },
            })
            .collect()
    }
}
