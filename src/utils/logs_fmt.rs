use once_cell::sync::Lazy;
use std::fmt;
use std::time::Instant;
use tracing_subscriber::fmt::time::FormatTime;

static START: Lazy<Instant> = Lazy::new(Instant::now);

pub struct UptimeSeconds;

impl FormatTime for UptimeSeconds {
    fn format_time(&self, w: &mut tracing_subscriber::fmt::format::Writer<'_>) -> fmt::Result {
        let elapsed = START.elapsed();
        write!(w, "{:.3}s", elapsed.as_secs_f64())
    }
}

pub fn abbrev(s: &str) -> String {
    if s.len() > 14 {
        format!("{}...{}", &s[..8], &s[s.len() - 4..])
    } else {
        s.to_string()
    }
}
