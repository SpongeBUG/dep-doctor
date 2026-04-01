use super::builtin::{cargo_rustls_mitm, npm_axios_csrf, npm_lodash_prototype, pip_requests_ssrf};
use crate::problems::schema::Problem;

/// Returns every built-in problem definition.
pub fn all_problems() -> Vec<Problem> {
    vec![
        npm_axios_csrf::problem(),
        npm_lodash_prototype::problem(),
        pip_requests_ssrf::problem(),
        cargo_rustls_mitm::problem(),
    ]
}
