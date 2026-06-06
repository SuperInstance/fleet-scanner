# fleet-scanner

CLI tool for scanning a directory of git repositories and producing a health report. Built for managing large open-source fleets.

## Features

- **Language Detection** — Auto-detect Rust, Go, Python, C, TypeScript by file extension analysis
- **Health Scoring** — 0-100 based on README, tests, CI, license presence
- **Fleet Summary** — Aggregate statistics across all repos
- **JSON Output** — Machine-readable reports for automation
- **Table Output** — Human-readable formatted tables
- **Top N Filter** — Show best (or worst) scoring repos

## Installation

```bash
cargo install --git https://github.com/SuperInstance/fleet-scanner
```

## Usage

```bash
# Scan current directory
fleet-scanner scan .

# Show top 10 healthiest repos
fleet-scanner scan ~/repos --top 10

# JSON output for scripting
fleet-scanner scan ~/repos --format json > fleet-report.json

# Filter by language
fleet-scanner scan ~/repos --language rust
```

### Health Score Breakdown

| Criteria | Points |
|----------|--------|
| Has README | +20 |
| Has tests | +20 |
| Test density (files/test ratio) | +20 |
| Has CI configuration | +20 |
| Has LICENSE | +20 |

## Output Example

```
📊 Fleet Composition:
  rust        : 493 (83.3%)
  python      :  29 ( 4.9%)
  typescript  :  10 ( 1.7%)

📋 Quality Metrics:
  README:   581/592 (98%)
  Tests:    557/592 (94%)
  CI/CD:     40/592 (7%)
```

## Testing

```bash
cargo test    # 24 tests
```

## License

MIT
