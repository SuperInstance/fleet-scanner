# Fleet Scanner

A **CLI health-check tool** that traverses a directory of Git repositories, classifies each by primary language, measures test density, CI presence, and documentation completeness, then produces a weighted health score (0–100) and formatted report.

## Why It Matters

A fleet of 500+ repositories decays without systematic inspection. Repositories lose CI configs, test coverage drifts, and new repos arrive without READMEs. Fleet Scanner automates the audit loop: run it over a monorepo parent directory and get an immediately actionable table showing which repos need attention. The health scoring function is deliberately transparent — each pillar contributes 20 points — so teams know exactly what to fix. The JSON output mode enables integration with dashboards and CI gates.

## How It Works

**Language detection** walks the directory tree using `walkdir`, counting files by extension (`.rs` → Rust, `.go` → Go, `.py` → Python, `.c`/`.h` → C, `.ts`/`.tsx` → TypeScript). The language with the most files wins. Time complexity: O(F) where F is total file count per repo.

**Test counting** scans source files for language-specific test markers:
- Rust: `#[test]` attribute annotations
- Go: `func Test*` function declarations
- Python: `def test*` function definitions
- TypeScript: `it(`, `test(`, `describe(` calls

**Test density** is computed as `test_count / file_count`. A density ≥ 0.1 (at least 1 test per 10 files) earns full marks; any non-zero density earns half.

**Health score** is a sum of five binary/threshold checks:

| Pillar | Condition | Points |
|--------|-----------|--------|
| README | File starting with "readme" exists | 20 |
| Tests | At least one test file found | 20 |
| Test density | `density ≥ 0.1` → 20, `density > 0` → 10 | 20 |
| CI | `.github/workflows/`, `.gitlab-ci.yml`, `Jenkinsfile`, or `.circleci/` exists | 20 |
| License | File starting with "license" or "copying" exists | 20 |

The summary aggregates by language, flags repos scoring < 40 as "needs attention," and optionally ranks the top-N healthiest repos.

## Quick Start

```bash
cargo install fleet-scanner
fleet-scanner ~/repos --top 10
fleet-scanner ~/repos --format json | jq '.summary.average_health'
```

## API

| Module | Type | Description |
|--------|------|-------------|
| `language::detect_language(path)` | fn | Detect primary language by file extension counting |
| `report::RepoReport` | struct | Per-repo health data (name, language, scores, flags) |
| `report::ScanReport` | struct | Full scan results with summary |
| `report::compute_health_score(...)` | fn | Calculate 0–100 score from five pillars |
| `report::count_tests(path)` | fn | Count test functions across all source files |

## Architecture Notes

Fleet Scanner is a diagnostic tool in the SuperInstance observability layer. It provides the ground-truth health metrics that feed into fleet-status dashboards. The scoring rubric maps to the **η** (reflex) side of **γ + η = C**: automated inspection replaces manual review, converting coordination cost into tooling. See [Architecture](https://github.com/SuperInstance/SuperInstance/blob/main/ARCHITECTURE.md).

**Output formats**: The CLI supports two output modes via `--format`:
- `table` (default): Rich Unicode table with colored health scores (green ≥ 80, yellow ≥ 40, red < 40), summary panel with language breakdown, and needs-attention flags
- `json`: Machine-readable structured output suitable for piping to `jq` or feeding into dashboards

**Use in CI**: Add `fleet-scanner ~/repos --format json | jq '[.summary.needs_attention | length]'` as a CI gate to block deploys when repos fall below health threshold.

## References

- walkdir crate: Klabnik, S. "Recursive Directory Walking in Rust." https://docs.rs/walkdir
- clap derive macros: https://docs.rs/clap/latest/clap/_derive/
- Cohen, D. "Software Repository Health Metrics," IEEE Software (2021).
- Munaiah, N. et al. "Curating GitHub for Engineered Software Projects," EMSE (2017).

## License

MIT
