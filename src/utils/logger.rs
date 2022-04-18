use log::{Record, Level, Metadata, SetLoggerError};
use std::sync::Mutex;

use chrono::Local;

use std::collections::VecDeque;

use bitmask_enum::bitmask;

#[bitmask(u8)]
pub enum FilteringMask {
    Error,
    Warn,
    Info,
    Trace,
    Debug, 
}

impl From<log::Level> for FilteringMask {
    fn from(level: log::Level) -> Self {
        match level {
            Level::Error => FilteringMask::Error,
            Level::Warn => FilteringMask::Warn,
            Level::Info => FilteringMask::Info,
            Level::Trace => FilteringMask::Trace,
            Level::Debug => FilteringMask::Debug,
        }
    }
}

pub struct MyRecord {
    level: log::Level,
    target: String,
    message: String,
    file: Option<String>,
    line: Option<u32>,
}

// impl<'a> From<log::Record<'a>> for MyRecord<'a> {
//     fn from(record: log::Record<'a>) -> Self {
//         MyRecord {
//             metadata: *record.metadata(),
//             message: format!("{}", record.args()),
//             file: record.file(),
//             line: record.line()
//         }
//     }
// }

impl From<&log::Record<'_>> for MyRecord {
    fn from(record: &log::Record<'_>) -> Self {
        MyRecord {
            level: record.level(),
            target: record.target().to_owned(),
            message: format!("{}", record.args()),
            file: record.file().map(ToOwned::to_owned),
            line: record.line()
        }
    }
}



// impl<'a> MyRecord<'a> {
//     /// The verbosity level of the message.
//     #[inline]
//     pub fn level(&self) -> Level {
//         self.metadata.level()
//     }

//     /// The name of the target of the directive.
//     #[inline]
//     pub fn target(&self) -> &'a str {
//         self.metadata.target()
//     }
// }


pub struct ConsoleLogger {
    // Use wrapper with only wanted information insteal of whole Record struct not threadSafe (impl<'a> log::Log : `core::fmt::Opaque` cannot be shared between threads safely)
    pub history: Mutex<VecDeque<MyRecord>>,
    pub filtering_mask: FilteringMask,
}

impl ConsoleLogger {
    fn new() -> ConsoleLogger {
        ConsoleLogger {
            history: Mutex::new(VecDeque::with_capacity(256)),
            filtering_mask: FilteringMask::all()
        }
    }

    pub fn init() -> Result<(), SetLoggerError> {
        let logger: ConsoleLogger = Self::new();
        
        log::set_boxed_logger(Box::new(logger))
    }

    // -> impl Iterator<Item = Record>
    // pub fn get_filtered_logs(&self) {
    //     self.history.iter().filter(|& record| self.filtering_mask.contains(record.level().into()));
    // }

    pub fn get_filtered_logs(&self) {
        let mut hist = self.history.lock().unwrap();
        hist.iter().filter(|& record| self.filtering_mask.contains(record.level.into())).collect::<Vec<&MyRecord>>();
    }

}

// pub fn mut_logger<FN>(funct: FN) where FN: FnOnce(&mut ConsoleLogger) {
//     let mut logger = LOGGER.lock().unwrap();
//     funct(&mut logger);
// }

// https://github.com/rust-lang/log/issues/51

impl log::Log for ConsoleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        // enable only for my module target
        metadata.target().starts_with("rust_nca") && metadata.level() <= Level::Trace
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            // TODO: how handle color in println!
            // let mut level_style = buf.style();
            // level_style.set_color(match record.level() {
            //     Level::Error=>Color::Red,
            //     Level::Warn=>Color::Yellow,
            //     Level::Info=>Color::White,
            //     Level::Debug=>Color::Rgb(200, 200, 200),
            //     Level::Trace=>Color::White
            // }).set_bold(true);
            {
                let mut mutable_history = self.history.lock().unwrap();
                mutable_history.push_back(MyRecord::from(record));
            }

            println!("[{:>5}] {} - {}(l.{}) - {}",
            // level_style.value(record.level()),
                record.level(),
                Local::now().format("%Y_%m_%d - %H:%M:%S"),
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                record.args()
            )
        }
    }

    fn flush(&self) {
        self.history.lock().unwrap().clear();
    }
}
