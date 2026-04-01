/// Simple logger macros. Uses stderr so stdout stays clean for reporters.
/// Replace with `tracing` crate if you need structured logging later.

#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        if std::env::var("DEP_DOCTOR_DEBUG").is_ok() {
            eprintln!("[debug] {}", format!($($arg)*));
        }
    };
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        eprintln!("[warn]  {}", format!($($arg)*));
    };
}
