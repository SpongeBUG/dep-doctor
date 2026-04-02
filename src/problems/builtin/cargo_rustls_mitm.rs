use crate::problems::schema::{Confidence, Problem, ProblemKind, SourcePattern, SourcePatternSet};

pub fn problem() -> Problem {
    Problem {
        id: "cargo-rustls-MitM-RUSTSEC-2024-0336".into(),
        title: "rustls accepts certificate with invalid revocation status — MitM possible".into(),
        severity: "high".into(),
        ecosystem: "cargo".into(),
        package: "rustls".into(),
        affected_range: ">=0.21.0 <0.21.11,>=0.22.0 <0.22.4,>=0.23.0 <0.23.5".into(),
        fixed_in: Some("0.23.5".into()),
        references: vec!["https://rustsec.org/advisories/RUSTSEC-2024-0336.html".into()],
        kind: ProblemKind::Cve,
        source_patterns: Some(SourcePatternSet {
            languages: vec!["rs".into()],
            patterns: vec![SourcePattern {
                description: "ClientConfig with CRL (revocation) configured".into(),
                regex: r"ClientConfig::builder|with_root_certificates|with_crl".into(),
                confidence: Confidence::Possible,
                remediation:
                    "Upgrade rustls to >=0.23.5. If using CRL-based revocation, this is a definite risk."
                        .into(),
            }],
        }),
    }
}
