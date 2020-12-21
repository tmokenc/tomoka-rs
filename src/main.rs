use colorful::Colorful;
use dotenv::dotenv;
use std::env;
use tokio::runtime::Runtime;
use tokio::signal::{self, unix};
use tomoka_rs::Result;

fn main() -> Result<()> {
    dotenv().ok();
    let token = env::var("DISCORD_TOKEN")?;
    init_logger();
    Runtime::new()?.block_on(async move {
        let mut tomo = tomoka_rs::Instance::start(&token).await?;

        if let Some(shard) = tomo.shard() {
            tokio::spawn(ctrl_c_handle(shard));
        }

        tomo.wait().await
    })?;

    println!("Bye! for real");
    Ok(())
}

// To handle SIGINT and SIGTERM from the cargo watch
async fn ctrl_c_handle(shard_manager: tomoka_rs::Shard) {
    let mut term_sig = unix::signal(unix::SignalKind::terminate()).unwrap();
    let mut sig = Box::pin(term_sig.recv());
    let ctrl_c = Box::pin(signal::ctrl_c());
    futures::future::select(sig.as_mut(), ctrl_c).await;
    log::info!("{}", "RECEIVED THE EXIT SIGNAL".red().bold().underlined());
    shard_manager.lock().await.shutdown_all().await;
}

use colorful::core::color_string::CString;
use core::fmt::Arguments;
use lazy_static::lazy_static;
use log::{Level, LevelFilter, Record};
use std::io;
use core::time::Duration;

fn get_time_and_update(name: &str) -> Duration {
    use std::time::Instant;
    use dashmap::DashMap;
    
    lazy_static! {
        static ref TRACKING: DashMap<String, Instant> = DashMap::new();
    }

    let now = Instant::now();
    let duration = match TRACKING.get(name) {
        Some(time) => now.duration_since(*time),
        None => Duration::from_millis(0),
    };

    TRACKING.insert(name.to_owned(), now);

    duration
}

pub fn init_logger() {
    let console = fern::Dispatch::new()
        .format(console_format)
        .level(LevelFilter::Info)
        .level_for("tomoka_rs", LevelFilter::Trace)
        .level_for("tracing", LevelFilter::Error)
        .level_for("serenity", LevelFilter::Error)
        .filter(|meta| meta.level() > LevelFilter::Warn)
        .chain(io::stdout());

    let err_console = fern::Dispatch::new()
        .format(console_format)
        .level(LevelFilter::Warn)
        .chain(io::stderr());

    let file = fern::Dispatch::new()
        .format(file_format)
        .level(LevelFilter::Warn)
        //.level_for("serenity", LevelFilter::Trace)
        .level_for("tomoka_rs", LevelFilter::Debug)
        .chain(fern::DateBased::new("logs/", "tomo-%F.log"));

    fern::Dispatch::new()
        .chain(file)
        .chain(console)
        .chain(err_console)
        .apply()
        .unwrap();
}

fn console_format(cb: fern::FormatCallback, message: &Arguments, record: &Record) {
    let mut name = record.target().to_owned();

    if let Some(line) = record.line() {
        name.push_str(&format!(":{}", line));
    }

    let duration = get_time_and_update(&name);

    cb.finish(format_args!(
        "{}{} {}{} {} ({}ms)",
        "[".dark_gray(),
        level_style(record.level()),
        name,
        "]".dark_gray(),
        message,
        duration.as_millis(),
    ))
}

fn file_format(cb: fern::FormatCallback, message: &Arguments, record: &Record) {
    let line = record.line().map(|v| format!(":{}", v));

    cb.finish(format_args!(
        "{} {:<5} {}{} {}",
        chrono::Local::now().format("%T%.3f"),
        record.level(),
        record.target(),
        line.unwrap_or_default(),
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