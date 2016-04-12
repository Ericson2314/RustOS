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
  fn top(&mut self) -> *mut u8 {
    let l = self.0.len() as isize;
    unsafe { (&mut*self.0).as_mut_ptr().offset(l) }
  }

  fn limit(&self) -> *const u8 {
    (&*self.0).as_ptr()
  }

}
