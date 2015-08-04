use core::fmt::Arguments;
use io::Writer;

pub fn write(args: Arguments) {
  // Arguments are already evaluated, so dead-lock is avoided
  ::terminal::GLOBAL.lock().write_fmt(args).ok();
}

//#[macro_export]
macro_rules! __print(
  ($($arg:tt)*) => (::log::write(format_args!($($arg)*)))
);

#[macro_export]
macro_rules! log(
  ($lvl: expr, $($arg:tt)*) => (
    // Must be one print for atomicity
    __print!("[{}:{} {}]: {}\n", $lvl, file!(), line!(), format_args!($($arg)*))
  )
);

#[macro_export]
macro_rules! debug(
  ($($arg:tt)*) => (log!("DEBUG", $($arg)*))
);

#[macro_export]
macro_rules! warn(
  ($($arg:tt)*) => (log!("WARN", $($arg)*))
);

#[macro_export]
macro_rules! info(
  ($($arg:tt)*) => (log!("INFO", $($arg)*))
);

#[macro_export]
macro_rules! trace(
  ($($arg:tt)*) => (log!("TRACE", $($arg)*))
);
