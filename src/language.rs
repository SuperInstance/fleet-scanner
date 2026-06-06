use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    Rust,
    Go,
    Python,
    C,
    TypeScript,
    Unknown,
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Language::Rust => write!(f, "Rust"),
            Language::Go => write!(f, "Go"),
            Language::Python => write!(f, "Python"),
            Language::C => write!(f, "C"),
            Language::TypeScript => write!(f, "TypeScript"),
            Language::Unknown => write!(f, "Unknown"),
        }
    }
}

pub fn detect_language(path: &Path) -> Language {
    let mut counts = [(Language::Rust, 0u32), (Language::Go, 0), (Language::Python, 0), (Language::C, 0), (Language::TypeScript, 0)];

    for entry in walkdir::WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            if let Some(ext) = entry.path().extension().and_then(|e| e.to_str()) {
                match ext {
                    "rs" => counts[0].1 += 1,
                    "go" => counts[1].1 += 1,
                    "py" => counts[2].1 += 1,
                    "c" | "h" => counts[3].1 += 1,
                    "ts" | "tsx" => counts[4].1 += 1,
                    _ => {}
                }
            }
    }

    counts.sort_by(|a, b| b.1.cmp(&a.1));
    if counts[0].1 > 0 {
        counts[0].0
    } else {
        Language::Unknown
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_detect_rust() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();
        fs::write(dir.path().join("lib.rs"), "pub fn hello() {}").unwrap();
        assert_eq!(detect_language(dir.path()), Language::Rust);
    }

    #[test]
    fn test_detect_go() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("go.mod"), "module test\n").unwrap();
        fs::write(dir.path().join("main.go"), "package main").unwrap();
        assert_eq!(detect_language(dir.path()), Language::Go);
    }

    #[test]
    fn test_detect_python() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("main.py"), "def hello(): pass").unwrap();
        assert_eq!(detect_language(dir.path()), Language::Python);
    }

    #[test]
    fn test_detect_typescript() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("index.ts"), "export const x = 1;").unwrap();
        assert_eq!(detect_language(dir.path()), Language::TypeScript);
    }

    #[test]
    fn test_detect_c() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("main.c"), "int main() {}").unwrap();
        assert_eq!(detect_language(dir.path()), Language::C);
    }
}
