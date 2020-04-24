use crate::Result;
use colorful::core::color_string::CString;
use colorful::Colorful;
use core::fmt::Arguments;
use dashmap::DashMap;
use lazy_static::lazy_static;
use log::{Level, LevelFilter, Record};
use std::io;
use std::time::{Duration, Instant};

lazy_static! {
    static ref TRACKING: DashMap<String, Instant> = DashMap::new();
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

pub fn init() -> Result<()> {
    let console = fern::Dispatch::new()
        .format(console_format)
        .level(LevelFilter::Info)
        .level_for("tomoka_rs", LevelFilter::Trace)
        .filter(|meta| meta.level() > LevelFilter::Warn)
        .chain(io::stdout());

    let err_console = fern::Dispatch::new()
        .format(console_format)
        .level(LevelFilter::Warn)
        .chain(io::stderr());

    let file = fern::Dispatch::new()
        .format(file_format)
        .level(LevelFilter::Warn)
        .level_for("serenity", LevelFilter::Debug)
        .level_for("tomoka_rs", LevelFilter::Debug)
        .chain(fern::DateBased::new("logs/", "%F.tomolog"));

    fern::Dispatch::new()
        .chain(file)
        .chain(console)
        .chain(err_console)
        .apply()?;

    Ok(())
}

fn console_format(callback: fern::FormatCallback, message: &Arguments, record: &Record) {
    let mut name = record.target().to_owned();

    if let Some(line) = record.line() {
        name.push_str(&format!(":{}", line));
    }

    let duration = get_time_and_update(&name);

    callback.finish(format_args!(
        "{}{} {}{} {} ({}ms)",
        "[".dark_gray(),
        level_style(record.level()),
        name,
        "]".dark_gray(),
        message,
        duration.as_millis(),
    ))
}

fn file_format(callback: fern::FormatCallback, message: &Arguments, record: &Record) {
    callback.finish(format_args!(
        "{} {} {} {}",
        chrono::Local::now().format("%F %T%.3f"),
        record.level(),
        record.target(),
        message,
    ))
}

fn level_style(level: Level) -> CString {
    match level {
        Level::Trace => "TRACE".dark_gray(),
        Level::Debug => "DEBUG".white(),
        Level::Info => "INFO".green(),
        Level::Warn => "WARN".yellow(),
        Level::Error => "ERROR".red().bold(),
    }
}
