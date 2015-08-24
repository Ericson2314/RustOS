use core::prelude::*;

use alloc::boxed::Box;

use collections::Vec;

use io::*;

use void::*;

pub trait Driver {

  fn init(&mut self);

}

pub trait DriverManager {

  fn get_drivers(&mut self) -> Vec<Box<NetworkDriver + 'static>>;

}

pub trait NetworkDriver: Driver
{
  fn address(&mut self) -> [u8; 6];

  fn put_frame(&mut self, buf: &[u8]) -> Result<usize, Void>;
  // TODO(ryan): more
}

pub fn adap_ref<T: NetworkDriver + ?Sized>(t: &mut T) -> &mut Adaptor<T> {
  unsafe { ::std::mem::transmute(t) }
}

pub struct Adaptor<T: NetworkDriver + ?Sized>(T);

impl<T: NetworkDriver + ?Sized> Write for Adaptor<T>
{
  type Err = Void;

  fn write(&mut self, buf: &[u8]) -> Result<usize, Void> {
    self.0.put_frame(buf)
  }
}
