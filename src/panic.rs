use io::EndOfFile;

use terminal;

#[lang = "panic_fmt"] #[inline(never)] #[cold]
pub extern fn panic_impl(msg: ::core::fmt::Arguments,
                         file: &'static str,
                         line: usize) -> !
{
  use io::Write;
  let _: Result<_, EndOfFile> = write!(terminal::GLOBAL.lock(), "{}:{} {}", file, line, msg);
  ::abort()
}

pub unsafe fn init() {
  terminal::GLOBAL.lock().clear_screen()
}
