use core::prelude::*;

use alloc::boxed::Box;

use collections::Vec;

use io::*;

pub trait Driver {

  fn init(&mut self);

}

pub trait DriverManager {

  fn get_drivers(&mut self) -> Vec<Box<NetworkDriver + 'static>>;

}

pub trait NetworkDriver: Driver {

  fn address(&mut self) -> [u8; 6];

  fn put_frame(&mut self, buf: &[u8]) -> Result<usize, ()>;
  // TODO(ryan): more
}

impl<T> Writer for T where T: NetworkDriver
{
  type Err = ();

  fn write(&mut self, buf: &[u8]) -> Result<usize, ()> {
    match self.put_frame(buf) {
      Ok(_)  => Ok(buf.len()),
      Err(_) => Err(())
    }
  }
}

impl<'a> Writer for NetworkDriver + 'a
{
  type Err = ();

  fn write(&mut self, buf: &[u8]) -> Result<usize, ()> {
    match self.put_frame(buf) {
      Ok(_)  => Ok(buf.len()),
      Err(_) => Err(())
    }
  }
}
