use core::fmt::Arguments;

use log::*;

use io::{EndOfFile, Write};

#[no_mangle]
pub extern "C" fn global_log_enabled(_: &LogMetadata) -> bool {
  true
}

#[no_mangle]
pub extern "C" fn global_log_log(record: &LogRecord) {
  let _ = write(format_args!("{}:{}: {}\n",
                             record.level(),
                             record.location().module_path(),
                             record.args()));
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
