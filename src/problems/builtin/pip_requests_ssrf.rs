use crate::problems::schema::{Confidence, Problem, ProblemKind, SourcePattern, SourcePatternSet};

pub fn problem() -> Problem {
    Problem {
        id: "pip-requests-redirect-credential-leak-CVE-2023-32681".into(),
        title: "Requests forwards Authorization header across redirects to different hosts".into(),
        severity: "medium".into(),
        ecosystem: "pip".into(),
        package: "requests".into(),
        affected_range: ">=2.1.0 <2.31.0".into(),
        fixed_in: Some("2.31.0".into()),
        references: vec![
            "https://nvd.nist.gov/vuln/detail/CVE-2023-32681".into(),
            "https://github.com/psf/requests/security/advisories/GHSA-j8r2-6x86-q33q".into(),
        ],
        kind: ProblemKind::Cve,
        source_patterns: Some(SourcePatternSet {
            languages: vec!["py".into()],
            patterns: vec![
                SourcePattern {
                    description: "requests call with Authorization header".into(),
                    regex: r#"requests\.(get|post|put|delete|patch|request|Session)\s*\([^)]*["\']Authorization["\']"#.into(),
                    confidence: Confidence::Likely,
                    remediation: "Upgrade requests to >=2.31.0.".into(),
                },
                SourcePattern {
                    description: "Session with auth set".into(),
                    regex: r"session\.auth\s*=".into(),
                    confidence: Confidence::Possible,
                    remediation: "Upgrade requests to >=2.31.0. Use allow_redirects=False for sensitive authenticated calls.".into(),
                },
            ],
        }),
    }
}
