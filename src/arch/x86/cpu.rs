use core::prelude::*;

use io::{self, Reader, Writer};

use arch::idt::IDT;
use arch::gdt::GDT;

use arch::keyboard::Keyboard;

static DEFAULT_KEYBOARD: Keyboard = Keyboard {
  callback:     ::put_char,
  control_port: Port(0x64),
  data_port:    Port(0x60),
};

#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
pub enum IRQ { // after remap
  Timer        = 0x20,
  PS2Keyboard  = 0x21,
  Cascade      = 0x22,
  COM2         = 0x23,
  COM1         = 0x24,
  LPT2         = 0x25,
  Floppy       = 0x26,
  LPT1         = 0x27,
  CmosClock    = 0x28,
  FreeOne      = 0x29,
  FreeTwo      = 0x2a,
  FreeThree    = 0x2b,
  PsMouse      = 0x2c,
  FPU          = 0x2d,
  PrimaryAta   = 0x2e,
  SecondaryAta = 0x2f
}

pub unsafe fn init() {
  let mut gdt = GDT::new();

  gdt.identity_map();
  gdt.enable();

  PIC::master().remap_to(0x20);
  PIC::slave().remap_to(0x28);

  let mut idt = IDT::new();

  idt.enable();
}

fn acknowledge_irq(_: u32) {
  PIC::master().control_port.out_b(0x20); //TODO(ryan) ugly and only for master PIC
}

pub unsafe fn enable_interrupts() {
  IDT::enable_interrupts();
}

pub fn disable_interrupts() {
  IDT::disable_interrupts();
}

pub unsafe fn test_interrupt() {
  asm!("int 0x15" :::: "volatile", "intel");
}

pub unsafe fn idle() {
  asm!("hlt" ::::);
}

#[no_mangle]
pub extern "C" fn unified_handler(interrupt_number: u32) {
  match interrupt_number {
    0x15 => debug!("Hi from test interrupt handler"),
    0x20 => (), // timer
    0x21 => DEFAULT_KEYBOARD.got_interrupted(),
    _    => panic!("interrupt with no handler: {}", interrupt_number)
  }
  acknowledge_irq(interrupt_number);
}

#[no_mangle]
pub extern "C" fn add_entry(idt: &mut IDT, index: u32, f: unsafe extern "C" fn() -> ()) {
  idt.add_entry(index, f);
}


struct PIC {
  control_port: Port,
  mask_port: Port,
  is_master: bool
}

impl PIC {

  fn master() -> PIC {
    PIC { control_port: Port::new(0x20), mask_port: Port::new(0x21), is_master: true}
  }

  fn slave() -> PIC {
    PIC { control_port: Port::new(0xA0), mask_port: Port::new(0xA1), is_master: false}
  }

  unsafe fn remap_to(&mut self, start: u8) {
    let icw1 = 0x11;
    let icw4 = 0x1;
    let enable_all = 0x00;
    let typ = if self.is_master { 0x2 } else { 0x4 };

    self.control_port.out_b(icw1);
    self.mask_port.write(&[start, typ, icw4, enable_all]).ok();
  }

}

#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
pub struct Port(u16);

impl Port {

  pub fn new(number: u16) -> Port {
    Port(number)
  }

  pub fn in_b(self) -> u8 {
    let mut ret: u8;
    unsafe {
      asm!("inb $1, $0" : "={al}"(ret) :"{dx}"(self.0) ::)
    }
    return ret;
  }

  pub fn out_b(self, byte: u8) {
    unsafe {
      asm!("outb $1, $0" :: "{dx}"(self.0), "{al}"(byte) ::)
    }
  }

  pub fn out_w(self, word: u16) {
    unsafe {
      asm!("outw $1, $0" ::"{dx}"(self.0), "{ax}"(word) ::)
    }
  }

  pub fn in_w(self) -> u16 {
    let mut ret: u16;
    unsafe {
      asm!("inw $1, $0" : "={ax}"(ret) :"{dx}"(self.0)::)
    }
    ret
  }

  pub fn out_l(self, long: u32) {
    unsafe {
      asm!("outl $1, $0" ::"{dx}"(self.0), "{eax}"(long) ::)
    }
  }

  pub fn in_l(self) -> u32 {
    let mut ret: u32;
    unsafe {
      asm!("inl $1, $0" : "={eax}"(ret) :"{dx}"(self.0)::)
    }
    ret
  }

  pub fn io_wait() {
    Port::new(0x80).out_b(0);
  }

}

impl io::Reader for Port
{
  type Err = (); // TODO use bottom type

  //fn read_u8(&mut self) -> Result<u8, ()> {
  //  Ok(self.in_b())
  //}

  fn read(&mut self, buf: &mut [u8]) -> Result<usize, ()> {
    for el in buf.iter_mut() {
      *el = self.in_b();
    }
    Ok(buf.len())
  }

}

impl io::Writer for Port
{
  type Err = (); // TODO use bottom type

  //fn write_u8(&mut self, byte: u8) -> Result<(), ()> {
  //  self.out_b(byte);
  //  Ok(())
  //}

  fn write(&mut self, buf: &[u8]) -> Result<usize, ()> {
    for &byte in buf.iter() {
      self.out_b(byte);
    }
    Ok(buf.len())
  }

  fn flush(&mut self) -> Result<(), ()> {
    Ok(())
  }
}
