use core::prelude::*;

use io::{self, Reader, Writer};

use arch::idt::IDT;
pub use cpu::*;

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
  SecondaryAta = 0x2f,
}



pub unsafe fn init() {
  set_gdt(GDT.get_or_init());

  // Reload segment registers after lgdt
  set_cs(SegmentSelector::new(1, PrivilegeLevel::Ring0));

  let ds = SegmentSelector::new(2, PrivilegeLevel::Ring0);
  set_ds(ds);
  set_es(ds);
  set_fs(ds);
  set_gs(ds);
  set_ss(ds);

  PIC::master().remap_to(0x20);
  PIC::slave().remap_to(0x28);

  let mut idt = IDT::new();

  idt.enable();
}

fn acknowledge_irq(_: u32) {
  PIC::master().control_port.out8(0x20); //TODO(ryan) ugly and only for master PIC
}

pub unsafe fn test_interrupt() {
  asm!("int 0x15" :::: "volatile", "intel");
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

// TODO should be real statics
lazy_static_spin! {

  static GDT: [GdtEntry; 3] = {[
    GdtEntry::NULL,
    GdtEntry::new(0 as *const (),
                  0xFFFFFFFF,
                  GdtAccess::Executable | GdtAccess::NotTss,
                  PrivilegeLevel::Ring0),
    GdtEntry::new(0 as *const (),
                  0xFFFFFFFF,
                  GdtAccess::Writable | GdtAccess::NotTss,
                  PrivilegeLevel::Ring0),
    //gdt.add_entry( = {.base=&myTss, .limit=sizeof(myTss), .type=0x89}; // You can use LTR(0x18)
  ]};

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

    self.control_port.out8(icw1);
    self.mask_port.write(&[start, typ, icw4, enable_all]).ok();
  }

}



#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Debug)]
pub struct Port(u16);

impl Port {

  pub const fn new(number: u16) -> Port {
    Port(number)
  }

  pub fn in8(self) -> u8 {
    unsafe { ::cpu::in8(self.0) }
  }

  pub fn out8(self, num: u8) {
    unsafe { ::cpu::out8(self.0, num) }
  }

  pub fn in16(self) -> u16 {
    unsafe { ::cpu::in16(self.0) }
  }

  pub fn out16(self, num: u16) {
    unsafe { ::cpu::out16(self.0, num) }
  }

  pub fn in32(self) -> u32 {
    unsafe { ::cpu::in32(self.0) }
  }

  pub fn out32(self, num: u32) {
    unsafe { ::cpu::out32(self.0, num) }
  }

  pub fn io_wait() {
    Port::new(0x80).out8(0);
  }

}

impl io::Reader for Port
{
  type Err = (); // TODO use bottom type

  //fn read_u8(&mut self) -> Result<u8, ()> {
  //  Ok(self.in8())
  //}

  fn read(&mut self, buf: &mut [u8]) -> Result<usize, ()> {
    for el in buf.iter_mut() {
      *el = self.in8();
    }
    Ok(buf.len())
  }

}

impl io::Writer for Port
{
  type Err = (); // TODO use bottom type

  //fn write_u8(&mut self, byte: u8) -> Result<(), ()> {
  //  self.out8(byte);
  //  Ok(())
  //}

  fn write(&mut self, buf: &[u8]) -> Result<usize, ()> {
    for &byte in buf.iter() {
      self.out8(byte);
    }
    Ok(buf.len())
  }

  fn flush(&mut self) -> Result<(), ()> {
    Ok(())
  }
}
