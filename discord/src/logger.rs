use dashmap::DashMap;
use env_logger::fmt::{Color, Formatter};
use lazy_static::lazy_static;
use log::Record;
use std::io::{Result as IoResult, Write};
use std::time::{Duration, Instant};

lazy_static! {
    static ref TRACKING: DashMap<String, Instant> = DashMap::new();
}

/// Just base on env_logger for simplicity
pub fn init() {
    env_logger::builder().format(logging).init();
}

fn logging(buf: &mut Formatter, record: &Record) -> IoResult<()> {
    let level = buf.default_styled_level(record.level());

    let mut bracket_style = buf.style();
    bracket_style.set_color(Color::Black).set_intense(true);
    
    let mut name = record.target().to_owned();
    
    if let Some(line) = record.line() {
        name.push_str(&format!(":{}", line));
    }

    let duration = get_time_and_update(&name);

    writeln!(
        buf,
        "{}{} {}{} {:#?} ({}ms)",
        bracket_style.value("["),
        level,
        name,
        bracket_style.value("]"),
        record.args(),
        duration.as_millis(),
    )
}

fn get_time_and_update(name: &str) -> Duration {
    let now = Instant::now();
    let duration = match TRACKING.get(name) {
        Some(time) => now.duration_since(*time),
        None => Duration::from_millis(0),
    };
 TRACKING.insert(name.to_owned(), now);

    duration
}
