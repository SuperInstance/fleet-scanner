use clap::{Parser, ValueEnum};
use colored::*;
use comfy_table::{modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, Table, ContentArrangement, Cell, Color as TableColor};
use std::path::PathBuf;

use fleet_scanner::{language::detect_language, report::*};

#[derive(Parser)]
#[command(name = "fleet-scanner")]
#[command(about = "Scan a directory of git repositories and produce a health report")]
#[command(version)]
struct Cli {
    /// Directory to scan for git repositories
    path: PathBuf,

    /// Show top N healthiest repos
    #[arg(long, short)]
    top: Option<usize>,

    /// Output format
    #[arg(long, short, default_value = "table")]
    format: OutputFormat,
}

#[derive(Clone, ValueEnum)]
enum OutputFormat {
    Json,
    Table,
}

fn find_git_repos(path: &PathBuf) -> Vec<PathBuf> {
    let mut repos = Vec::new();
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let entry_path = entry.path();
            if entry_path.is_dir() && entry_path.join(".git").exists() {
                repos.push(entry_path);
            }
        }
    }
    repos.sort();
    repos
}

fn scan_repo(path: &PathBuf) -> RepoReport {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let lang = detect_language(path);
    let file_count = count_files(path);
    let test_count = count_tests(path);
    let has_r = has_readme(path);
    let has_l = has_license(path);
    let has_c = has_ci(path);
    let has_tests = test_count > 0;
    let test_density = if file_count > 0 { test_count as f64 / file_count as f64 } else { 0.0 };
    let health_score = compute_health_score(has_r, has_tests, test_density, has_c, has_l);

    RepoReport {
        name,
        path: path.to_string_lossy().to_string(),
        language: lang.to_string(),
        file_count,
        test_count,
        has_readme: has_r,
        has_license: has_l,
        has_ci: has_c,
        health_score,
    }
}

fn score_color(score: u32) -> Color {
    if score >= 80 { Color::Green }
    else if score >= 40 { Color::Yellow }
    else { Color::Red }
}

fn score_table_color(score: u32) -> TableColor {
    if score >= 80 { TableColor::Green }
    else if score >= 40 { TableColor::Yellow }
    else { TableColor::Red }
}

fn print_table(report: &ScanReport) {
    let ScanReport { repos, summary } = report;

    // Repo table
    let mut table = Table::new();
    table.load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("Repo"), Cell::new("Language"), Cell::new("Files"),
            Cell::new("Tests"), Cell::new("README"), Cell::new("CI"),
            Cell::new("License"), Cell::new("Score"),
        ]);

    for repo in repos {
        let check = "✓".to_string();
        let cross = "✗".to_string();
        table.add_row(vec![
            Cell::new(&repo.name),
            Cell::new(&repo.language),
            Cell::new(repo.file_count),
            Cell::new(repo.test_count),
            Cell::new(if repo.has_readme { &check } else { &cross }),
            Cell::new(if repo.has_ci { &check } else { &cross }),
            Cell::new(if repo.has_license { &check } else { &cross }),
            Cell::new(repo.health_score).fg(score_table_color(repo.health_score)),
        ]);
    }

    println!("{}", table);

    // Summary
    println!("\n{}", "📊 Summary".bold().cyan());
    println!("  Total repos: {}", summary.total_repos.to_string().bold());
    println!("  Average health: {}", format!("{:.1}", summary.average_health).bold());
    
    if !summary.by_language.is_empty() {
        println!("  By language:");
        let mut langs: Vec<_> = summary.by_language.iter().collect();
        langs.sort_by(|a, b| b.1.cmp(a.1));
        for (lang, count) in langs {
            println!("    {} {}", count.to_string().bold(), lang);
        }
    }

    if !summary.needs_attention.is_empty() {
        println!("\n  {} {}", "⚠ ".yellow(), "Needs attention (score < 40):".yellow().bold());
        for name in &summary.needs_attention {
            println!("    {} {}", "•".red(), name.red());
        }
    }
}

fn main() {
    let cli = Cli::parse();

    if !cli.path.exists() {
        eprintln!("{} Path does not exist: {}", "Error:".red().bold(), cli.path.display());
        std::process::exit(1);
    }

    if !cli.path.is_dir() {
        eprintln!("{} Path is not a directory: {}", "Error:".red().bold(), cli.path.display());
        std::process::exit(1);
    }

    let repos = find_git_repos(&cli.path);
    if repos.is_empty() {
        eprintln!("{} No git repositories found in {}", "Warning:".yellow().bold(), cli.path.display());
        std::process::exit(0);
    }

    let repo_reports: Vec<RepoReport> = repos.iter().map(|r| scan_repo(r)).collect();
    let summary = compute_summary(&repo_reports, cli.top);
    let report = ScanReport { repos: repo_reports, summary };

    match cli.format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&report).unwrap());
        }
        OutputFormat::Table => {
            print_table(&report);
        }
    }
}
