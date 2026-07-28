use anyhow::{Context, Result};
use bashlens_report::{BehaviorClass, ClassRisk, Finding, Report, RiskSummary, Severity};
use bashlens_rules::RuleSet;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;

/// Classes that contribute to the risk score. Verification is deliberately
/// excluded here - its presence reduces risk rather than adding to it, and
/// it's reported to the user as a separate present/absent signal, not a bar.
const RISK_CLASSES: [BehaviorClass; 4] = [
    BehaviorClass::Network,
    BehaviorClass::Privilege,
    BehaviorClass::Persistence,
    BehaviorClass::Obfuscation,
];

/// Corpus scan results used to rank a single report against real-world installers.
pub struct CorpusBaseline {
    size: usize,
    overall: Vec<f64>,
    per_class: std::collections::HashMap<BehaviorClass, Vec<f64>>,
}

/// On-disk form of a `CorpusBaseline`. A flat struct with one field per risk
/// class rather than a generic map, since `BehaviorClass` isn't a valid JSON
/// map key (serde_json requires string keys) and there are only ever the 4
/// fixed `RISK_CLASSES` to store.
#[derive(Serialize, Deserialize)]
struct BaselineFile {
    size: usize,
    overall: Vec<f64>,
    network: Vec<f64>,
    privilege: Vec<f64>,
    persistence: Vec<f64>,
    obfuscation: Vec<f64>,
}

impl CorpusBaseline {
    pub fn len(&self) -> usize {
        self.size
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    /// Loads a previously cached baseline (see `save`). Rebuilding it from
    /// scratch means re-parsing every corpus script with tree-sitter on
    /// every single invocation, which is the difference between a ~10ms and
    /// a ~3s CLI run at 174 scripts - this cache is what keeps `bashlens`
    /// fast day-to-day. Regenerate it (via `--update-baseline`) after any
    /// change to `corpus/scripts/` or `rules/`.
    pub fn load<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        let data = std::fs::read_to_string(path.as_ref())
            .with_context(|| format!("failed to read baseline cache {:?}", path.as_ref()))?;
        Self::from_json(&data)
    }

    /// The baseline snapshot embedded into the binary at compile time
    /// (`corpus/stats/baseline.json` as of the last build) - what a
    /// distributed binary falls back to when no local cache or corpus
    /// checkout is present. Necessarily a point-in-time snapshot: it only
    /// updates when the binary itself is rebuilt and redistributed.
    pub fn embedded() -> Result<Self> {
        const EMBEDDED_BASELINE: &str = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../corpus/stats/baseline.json"
        ));
        Self::from_json(EMBEDDED_BASELINE)
    }

    fn from_json(data: &str) -> Result<Self> {
        let file: BaselineFile =
            serde_json::from_str(data).context("failed to parse baseline cache")?;
        let mut per_class = std::collections::HashMap::new();
        per_class.insert(BehaviorClass::Network, file.network);
        per_class.insert(BehaviorClass::Privilege, file.privilege);
        per_class.insert(BehaviorClass::Persistence, file.persistence);
        per_class.insert(BehaviorClass::Obfuscation, file.obfuscation);
        Ok(Self {
            size: file.size,
            overall: file.overall,
            per_class,
        })
    }

    pub fn save<P: AsRef<std::path::Path>>(&self, path: P) -> Result<()> {
        let empty = Vec::new();
        let file = BaselineFile {
            size: self.size,
            overall: self.overall.clone(),
            network: self
                .per_class
                .get(&BehaviorClass::Network)
                .unwrap_or(&empty)
                .clone(),
            privilege: self
                .per_class
                .get(&BehaviorClass::Privilege)
                .unwrap_or(&empty)
                .clone(),
            persistence: self
                .per_class
                .get(&BehaviorClass::Persistence)
                .unwrap_or(&empty)
                .clone(),
            obfuscation: self
                .per_class
                .get(&BehaviorClass::Obfuscation)
                .unwrap_or(&empty)
                .clone(),
        };
        let data = serde_json::to_string_pretty(&file)?;
        std::fs::write(path.as_ref(), data)
            .with_context(|| format!("failed to write baseline cache {:?}", path.as_ref()))
    }
}

pub struct Analyzer {
    rules: RuleSet,
}

impl Analyzer {
    pub fn from_rules_dir<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        let p = path.as_ref();
        Ok(Self {
            rules: RuleSet::load_dir(p)
                .with_context(|| format!("failed to load rules from {:?}", p))?,
        })
    }

    /// Rules baked into the binary at compile time - what a distributed
    /// binary should use, since it can't assume a `rules/` checkout exists
    /// next to it.
    pub fn embedded() -> Result<Self> {
        Ok(Self {
            rules: RuleSet::embedded().context("failed to load embedded rules")?,
        })
    }

    pub fn analyze(&self, name: &str, script: &str) -> Result<Report> {
        let tree = bashlens_parser::parse(script).context("failed to parse script")?;
        let mut findings = Vec::new();

        for rule in self.rules.iter() {
            let mut evidence = rule.evaluate(&tree, script);
            if !evidence.is_empty() {
                // Cap for readability - a rule that fires dozens of times
                // (e.g. many URLs in a long installer) shouldn't drown the
                // report; the count is still implied by the finding existing.
                evidence.truncate(6);
                findings.push(Finding {
                    class: class_from_str(&rule.class),
                    severity: severity_from_str(&rule.severity),
                    description: rule.description.clone(),
                    evidence,
                });
            }
        }

        let classes: Vec<BehaviorClass> = findings
            .iter()
            .map(|f| f.class.clone())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        // Mirrored from findings rather than tracked separately, so
        // "unresolvable" constructs (dynamic URLs/targets tree-sitter can
        // structurally locate but not resolve to a literal) are reported
        // explicitly instead of silently dropped - see rules/dynamic-target.
        let unresolvable: Vec<String> = findings
            .iter()
            .filter(|f| f.class == BehaviorClass::Unresolvable)
            .flat_map(|f| f.evidence.clone())
            .collect();

        let digest = hex::encode(Sha256::digest(script.as_bytes()));

        Ok(Report {
            name: name.to_string(),
            sha256: digest,
            classes,
            findings,
            unresolvable,
            risk: None,
        })
    }

    /// Re-analyzes every script in `corpus_dir` to build the score distribution
    /// a single report is later ranked against. Crude by design (matches the
    /// rest of v0.1): re-scans the corpus on every invocation rather than
    /// shipping a precomputed cache, which is fine at a few hundred small files.
    pub fn build_corpus_baseline<P: AsRef<std::path::Path>>(
        &self,
        corpus_dir: P,
    ) -> Result<CorpusBaseline> {
        let dir = corpus_dir.as_ref();
        let mut overall = Vec::new();
        let mut per_class: std::collections::HashMap<BehaviorClass, Vec<f64>> =
            std::collections::HashMap::new();

        for entry in std::fs::read_dir(dir)
            .with_context(|| format!("corpus directory {:?} not readable", dir))?
        {
            let entry = entry?;
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            if path
                .file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.starts_with('.'))
                .unwrap_or(true)
            {
                continue;
            }
            let bytes = std::fs::read(&path)?;
            let text = String::from_utf8_lossy(&bytes);
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            let report = self.analyze(name, &text)?;

            let mut total = 0.0;
            for class in RISK_CLASSES {
                let score = class_score(&report.findings, &class);
                per_class.entry(class).or_default().push(score);
                total += score;
            }
            if has_verification(&report.findings) {
                total -= VERIFIED_BONUS;
            }
            overall.push(total);
        }

        Ok(CorpusBaseline {
            size: overall.len(),
            overall,
            per_class,
        })
    }

    pub fn risk_summary(&self, report: &Report, baseline: &CorpusBaseline) -> RiskSummary {
        let mut per_class = Vec::new();
        let mut overall_score = 0.0;

        for class in RISK_CLASSES {
            let score = class_score(&report.findings, &class);
            overall_score += score;
            let empty = Vec::new();
            let dist = baseline.per_class.get(&class).unwrap_or(&empty);
            per_class.push(ClassRisk {
                class: class.clone(),
                score,
                percentile: percentile_rank(score, dist),
            });
        }

        let verified = has_verification(&report.findings);
        if verified {
            overall_score -= VERIFIED_BONUS;
        }

        RiskSummary {
            corpus_size: baseline.size,
            overall_score,
            overall_percentile: percentile_rank(overall_score, &baseline.overall),
            per_class,
            verified,
        }
    }
}

const VERIFIED_BONUS: f64 = 5.0;

fn severity_weight(s: &Severity) -> f64 {
    match s {
        Severity::Low => 1.0,
        Severity::Medium => 2.0,
        Severity::High => 3.0,
        Severity::Critical => 4.0,
    }
}

fn class_score(findings: &[Finding], class: &BehaviorClass) -> f64 {
    let score: f64 = findings
        .iter()
        .filter(|f| &f.class == class)
        .map(|f| severity_weight(&f.severity) * (f.evidence.len().max(1) as f64))
        .sum();
    // `Sum for f64` folds from -0.0, so an empty (no findings in this class)
    // sum comes out as negative zero. All real terms here are non-negative,
    // so `.abs()` only ever normalizes that sign artifact, never a real value.
    score.abs()
}

fn has_verification(findings: &[Finding]) -> bool {
    findings
        .iter()
        .any(|f| f.class == BehaviorClass::Verification)
}

/// Mid-rank percentile: ties split the difference rather than all clustering
/// at the top of the tied group, which matters here since scores come from a
/// handful of discrete evidence-count buckets and ties are common.
fn percentile_rank(score: f64, distribution: &[f64]) -> u8 {
    if distribution.is_empty() {
        return 0;
    }
    let n = distribution.len() as f64;
    let below = distribution.iter().filter(|&&s| s < score).count() as f64;
    let equal = distribution.iter().filter(|&&s| s == score).count() as f64;
    let pr = 100.0 * (below + 0.5 * equal) / n;
    (pr.round() as i64).clamp(0, 100) as u8
}

fn class_from_str(s: &str) -> BehaviorClass {
    match s.to_ascii_lowercase().as_str() {
        "privilege" => BehaviorClass::Privilege,
        "persistence" => BehaviorClass::Persistence,
        "network" => BehaviorClass::Network,
        "verification" => BehaviorClass::Verification,
        "obfuscation" => BehaviorClass::Obfuscation,
        "unresolvable" => BehaviorClass::Unresolvable,
        _ => BehaviorClass::Unresolvable,
    }
}

fn severity_from_str(s: &str) -> Severity {
    match s.to_ascii_lowercase().as_str() {
        "low" => Severity::Low,
        "medium" => Severity::Medium,
        "high" => Severity::High,
        "critical" => Severity::Critical,
        _ => Severity::Medium,
    }
}
