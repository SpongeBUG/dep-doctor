use crate::problems::schema::Problem;
use super::builtin::{npm_axios_csrf, npm_lodash_prototype, pip_requests_ssrf, cargo_rustls_mitm};

/// Returns every built-in problem definition.
/// Community definitions (from problems.d/) are loaded separately.
pub fn all_problems() -> Vec<Problem> {
    vec![
        npm_axios_csrf::problem(),
        npm_lodash_prototype::problem(),
        pip_requests_ssrf::problem(),
        cargo_rustls_mitm::problem(),
    ]
}
