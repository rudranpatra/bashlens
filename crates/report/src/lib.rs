use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt::Write;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BehaviorClass {
    Privilege,
    Persistence,
    Network,
    Verification,
    Obfuscation,
    Unresolvable,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Finding {
    pub class: BehaviorClass,
    pub severity: Severity,
    pub description: String,
    pub evidence: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClassRisk {
    pub class: BehaviorClass,
    pub score: f64,
    pub percentile: u8,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RiskSummary {
    pub corpus_size: usize,
    pub overall_score: f64,
    pub overall_percentile: u8,
    pub per_class: Vec<ClassRisk>,
    pub verified: bool,
}

impl RiskSummary {
    fn bar(percentile: u8) -> String {
        let filled = ((percentile as f64 / 100.0) * 10.0).round() as usize;
        let filled = filled.min(10);
        format!(
            "{}{}",
            "\u{2588}".repeat(filled),
            "\u{2591}".repeat(10 - filled)
        )
    }

    fn label(percentile: u8) -> &'static str {
        if percentile < 34 {
            "low"
        } else if percentile < 67 {
            "medium"
        } else {
            "high"
        }
    }

    pub fn render(&self) -> String {
        let mut s = String::new();
        for cr in &self.per_class {
            let name = format!("{:?}", cr.class);
            writeln!(
                s,
                "  {:<13} {}  {}",
                name,
                Self::bar(cr.percentile),
                Self::label(cr.percentile)
            )
            .ok();
        }
        writeln!(
            s,
            "  {:<13} {}",
            "Verification",
            if self.verified {
                "\u{2705} present"
            } else {
                "\u{274c} none"
            }
        )
        .ok();
        writeln!(s).ok();
        writeln!(s, "  Risk percentile: {}th", self.overall_percentile).ok();
        writeln!(
            s,
            "  Compared against {} installers in the corpus",
            self.corpus_size
        )
        .ok();
        s
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Report {
    pub name: String,
    pub sha256: String,
    pub classes: Vec<BehaviorClass>,
    pub findings: Vec<Finding>,
    pub unresolvable: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub risk: Option<RiskSummary>,
}

impl Report {
    pub fn text(&self) -> String {
        let mut s = format!("{}\nsha256: {}\n\n", self.name, self.sha256);
        if let Some(risk) = &self.risk {
            s.push_str(&risk.render());
            s.push('\n');
        }
        let mut grouped: BTreeMap<String, Vec<&Finding>> = BTreeMap::new();
        for f in &self.findings {
            grouped.entry(format!("{:?}", f.class)).or_default().push(f);
        }
        for (class, items) in grouped {
            writeln!(s, "[{}]", class).ok();
            for it in items {
                writeln!(
                    s,
                    "  [{:?}] {}  evidence: {:?}",
                    it.severity, it.description, it.evidence
                )
                .ok();
            }
        }
        s
    }

    pub fn markdown(&self) -> String {
        let mut s = format!("# {}\n\n- sha256: `{}`\n\n", self.name, self.sha256);
        if let Some(risk) = &self.risk {
            writeln!(s, "```\n{}```\n", risk.render()).ok();
        }
        for f in &self.findings {
            writeln!(
                s,
                "- **{:?}** (`{:?}`): {}",
                f.class, f.severity, f.description
            )
            .ok();
            if !f.evidence.is_empty() {
                for e in &f.evidence {
                    writeln!(s, "  - `{}`", e).ok();
                }
            }
        }
        s
    }
}
