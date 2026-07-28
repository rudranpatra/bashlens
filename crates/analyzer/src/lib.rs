use anyhow::{Context, Result};
use bashlens_report::{BehaviorClass, ClassRisk, Finding, Report, RiskSummary, Severity};
use bashlens_rules::RuleSet;
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

impl CorpusBaseline {
    pub fn len(&self) -> usize {
        self.size
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
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

    pub fn analyze(&self, name: &str, script: &str) -> Result<Report> {
        let mut findings = Vec::new();

        for rule in self.rules.iter() {
            if rule.applies(script) {
                findings.push(Finding {
                    class: class_from_str(&rule.class),
                    severity: severity_from_str(&rule.severity),
                    description: rule.description.clone(),
                    evidence: rule.evidence(script),
                });
            }
        }

        let classes: Vec<BehaviorClass> = findings
            .iter()
            .map(|f| f.class.clone())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        let digest = hex::encode(Sha256::digest(script.as_bytes()));

        Ok(Report {
            name: name.to_string(),
            sha256: digest,
            classes,
            findings,
            unresolvable: Vec::new(),
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
