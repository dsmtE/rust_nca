use log::{Level, LevelFilter, Metadata, Record, SetLoggerError};

use env_logger::{fmt::Color, Builder, Logger, Target};
use std::io::Write;

use chrono::Local;

// use std::collections::VecDeque;

pub struct ConsoleLogger {
    // pub history: VecDeque<Record>
    inner: Logger,
    // level: bool,
}

impl ConsoleLogger {
    fn new() -> ConsoleLogger {
        let mut builder = Builder::from_default_env();
        builder
        .filter(None, LevelFilter::Trace)
        // .filter_module("wgpu", LevelFilter::Error)
            .target(Target::Stderr);

        // custom formating
        builder.format(|buf, record| {
            let mut level_style = buf.style();
            level_style
                .set_color(match record.level() {
                    Level::Error => Color::Red,
                    Level::Warn => Color::Yellow,
                    Level::Info => Color::White,
                    Level::Debug => Color::Rgb(200, 200, 200),
                    Level::Trace => Color::White,
                })
                .set_bold(true);

            writeln!(
                buf,
                "[{:>5}] {} - {}(l.{}) - {}",
                level_style.value(record.level()),
                Local::now().format("%Y_%m_%d - %H:%M:%S"),
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                record.args()
            )
        });

        ConsoleLogger {
            inner: builder.build(),
            // level: true,
            // history: VecDeque::new()
        }
    }

    pub fn init() -> Result<(), SetLoggerError> {
        let logger: ConsoleLogger = Self::new();

        log::set_max_level(logger.inner.filter());
        log::set_boxed_logger(Box::new(logger))
    }
}

impl log::Log for ConsoleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        // enable only for my module target
        metadata.target().starts_with("rust_nca") && metadata.level() <= Level::Trace
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            // if self.inner.filter.matches(record) {
            // do other things ? change target of inner logger ?
            self.inner.log(record)
            // }
        }
    }

    fn flush(&self) {
        // self.history.clear();
    }
}
