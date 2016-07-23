use alloc::boxed::Box;
use collections::Vec;

use fringe::Stack;

pub struct BoxStack(Box<[u8]>);

impl BoxStack {
  pub fn new(size: usize) -> BoxStack {
    let mut v = Vec::with_capacity(size);
    unsafe { v.set_len(size) };
    BoxStack(v.into_boxed_slice())
  }
}

impl Stack for BoxStack {
  fn top(&self) -> *mut u8 {
    let p: &[u8] = &*self.0;
    let raw_top = unsafe { p.as_ptr().offset(p.len() as isize) };
    (raw_top as usize & !0xF) as _
  }

  fn limit(&self) -> *mut u8 {
    (&*self.0).as_ptr() as _
  }

}
