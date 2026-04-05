//! DTO conversions from txguard types to shared frontend types.

use rustok_types::{AnalysisResponse, FindingDto};
use txguard::types::{Action, Finding, Severity, Verdict};

/// Convert a txguard `Verdict` into a frontend-safe `AnalysisResponse`.
#[must_use]
pub fn verdict_to_dto(v: Verdict) -> AnalysisResponse {
    AnalysisResponse {
        action: match v.action {
            Action::Allow => "allow",
            Action::Warn => "warn",
            Action::Block => "block",
        }
        .to_string(),
        risk_score: v.risk_score,
        description: v.description,
        findings: v.findings.into_iter().map(finding_to_dto).collect(),
    }
}

/// Convert a txguard `Finding` into a frontend-safe `FindingDto`.
#[must_use]
pub fn finding_to_dto(f: Finding) -> FindingDto {
    FindingDto {
        rule: f.rule.to_string(),
        severity: match f.severity {
            Severity::Info => "info",
            Severity::Warning => "warning",
            Severity::Danger => "danger",
            Severity::Forbidden => "forbidden",
        }
        .to_string(),
        description: f.description,
    }
}
