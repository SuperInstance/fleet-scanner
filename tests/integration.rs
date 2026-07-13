#[cfg(test)]
mod tests {
    use fleet_scanner::*;
    use std::fs;
    use std::process::Command;
    use tempfile::TempDir;

    // --- language detection ---

    #[test]
    fn detect_rust() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("main.rs"), "fn main() {}").unwrap();
        assert_eq!(
            language::detect_language(dir.path()),
            language::Language::Rust
        );
    }

    #[test]
    fn detect_go() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("main.go"), "package main").unwrap();
        assert_eq!(
            language::detect_language(dir.path()),
            language::Language::Go
        );
    }

    #[test]
    fn detect_python() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("app.py"), "print('hi')").unwrap();
        assert_eq!(
            language::detect_language(dir.path()),
            language::Language::Python
        );
    }

    #[test]
    fn detect_typescript() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("index.ts"), "console.log('hi')").unwrap();
        assert_eq!(
            language::detect_language(dir.path()),
            language::Language::TypeScript
        );
    }

    #[test]
    fn detect_c() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("main.c"), "int main() {}").unwrap();
        assert_eq!(language::detect_language(dir.path()), language::Language::C);
    }

    #[test]
    fn detect_unknown_when_empty() {
        let dir = TempDir::new().unwrap();
        assert_eq!(
            language::detect_language(dir.path()),
            language::Language::Unknown
        );
    }

    // --- file counting ---

    #[test]
    fn count_files_in_dir() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("a.rs"), "").unwrap();
        fs::write(dir.path().join("b.rs"), "").unwrap();
        fs::create_dir(dir.path().join("src")).unwrap();
        fs::write(dir.path().join("src/c.rs"), "").unwrap();
        assert_eq!(report::count_files(dir.path()), 3);
    }

    #[test]
    fn count_files_excludes_git() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("main.rs"), "").unwrap();
        let git_objects = dir.path().join(".git/objects/ab");
        fs::create_dir_all(&git_objects).unwrap();
        fs::write(git_objects.join("deadbeef"), "blob").unwrap();
        assert_eq!(report::count_files(dir.path()), 1);
    }

    // --- test counting ---

    #[test]
    fn count_rust_tests() {
        let dir = TempDir::new().unwrap();
        fs::write(
            dir.path().join("lib.rs"),
            "#[test]\nfn test_one() {}\n#[test]\nfn test_two() {}",
        )
        .unwrap();
        assert_eq!(report::count_tests(dir.path()), 2);
    }

    #[test]
    fn count_go_tests() {
        let dir = TempDir::new().unwrap();
        fs::write(
            dir.path().join("main_test.go"),
            "func TestSomething(t *testing.T) {}",
        )
        .unwrap();
        assert_eq!(report::count_tests(dir.path()), 1);
    }

    #[test]
    fn count_python_tests() {
        let dir = TempDir::new().unwrap();
        fs::write(
            dir.path().join("test_app.py"),
            "def test_one(): pass\ndef test_two(): pass",
        )
        .unwrap();
        assert_eq!(report::count_tests(dir.path()), 2);
    }

    #[test]
    fn count_typescript_tests() {
        let dir = TempDir::new().unwrap();
        fs::write(
            dir.path().join("app.test.ts"),
            "it('works', () => {});\ntest('also', () => {});",
        )
        .unwrap();
        assert_eq!(report::count_tests(dir.path()), 2);
    }

    #[test]
    fn count_tests_excludes_git() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("lib.rs"), "#[test]\nfn real() {}").unwrap();
        let git_dir = dir.path().join(".git");
        fs::create_dir_all(&git_dir).unwrap();
        fs::write(git_dir.join("fake.rs"), "#[test]\nfn fake() {}").unwrap();
        assert_eq!(report::count_tests(dir.path()), 1);
    }

    // --- has_readme / has_license / has_ci ---

    #[test]
    fn detects_readme() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("README.md"), "# Hello").unwrap();
        assert!(report::has_readme(dir.path()));
    }

    #[test]
    fn detects_readme_case_insensitive() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("readme"), "info").unwrap();
        assert!(report::has_readme(dir.path()));
    }

    #[test]
    fn no_readme() {
        let dir = TempDir::new().unwrap();
        assert!(!report::has_readme(dir.path()));
    }

    #[test]
    fn detects_license() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("LICENSE"), "MIT").unwrap();
        assert!(report::has_license(dir.path()));
    }

    #[test]
    fn detects_copying_as_license() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("COPYING"), "GPL").unwrap();
        assert!(report::has_license(dir.path()));
    }

    #[test]
    fn detects_github_ci() {
        let dir = TempDir::new().unwrap();
        let workflows = dir.path().join(".github/workflows");
        fs::create_dir_all(&workflows).unwrap();
        fs::write(workflows.join("ci.yml"), "name: CI").unwrap();
        assert!(report::has_ci(dir.path()));
    }

    #[test]
    fn detects_gitlab_ci() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join(".gitlab-ci.yml"), "stages: []").unwrap();
        assert!(report::has_ci(dir.path()));
    }

    #[test]
    fn detects_jenkins_ci() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("Jenkinsfile"), "").unwrap();
        assert!(report::has_ci(dir.path()));
    }

    #[test]
    fn detects_circleci() {
        let dir = TempDir::new().unwrap();
        let circle = dir.path().join(".circleci");
        fs::create_dir_all(&circle).unwrap();
        fs::write(circle.join("config.yml"), "").unwrap();
        assert!(report::has_ci(dir.path()));
    }

    #[test]
    fn empty_circleci_is_not_ci() {
        let dir = TempDir::new().unwrap();
        fs::create_dir(dir.path().join(".circleci")).unwrap();
        assert!(!report::has_ci(dir.path()));
    }

    #[test]
    fn no_ci() {
        let dir = TempDir::new().unwrap();
        assert!(!report::has_ci(dir.path()));
    }

    // --- health score ---

    #[test]
    fn perfect_health() {
        let score = report::compute_health_score(true, true, 0.2, true, true);
        assert_eq!(score, 100);
    }

    #[test]
    fn zero_health() {
        let score = report::compute_health_score(false, false, 0.0, false, false);
        assert_eq!(score, 0);
    }

    #[test]
    fn partial_health_low_density() {
        // has readme + tests but low density => 20 + 20 + 10 = 50
        let score = report::compute_health_score(true, true, 0.01, false, false);
        assert_eq!(score, 50);
    }

    // --- summary ---

    #[test]
    fn summary_needs_attention() {
        let repos = vec![
            report::RepoReport {
                name: "bad".into(),
                path: "/bad".into(),
                language: "Rust".into(),
                file_count: 1,
                test_count: 0,
                has_readme: false,
                has_license: false,
                has_ci: false,
                health_score: 10,
            },
            report::RepoReport {
                name: "good".into(),
                path: "/good".into(),
                language: "Go".into(),
                file_count: 10,
                test_count: 5,
                has_readme: true,
                has_license: true,
                has_ci: true,
                health_score: 100,
            },
        ];
        let summary = report::compute_summary(&repos, None);
        assert_eq!(summary.total_repos, 2);
        assert_eq!(summary.needs_attention, vec!["bad"]);
        assert_eq!(summary.average_health, 55.0);
    }

    #[test]
    fn summary_top_n() {
        let repos = vec![
            report::RepoReport {
                name: "a".into(),
                path: "/a".into(),
                language: "Rust".into(),
                file_count: 1,
                test_count: 0,
                has_readme: true,
                has_license: false,
                has_ci: false,
                health_score: 20,
            },
            report::RepoReport {
                name: "b".into(),
                path: "/b".into(),
                language: "Go".into(),
                file_count: 10,
                test_count: 5,
                has_readme: true,
                has_license: true,
                has_ci: true,
                health_score: 100,
            },
        ];
        let summary = report::compute_summary(&repos, Some(1));
        assert_eq!(summary.top_repos.len(), 1);
        assert_eq!(summary.top_repos[0].name, "b");
    }

    #[test]
    fn summary_by_language() {
        let repos = vec![
            report::RepoReport {
                name: "a".into(),
                path: "/a".into(),
                language: "Rust".into(),
                file_count: 1,
                test_count: 0,
                has_readme: false,
                has_license: false,
                has_ci: false,
                health_score: 0,
            },
            report::RepoReport {
                name: "b".into(),
                path: "/b".into(),
                language: "Rust".into(),
                file_count: 1,
                test_count: 0,
                has_readme: false,
                has_license: false,
                has_ci: false,
                health_score: 0,
            },
            report::RepoReport {
                name: "c".into(),
                path: "/c".into(),
                language: "Go".into(),
                file_count: 1,
                test_count: 0,
                has_readme: false,
                has_license: false,
                has_ci: false,
                health_score: 0,
            },
        ];
        let summary = report::compute_summary(&repos, None);
        assert_eq!(*summary.by_language.get("Rust").unwrap(), 2);
        assert_eq!(*summary.by_language.get("Go").unwrap(), 1);
    }

    // --- CLI ---

    fn bin() -> &'static str {
        env!("CARGO_BIN_EXE_fleet-scanner")
    }

    fn make_repo(dir: &std::path::Path, name: &str, perfect: bool) {
        let repo = dir.join(name);
        fs::create_dir_all(&repo).unwrap();
        fs::create_dir(repo.join(".git")).unwrap();
        if perfect {
            fs::write(repo.join("README.md"), "# test").unwrap();
            fs::write(repo.join("LICENSE"), "MIT").unwrap();
            fs::create_dir_all(repo.join(".github/workflows")).unwrap();
            fs::write(repo.join(".github/workflows/ci.yml"), "name: CI").unwrap();
            fs::create_dir(repo.join("src")).unwrap();
            fs::write(repo.join("src/main.rs"), "fn main() {}").unwrap();
            fs::create_dir(repo.join("tests")).unwrap();
            fs::write(repo.join("tests/lib.rs"), "#[test]\nfn t() {}").unwrap();
        }
    }

    #[test]
    fn cli_rejects_missing_path() {
        let output = Command::new(bin())
            .arg("/definitely/does/not/exist")
            .output()
            .unwrap();
        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("Path does not exist"));
    }

    #[test]
    fn cli_rejects_non_directory_path() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("not-a-dir");
        fs::write(&file, "").unwrap();
        let output = Command::new(bin()).arg(&file).output().unwrap();
        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("Path is not a directory"));
    }

    #[test]
    fn cli_warns_when_no_repos() {
        let dir = TempDir::new().unwrap();
        let output = Command::new(bin()).arg(dir.path()).output().unwrap();
        assert!(output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("No git repositories found"));
    }

    #[test]
    fn cli_json_output() {
        let dir = TempDir::new().unwrap();
        make_repo(dir.path(), "good", true);
        let output = Command::new(bin())
            .arg(dir.path())
            .arg("--format")
            .arg("json")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let report: report::ScanReport =
            serde_json::from_slice(&output.stdout).expect("valid JSON");
        assert_eq!(report.summary.total_repos, 1);
        assert_eq!(report.repos[0].name, "good");
        assert_eq!(report.repos[0].health_score, 100);
    }

    #[test]
    fn cli_top_n_limits_results() {
        let dir = TempDir::new().unwrap();
        make_repo(dir.path(), "bad", false);
        make_repo(dir.path(), "good", true);
        let output = Command::new(bin())
            .arg(dir.path())
            .arg("--format")
            .arg("json")
            .arg("--top")
            .arg("1")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let report: report::ScanReport =
            serde_json::from_slice(&output.stdout).expect("valid JSON");
        assert_eq!(report.summary.total_repos, 2);
        assert_eq!(report.summary.top_repos.len(), 1);
        assert_eq!(report.summary.top_repos[0].name, "good");
    }
}
