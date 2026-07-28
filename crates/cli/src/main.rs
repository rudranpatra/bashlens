use anyhow::Result;
use bashlens_analyzer::{Analyzer, CorpusBaseline};
use bashlens_report::{BehaviorClass, Report};
use clap::Parser;

#[derive(Parser)]
#[command(
    name = "bashlens",
    about = "Inspect shell install scripts before you run them"
)]
struct Args {
    #[command(subcommand)]
    command: Option<Command>,

    /// URL or local path to the install script (when no subcommand is given)
    target: Option<String>,

    /// Output format
    #[arg(long, value_enum, default_value = "text")]
    format: Format,

    /// Recompute the corpus risk-percentile cache (corpus/stats/baseline.json)
    /// from corpus/scripts/ and exit. Run this after changing the corpus or
    /// the rules - otherwise percentiles quietly drift out of date.
    #[arg(long)]
    update_baseline: bool,

    /// Suppress the full report; print one line and exit non-zero only if
    /// --fail-on matches. Intended for CI (`bashlens --quiet --fail-on
    /// obfuscation,unresolvable <url>`).
    #[arg(long)]
    quiet: bool,

    /// Comma-separated behavior classes that should make bashlens exit
    /// non-zero when present: network, privilege, persistence, verification,
    /// obfuscation, unresolvable.
    #[arg(long, value_delimiter = ',')]
    fail_on: Vec<String>,
}

#[derive(clap::Subcommand)]
enum Command {
    /// Analyze several installers side by side
    Compare {
        /// Two or more URLs or local paths to compare
        #[arg(required = true, num_args = 2..)]
        targets: Vec<String>,
    },
}

#[derive(clap::ValueEnum, Clone)]
enum Format {
    Text,
    Json,
    Markdown,
}

const BASELINE_CACHE_PATH: &str = "corpus/stats/baseline.json";
const CORPUS_DIR: &str = "corpus/scripts";

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let args = Args::parse();

    if args.update_baseline {
        // Maintenance path: always run from the repo root, so read straight
        // from the checkout rather than the embedded (build-time) copy.
        let analyzer = Analyzer::from_rules_dir(std::path::Path::new("rules"))?;
        let baseline = analyzer.build_corpus_baseline(std::path::Path::new(CORPUS_DIR))?;
        baseline.save(BASELINE_CACHE_PATH)?;
        println!(
            "Wrote {} (corpus size {})",
            BASELINE_CACHE_PATH,
            baseline.len()
        );
        return Ok(());
    }

    // Rules are embedded at compile time so the binary works from any
    // working directory - a `cargo install`/Homebrew/`npx` distribution
    // can't assume a `rules/` checkout sits next to it.
    let analyzer = Analyzer::embedded()?;
    let baseline = load_or_build_baseline(&analyzer);

    match args.command {
        Some(Command::Compare { targets }) => run_compare(&analyzer, baseline.as_ref(), &targets),
        None => {
            let target = args
                .target
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("a URL or file path is required"))?;
            run_single(&analyzer, baseline.as_ref(), target, &args)
        }
    }
}

/// Preference order: a local cache file (fastest, and lets a repo checkout
/// pick up a freshly-regenerated baseline without recompiling) -> a live
/// corpus scan if `corpus/scripts/` is present but no cache exists yet
/// (persisting the result so the next run is fast) -> the baseline embedded
/// into the binary at compile time, so percentiles still work when run from
/// somewhere with no corpus checkout at all. Always `Some` in practice.
fn load_or_build_baseline(analyzer: &Analyzer) -> Option<CorpusBaseline> {
    if let Ok(baseline) = CorpusBaseline::load(BASELINE_CACHE_PATH) {
        return Some(baseline);
    }
    let corpus_dir = std::path::Path::new(CORPUS_DIR);
    if corpus_dir.is_dir() {
        if let Ok(baseline) = analyzer.build_corpus_baseline(corpus_dir) {
            let _ = baseline.save(BASELINE_CACHE_PATH);
            return Some(baseline);
        }
    }
    CorpusBaseline::embedded().ok()
}

fn analyze_target(
    analyzer: &Analyzer,
    baseline: Option<&CorpusBaseline>,
    target: &str,
) -> Result<Report> {
    let source = fetch(target)?;
    let mut report = analyzer.analyze(target, &source)?;
    if let Some(baseline) = baseline {
        if !baseline.is_empty() {
            report.risk = Some(analyzer.risk_summary(&report, baseline));
        }
    }
    Ok(report)
}

fn run_single(
    analyzer: &Analyzer,
    baseline: Option<&CorpusBaseline>,
    target: &str,
    args: &Args,
) -> Result<()> {
    let report = analyze_target(analyzer, baseline, target)?;
    let triggered = matched_fail_on_classes(&report, &args.fail_on);

    if args.quiet {
        if triggered.is_empty() {
            println!("bashlens: OK  {target}");
        } else {
            println!("bashlens: FAIL  {target}  ({})", triggered.join(", "));
        }
    } else {
        match args.format {
            Format::Json => println!("{}", serde_json::to_string_pretty(&report)?),
            Format::Markdown => println!("{}", report.markdown()),
            Format::Text => println!("{}", report.text()),
        }
    }

    if !triggered.is_empty() {
        std::process::exit(1);
    }
    Ok(())
}

fn matched_fail_on_classes(report: &Report, fail_on: &[String]) -> Vec<String> {
    if fail_on.is_empty() {
        return Vec::new();
    }
    let present: std::collections::HashSet<String> = report
        .classes
        .iter()
        .map(|c| format!("{c:?}").to_ascii_lowercase())
        .collect();
    fail_on
        .iter()
        .map(|s| s.trim().to_ascii_lowercase())
        .filter(|s| present.contains(s))
        .collect()
}

fn run_compare(
    analyzer: &Analyzer,
    baseline: Option<&CorpusBaseline>,
    targets: &[String],
) -> Result<()> {
    let reports: Vec<Report> = targets
        .iter()
        .map(|t| analyze_target(analyzer, baseline, t))
        .collect::<Result<_>>()?;

    let name_width = targets.iter().map(|t| t.len()).max().unwrap_or(4).max(4) + 2;
    println!(
        "{:<name_width$}{:<15}{:<10}{:<12}{:<10}",
        "", "Verification", "Network", "Persistence", "Obfuscation"
    );
    for report in &reports {
        let verified = report
            .risk
            .as_ref()
            .map(|r| if r.verified { "yes" } else { "no" })
            .unwrap_or("?");
        let pct = |class: BehaviorClass| -> String {
            report
                .risk
                .as_ref()
                .and_then(|r| r.per_class.iter().find(|c| c.class == class))
                .map(|c| ordinal(c.percentile))
                .unwrap_or_else(|| "-".to_string())
        };
        println!(
            "{:<name_width$}{:<15}{:<10}{:<12}{:<10}",
            report.name,
            verified,
            pct(BehaviorClass::Network),
            pct(BehaviorClass::Persistence),
            pct(BehaviorClass::Obfuscation),
        );
    }
    Ok(())
}

fn ordinal(n: u8) -> String {
    let suffix = match (n % 100, n % 10) {
        (11..=13, _) => "th",
        (_, 1) => "st",
        (_, 2) => "nd",
        (_, 3) => "rd",
        _ => "th",
    };
    format!("{n}{suffix}")
}

fn fetch(target: &str) -> Result<String> {
    let bytes = if target.starts_with("http://") || target.starts_with("https://") {
        reqwest::blocking::get(target)?.bytes()?.to_vec()
    } else {
        std::fs::read(target)?
    };
    // Some installers (e.g. Miniconda/Mambaforge) are a shell header with a
    // binary payload appended (makeself-style). Lossy-decode rather than
    // fail, since the shell portion at the top is what we analyze anyway.
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}
