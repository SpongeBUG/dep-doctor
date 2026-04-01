# Community Problem Definitions

Anyone can contribute new problem definitions here in TOML format.
These are loaded automatically alongside the built-in definitions.

## File format

Create a `.toml` file named after the problem ID:

```toml
id = "npm-some-package-CVE-2024-XXXXX"
title = "Brief description of the problem"
severity = "high"          # critical | high | medium | low | info
ecosystem = "npm"          # npm | pip | go | cargo
package = "some-package"
affected_range = ">=1.0.0 <2.3.4"
fixed_in = "2.3.4"
references = [
  "https://nvd.nist.gov/vuln/detail/CVE-2024-XXXXX",
]

[[source_patterns]]
languages = ["js", "ts"]

[[source_patterns.patterns]]
description = "Unsafe usage of someMethod()"
regex = 'somePackage\.someMethod\s*\('
confidence = "likely"      # definite | likely | possible
remediation = "Upgrade to >=2.3.4 and replace someMethod() with safeMethod()."
```

## Severity guide

| Level    | When to use |
|----------|-------------|
| critical | RCE, auth bypass, data destruction |
| high     | SSRF, credential leak, privilege escalation |
| medium   | Info disclosure, CSRF, DoS |
| low      | Minor info leak, best-practice violation |
| info     | Deprecated API, breaking change (no security impact) |

## Submitting

1. Fork the repo
2. Add your `.toml` file to this directory
3. Add a matching test fixture under `tests/fixtures/`
4. Open a PR — CI will validate automatically
