use core::fmt::Arguments;

use log::*;

use io::{EndOfFile, Write};


struct TerminalLogger;

impl Log for TerminalLogger {
  fn enabled(&self, _: &LogMetadata) -> bool { true }
  fn log(&self, record: &LogRecord) {
    let _ = write(format_args!("{}:{}: {}\n",
                               record.level(),
                               record.location().module_path(),
                               record.args()));
  }
}

pub fn init() -> Result<(), SetLoggerError> {
  unsafe {
    set_logger_raw(|max_log_level| {
      max_log_level.set(LogLevelFilter::max());
      &TerminalLogger
    })
  }
}

pub fn write(args: Arguments) {
  // Arguments are already evaluated, so dead-lock is avoided
  let r: Result<(), EndOfFile> = ::terminal::GLOBAL.lock().write_fmt(args);
  drop(r)
}

//#[macro_export]
macro_rules! __print {
  ($($arg:tt)*) => {
    ::log_impl::write(format_args!($($arg)*))
  };
}
