# fleet-scanner

A CLI tool that scans a directory of git repositories and produces a structured health report. It detects the primary programming language, counts source files and test functions, checks for README/LICENSE/CI configuration, and computes a composite health score (0–100). Output is available as a colorized terminal table or machine-readable JSON.

## Why It Matters

Fleet management requires visibility. When you operate dozens of repositories — as in the SuperInstance fleet — manual auditing is infeasible. fleet-scanner automates the audit: one command produces a complete inventory with health scores, language distribution, test density, and CI/license compliance. This makes it possible to identify neglected repos, track improvement over time, and enforce minimum standards across the fleet.

## How It Works

### Language Detection

The scanner walks the entire directory tree and counts files by extension:

| Extension | Language |
|-----------|----------|
| `.rs` | Rust |
| `.go` | Go |
| `.py` | Python |
| `.c`, `.h` | C |
| `.ts`, `.tsx` | TypeScript |

The language with the most files wins (plurality vote). This is O(F) where F = total files across all repos.

**Time complexity**: O(F) per repo for language detection, O(F) for file counting, O(F · L_avg) for test counting where L_avg is average lines per file.

### Test Counting Heuristics

Test functions are detected by language-specific pattern matching:

- **Rust**: `#[test]` attribute lines
- **Go**: `func Test` function declarations
- **Python**: `def test` or `async def test` prefixes
- **TypeScript**: `it(`, `test(`, `describe(` call sites

### Health Score Model

The composite score is a weighted sum of five binary/dense signals:

```
Score = 20·R + 20·T + 20·D + 20·CI + 20·L
```

where:
- **R** ∈ {0, 1}: Has README
- **T** ∈ {0, 1}: Has at least one test
- **D** ∈ {0, 0.5, 1}: Test density (`tests/files ≥ 0.1` → 1.0, `> 0` → 0.5, else 0)
- **CI** ∈ {0, 1}: Has CI config (GitHub Actions, GitLab CI, Jenkins, or CircleCI)
- **L** ∈ {0, 1}: Has LICENSE or COPYING file

The score is clamped to [0, 100]. Repos scoring < 40 are flagged as "needs attention."

### Statistical Aggregation

The summary computes:
- **Average health**: arithmetic mean across all repos
- **Language distribution**: frequency count by detected language
- **Needs attention**: filter `score < 40`

**Time complexity**: O(R · F_avg) total where R = number of repos, F_avg = average files per repo.

**Space complexity**: O(R) for the report structures.

## Quick Start

```bash
# Build
cargo build --release

# Scan a directory
./target/release/fleet-scanner /path/to/repos

# Top 5 healthiest
./target/release/fleet-scanner /path/to/repos --top 5

# JSON output for CI integration
./target/release/fleet-scanner /path/to/repos --format json
```

## API

### Library Usage

```rust
use fleet_scanner::{language::detect_language, report::*};

// Detect primary language
let lang = detect_language(&path);

// Build a repo report
let report = RepoReport {
    name: "my-repo".into(),
    path: "/repos/my-repo".into(),
    language: lang.to_string(),
    file_count: count_files(&path),
    test_count: count_tests(&path),
    has_readme: has_readme(&path),
    has_license: has_license(&path),
    has_ci: has_ci(&path),
    health_score: compute_health_score(true, true, 0.15, true, true),
};
```

### CLI Flags

| Flag | Description |
|------|-------------|
| `--top N` | Show only top N healthiest repos |
| `--format json\|table` | Output format (default: table) |

### Report Schema (JSON)

```json
{
  "repos": [{ "name", "path", "language", "file_count", "test_count",
              "has_readme", "has_license", "has_ci", "health_score" }],
  "summary": { "total_repos", "by_language", "average_health",
               "needs_attention", "top_repos" }
}
```

## Architecture Notes

fleet-scanner is the **auditor (γ)** that closes the quality-assurance loop for the fleet. The repos are the **system (η)** under inspection. The health score is the **control signal (C)** that tells operators where to invest maintenance effort. Without this feedback, repos degrade silently — missing tests, absent CI, expired licenses accumulate. The scanner makes the invisible visible, which is the prerequisite for systematic improvement.

### Module Structure

```
src/
├── main.rs       — CLI parsing, table rendering, orchestration
├── language.rs   — Language detection (extension counting + plurality)
└── report.rs     — Data structures, health scoring, file/test counting
```

## References

- **Software quality metrics**: ISO/IEC 25010:2011, *Systems and software Quality Requirements and Evaluation (SQuaRE).*
- **Test density thresholds**: Williams, L., et al. "Test-driven development assessment." *MSR '04*, ACM, 2004.
- **Repository health scoring**: Munaiah, N., et al. "Relevance of repository metrics: A case study." *IEEE/ACM MSR*, 2017.

## License

MIT
