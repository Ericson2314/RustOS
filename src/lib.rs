#![no_std]

#![allow(improper_ctypes)]

#![feature(asm)]
#![feature(lang_items)]
#![feature(box_syntax)]
#![feature(box_patterns)]
#![feature(associated_consts)]
#![feature(slice_patterns)]
#![feature(const_fn)]
#![feature(core_intrinsics)]
#![feature(naked_functions)]

#![feature(alloc, collections)]
extern crate alloc;
extern crate collections;

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate log;
// not directly used, but needed to link to llvm emitted calls
extern crate rlibc;

extern crate coreio as io;
extern crate cpu;
extern crate fringe;
#[macro_use]
extern crate lazy_static;
extern crate spin;
extern crate void;

extern crate bump_pointer;

#[macro_use]
mod log_impl;
pub mod arch;
mod terminal;
mod panic;
mod multiboot;
mod pci;
mod rtl8139;
mod driver;
mod net;
mod sync;

use collections::Vec;

use multiboot::multiboot_info;
use pci::Pci;
use driver::DriverManager;
use sync::scheduler::{self, SchedulerCapabilityExt};
use sync::stack::BoxStack;


fn test_allocator() {
  let mut v = Vec::new();

  debug!("Testing allocator with a vector push");
  v.push("   hello from a vector!");
  debug!("   push didn't crash");
  match v.pop() {
    Some(string) => debug!("{}", string),
    None => debug!("    push was weird...")
  }

}

fn put_char(c: u8) {
  __print!("{}", c as char);
}

lazy_static! {
  static ref TEST: Vec<&'static str> = {
    let mut v = Vec::new();
    v.push("hi from lazy static");
    v
  };
}

#[no_mangle]
pub extern "C" fn main(magic: u32, info: *mut multiboot_info) -> ! {
  log_impl::init().unwrap();

  // some preliminaries
  terminal::init_global();
  bump_pointer::set_allocator((15usize * 1024 * 1024) as *mut u8,
                              (20usize * 1024 * 1024) as *mut u8);

  if magic != multiboot::MULTIBOOT_BOOTLOADER_MAGIC {
    panic!("Multiboot magic is invalid");
  } else {
    debug!("Multiboot magic is valid. Info at 0x{:x}", info as u32);
    unsafe { (*info).multiboot_stuff() };
  }

  debug!("kernel start!");
  panic::init();
  debug!("Going to set up CPU:");
  unsafe { arch::cpu::init() };

  debug!("And enable Interrupts");
  unsafe { cpu::enable_interrupts() };

  // We're going to now enter the scheduler to do the rest
  let mut s = scheduler::lock_scheduler();
  s.spawn(sync::BoxStack::new(512),
                                    bootstrapped_main);
  debug!("start scheduling...");

  // Okay, scheduler, take it away!
  s.exit(fringe::session::native_thread_locals())
}

fn bootstrapped_main(tl: &mut fringe::session::ThreadLocals<BoxStack>)
                     -> void::Void
{
  debug!("kernel main thread start!");

  debug!("Testing allocator");
  test_allocator();

  debug!("Going to test lazy_static:");
  debug!("{}", (*TEST)[0]);

  debug!("Going to interrupt: ");
  unsafe { arch::cpu::test_interrupt() };
  debug!("Back from interrupt!");

  pci_stuff();

  debug!("Testing scheduler");
  scheduler::thread_stuff(tl);

  info!("Kernel main thread is done!");
  scheduler::lock_scheduler().exit(Some(tl))
}

fn pci_stuff() {
  let mut pci = Pci::new();
  pci.init();
  let mut drivers = pci.get_drivers();
  debug!("Found drivers for {} pci devices", drivers.len());
  match drivers.pop() {
    Some(mut driver) => {
      driver.init();
      net::NetworkStack::new(driver).test().ok();
    }
    None => ()
  }

}

#[no_mangle]
pub extern "C" fn abort() -> ! {
  unsafe {
    cpu::disable_interrupts();
    cpu::halt();
    core::intrinsics::unreachable();
  }
}

// for deriving
#[doc(hidden)]
mod std {
  pub use core::*;
}
