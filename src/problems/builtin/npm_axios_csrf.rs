use crate::problems::schema::{Confidence, Problem, SourcePattern, SourcePatternSet};

pub fn problem() -> Problem {
    Problem {
        id: "npm-axios-csrf-ssrf-CVE-2023-45857".into(),
        title: "Axios CSRF token leak via cross-origin redirect (SSRF-adjacent)".into(),
        severity: "high".into(),
        ecosystem: "npm".into(),
        package: "axios".into(),
        affected_range: ">=0.8.1 <1.6.0".into(),
        fixed_in: Some("1.6.0".into()),
        references: vec![
            "https://github.com/advisories/GHSA-wf5p-g6vw-rhxx".into(),
            "https://nvd.nist.gov/vuln/detail/CVE-2023-45857".into(),
        ],
        source_patterns: Some(SourcePatternSet {
            languages: vec!["js".into(), "ts".into()],
            patterns: vec![
                SourcePattern {
                    description: "axios request with user-controlled URL variable".into(),
                    regex: r"axios\.(get|post|put|delete|patch|request)\s*\(\s*(req\.|res\.|params\.|query\.|body\.|user)".into(),
                    confidence: Confidence::Likely,
                    remediation: "Upgrade axios to >=1.6.0. Validate and allowlist URLs before passing to axios.".into(),
                },
                SourcePattern {
                    description: "axios.create with withCredentials: true".into(),
                    regex: r"axios\.create\s*\([^)]*withCredentials\s*:\s*true".into(),
                    confidence: Confidence::Definite,
                    remediation: "Upgrade axios to >=1.6.0. Avoid withCredentials:true for requests that follow redirects to external origins.".into(),
                },
                SourcePattern {
                    description: "axios instance created (any — review manually)".into(),
                    regex: r"axios\.create\s*\(".into(),
                    confidence: Confidence::Possible,
                    remediation: "Upgrade axios to >=1.6.0 and audit axios.create() instances for withCredentials usage.".into(),
                },
            ],
        }),
    }
}
