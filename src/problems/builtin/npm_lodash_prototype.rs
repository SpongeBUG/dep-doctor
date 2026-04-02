use crate::problems::schema::{Confidence, Problem, ProblemKind, SourcePattern, SourcePatternSet};

pub fn problem() -> Problem {
    Problem {
        id: "npm-lodash-prototype-pollution-CVE-2019-10744".into(),
        title: "Lodash prototype pollution via defaultsDeep / merge / set".into(),
        severity: "critical".into(),
        ecosystem: "npm".into(),
        package: "lodash".into(),
        affected_range: "<4.17.21".into(),
        fixed_in: Some("4.17.21".into()),
        references: vec![
            "https://nvd.nist.gov/vuln/detail/CVE-2019-10744".into(),
            "https://github.com/advisories/GHSA-jf85-cpcp-j695".into(),
        ],
        kind: ProblemKind::Cve,
        source_patterns: Some(SourcePatternSet {
            languages: vec!["js".into(), "ts".into()],
            patterns: vec![
                SourcePattern {
                    description: "_.defaultsDeep with external input".into(),
                    regex: r"_\.defaultsDeep\s*\(".into(),
                    confidence: Confidence::Likely,
                    remediation: "Upgrade lodash to >=4.17.21.".into(),
                },
                SourcePattern {
                    description: "_.merge with external input".into(),
                    regex: r"_\.merge\s*\(".into(),
                    confidence: Confidence::Possible,
                    remediation: "Upgrade lodash to >=4.17.21. Sanitize input objects before merging.".into(),
                },
                SourcePattern {
                    description: "_.set with user-controlled path".into(),
                    regex: r"_\.set\s*\([^,]+,\s*(req\.|params\.|query\.|body\.)".into(),
                    confidence: Confidence::Definite,
                    remediation: "Upgrade lodash to >=4.17.21. Never use user input as the path argument to _.set.".into(),
                },
            ],
        }),
    }
}
