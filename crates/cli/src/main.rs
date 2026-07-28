use anyhow::Result;
use bashlens_analyzer::Analyzer;
use clap::Parser;

#[derive(Parser)]
#[command(
    name = "bashlens",
    about = "Inspect shell install scripts before you run them"
)]
struct Args {
    /// URL or local path to the install script
    target: String,

    /// Output format
    #[arg(long, value_enum, default_value = "text")]
    format: Format,
}

#[derive(clap::ValueEnum, Clone)]
enum Format {
    Text,
    Json,
    Markdown,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let args = Args::parse();

    let source = fetch(&args.target)?;
    let analyzer = Analyzer::from_rules_dir(std::path::Path::new("rules"))?;
    let mut report = analyzer.analyze(&args.target, &source)?;

    let corpus_dir = std::path::Path::new("corpus/scripts");
    if corpus_dir.is_dir() {
        let baseline = analyzer.build_corpus_baseline(corpus_dir)?;
        if !baseline.is_empty() {
            report.risk = Some(analyzer.risk_summary(&report, &baseline));
        }
    }

    match args.format {
        Format::Json => println!("{}", serde_json::to_string_pretty(&report)?),
        Format::Markdown => println!("{}", report.markdown()),
        Format::Text => println!("{}", report.text()),
    }

    Ok(())
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
