#![no_std]

#![allow(unstable)]
#![allow(improper_ctypes)]

#![feature(asm)]
#![feature(lang_items)]
#![feature(box_syntax)]

// not directly used, but needed to link to llvm emitted calls
extern crate rlibc;

#[macro_use]
//extern crate std; // for useful macros and IO interfaces
extern crate core;
extern crate alloc;
extern crate collections;


#[macro_use] #[no_link]
extern crate bitflags;
extern crate "external" as bump_ptr;
#[macro_use]
extern crate lazy_static_spin;
extern crate spinlock;

use core::prelude::*;

use collections::Vec;

use multiboot::multiboot_info;
use arch::cpu;
use pci::Pci;
use driver::DriverManager;
//use thread::scheduler;

#[macro_use]
mod log;
pub mod arch;
mod terminal;
mod panic;
mod multiboot;
//mod thread;
mod pci;
mod rtl8139;
mod driver;
mod net;

mod io;


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

lazy_static_spin! {
  static ref TEST: Vec<&'static str> = {
    let mut v = Vec::new();
    v.push("hi from lazy static");
    v
  };
}

#[no_mangle]
pub extern "C" fn main(magic: u32, info: *mut multiboot_info) {
  unsafe {
    terminal::init_global();

    bump_ptr::set_allocator((15us * 1024 * 1024) as *mut u8, (20us * 1024 * 1024) as *mut u8);
    panic::init();
    test_allocator();

    if magic != multiboot::MULTIBOOT_BOOTLOADER_MAGIC {
      panic!("Multiboot magic is invalid");
    } else {
      debug!("Multiboot magic is valid. Info at 0x{:x}", info as u32);
      (*info).multiboot_stuff();
    }

    debug!("{}", (**TEST)[0]);

    cpu::CURRENT_CPU.lock().make_keyboard(put_char);

    cpu::CURRENT_CPU.lock().enable_interrupts();
    debug!("Going to interrupt: ");
    cpu::CURRENT_CPU.lock().test_interrupt();
    debug!("    back from interrupt!");

    debug!("start scheduling...");

    //scheduler::thread_stuff();

    pci_stuff();

    info!("Kernel is done!");
    loop {
      cpu::CURRENT_CPU.lock().idle()
    }
  }
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
pub extern "C" fn debug(s: &'static str, u: u32) {
  debug!("{} 0x{:x}", s, u)
}

#[no_mangle]
pub extern "C" fn __morestack() {
  loop { } //TODO(ryan) should I do anything here?
}

#[no_mangle]
pub extern "C" fn abort() -> ! {
    unsafe { core::intrinsics::abort(); }
}

#[no_mangle]
pub extern "C" fn callback() {
  debug!("    in an interrupt!");
}

// TODO(ryan): figure out what to do with these:

#[lang = "stack_exhausted"]
extern fn stack_exhausted() {}

#[lang = "eh_personality"]
extern fn eh_personality() {}

// for deriving
#[doc(hidden)]
mod std {
  pub use core::clone;
  pub use core::cmp;
  pub use core::marker;
  pub use core::option;
  pub use core::fmt;
  pub use core::hash;
}
