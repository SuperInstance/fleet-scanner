use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoReport {
    pub name: String,
    pub path: String,
    pub language: String,
    pub file_count: u32,
    pub test_count: u32,
    pub has_readme: bool,
    pub has_license: bool,
    pub has_ci: bool,
    pub health_score: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanSummary {
    pub total_repos: usize,
    pub by_language: HashMap<String, usize>,
    pub average_health: f64,
    pub needs_attention: Vec<String>,
    pub top_repos: Vec<RepoReport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanReport {
    pub repos: Vec<RepoReport>,
    pub summary: ScanSummary,
}

pub fn count_files(path: &Path) -> u32 {
    walkdir::WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .count() as u32
}

pub fn count_tests(path: &Path) -> u32 {
    let mut count = 0u32;
    for entry in walkdir::WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() {
            continue;
        }
        let ext = entry.path().extension().and_then(|e| e.to_str()).unwrap_or("");
        if !matches!(ext, "rs" | "go" | "py" | "ts" | "tsx") {
            continue;
        }
        if let Ok(content) = std::fs::read_to_string(entry.path()) {
            count += count_tests_in_content(&content, ext);
        }
    }
    count
}

fn count_tests_in_content(content: &str, ext: &str) -> u32 {
    match ext {
        "rs" => content.lines().filter(|l| l.trim().starts_with("#[test]")).count() as u32,
        "go" => content.lines().filter(|l| l.contains("func Test")).count() as u32,
        "py" => content.lines().filter(|l| l.trim().starts_with("def test") || l.trim().starts_with("async def test")).count() as u32,
        "ts" | "tsx" => content.lines().filter(|l| l.contains("it(") || l.contains("test(") || l.contains("describe(")).count() as u32,
        _ => 0,
    }
}

pub fn has_file_starting_with(path: &Path, prefix: &str) -> bool {
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                let lower = name.to_lowercase();
                if lower.starts_with(&prefix.to_lowercase()) {
                    return true;
                }
            }
        }
    }
    false
}

pub fn has_readme(path: &Path) -> bool {
    has_file_starting_with(path, "readme")
}

pub fn has_license(path: &Path) -> bool {
    has_file_starting_with(path, "license") || has_file_starting_with(path, "copying")
}

pub fn has_ci(path: &Path) -> bool {
    let github = path.join(".github/workflows");
    if github.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&github) {
            if entries.count() > 0 {
                return true;
            }
        }
    }
    let gitlab = path.join(".gitlab-ci.yml");
    if gitlab.exists() {
        return true;
    }
    let jenkins = path.join("Jenkinsfile");
    if jenkins.exists() {
        return true;
    }
    let circle = path.join(".circleci");
    if circle.is_dir() {
        return true;
    }
    false
}

pub fn compute_health_score(
    has_readme: bool,
    has_tests: bool,
    test_density: f64,
    has_ci: bool,
    has_license: bool,
) -> u32 {
    let mut score = 0u32;
    if has_readme { score += 20; }
    if has_tests { score += 20; }
    if test_density >= 0.1 { score += 20; } else if test_density > 0.0 { score += 10; }
    if has_ci { score += 20; }
    if has_license { score += 20; }
    score.min(100)
}

pub fn compute_summary(repos: &[RepoReport], top_n: Option<usize>) -> ScanSummary {
    let mut by_language: HashMap<String, usize> = HashMap::new();
    for repo in repos {
        *by_language.entry(repo.language.clone()).or_insert(0) += 1;
    }

    let average_health = if repos.is_empty() {
        0.0
    } else {
        repos.iter().map(|r| r.health_score as f64).sum::<f64>() / repos.len() as f64
    };

    let needs_attention: Vec<String> = repos
        .iter()
        .filter(|r| r.health_score < 40)
        .map(|r| r.name.clone())
        .collect();

    let mut sorted: Vec<RepoReport> = repos.to_vec();
    sorted.sort_by(|a, b| b.health_score.cmp(&a.health_score));
    let limit = top_n.unwrap_or(sorted.len());
    let top_repos = sorted.into_iter().take(limit).collect();

    ScanSummary {
        total_repos: repos.len(),
        by_language,
        average_health: (average_health * 100.0).round() / 100.0,
        needs_attention,
        top_repos,
    }
}
