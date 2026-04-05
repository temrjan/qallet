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

#[cfg(test)]
mod tests {
    use super::*;
    use txguard::types::RuleCategory;

    #[test]
    fn verdict_allow_converts() {
        let verdict = Verdict {
            action: Action::Allow,
            risk_score: 0,
            findings: vec![],
            description: "native ETH transfer".into(),
            simulation: None,
        };
        let dto = verdict_to_dto(verdict);
        assert_eq!(dto.action, "allow");
        assert_eq!(dto.risk_score, 0);
        assert_eq!(dto.description, "native ETH transfer");
        assert!(dto.findings.is_empty());
    }

    #[test]
    fn verdict_warn_with_findings() {
        let verdict = Verdict {
            action: Action::Warn,
            risk_score: 27,
            findings: vec![Finding {
                rule: "unlimited_approval",
                severity: Severity::Warning,
                category: RuleCategory::Approval,
                description: "unlimited token approval".into(),
            }],
            description: "ERC-20 approve".into(),
            simulation: None,
        };
        let dto = verdict_to_dto(verdict);
        assert_eq!(dto.action, "warn");
        assert_eq!(dto.risk_score, 27);
        assert_eq!(dto.findings.len(), 1);
        assert_eq!(dto.findings[0].rule, "unlimited_approval");
        assert_eq!(dto.findings[0].severity, "warning");
    }

    #[test]
    fn verdict_block_converts() {
        let verdict = Verdict {
            action: Action::Block,
            risk_score: 92,
            findings: vec![Finding {
                rule: "known_scam",
                severity: Severity::Forbidden,
                category: RuleCategory::Address,
                description: "known scam address".into(),
            }],
            description: "transfer to scam".into(),
            simulation: None,
        };
        let dto = verdict_to_dto(verdict);
        assert_eq!(dto.action, "block");
        assert_eq!(dto.findings[0].severity, "forbidden");
    }

    #[test]
    fn unified_balance_converts() {
        use alloy_primitives::U256;

        let core_balance = crate::provider::UnifiedBalance {
            total: U256::from(1_000_000_000_000_000_000u128),
            approximate_total_formatted: "~1.0 ETH".into(),
            chains: vec![crate::provider::ChainBalance {
                chain_id: 1,
                chain_name: "Ethereum".into(),
                balance: U256::from(1_000_000_000_000_000_000u128),
                formatted: "1.0".into(),
            }],
            errors: vec![],
        };

        let dto: rustok_types::UnifiedBalance = core_balance.into();
        assert_eq!(dto.approximate_total_formatted, "~1.0 ETH");
        assert_eq!(dto.chains.len(), 1);
        assert_eq!(dto.chains[0].chain_id, 1);
        assert_eq!(dto.chains[0].formatted, "1.0");
    }
}
