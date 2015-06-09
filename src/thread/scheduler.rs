// TODO(ryan): it really looks like bulk of libgreen could be used here where pthread <-> core

use core::prelude::*;
use core::cell::UnsafeCell;
use core::mem::{transmute, transmute_copy};
use core::ptr;

use alloc::boxed::Box;

use collections::LinkedList;

use spin;

use thread::context::Context;
use thread::stack::Stack;

use arch::cpu;

// thread control block
struct Tcb {
  context: Context,
}

// invariant: current thread is at front of queue
pub struct Scheduler {
  queue: LinkedList<Tcb>
}

lazy_static_spin! {
  static SCHEDULER: spin::Mutex<Scheduler> = spin::Mutex::new(Scheduler::new());
}

pub fn get_scheduler<'a>() -> &'a spin::Mutex<Scheduler> {
  SCHEDULER.get_or_init()
}

extern "C" fn run_thunk(thunk: &Fn() -> ()) {
  debug!("in run_thunk");
  thunk();
  unreachable!("didn't unschedule finished thread");
}

impl Scheduler {

  pub fn new() -> Scheduler {
    let idle_task = || {
        loop {
            trace!("in idle task 1");
            trace!("wait done");
            get_scheduler().lock().switch();
            trace!("switch done");
            loop {}
        }
    };

    let mut s = Scheduler { queue: LinkedList::new() };
    let tcb = s.new_tcb(box idle_task);
    s.queue.push_front(tcb);
    s
  }

  pub fn bootstrap_start(&mut self) -> ! {
    // scheduler now takes control of the CPU
    // current context is discarded and front of queue is started
    let mut dont_care = Context::empty();
    Context::swap(&mut dont_care, &self.queue.front_mut().unwrap().context);
    unreachable!();
  }

  fn new_tcb(&self, func: Box<Fn() -> ()>) -> Tcb {
    const STACK_SIZE: usize = 1024 * 1024;
    let stack = Stack::new(STACK_SIZE);

    let p = move || {
      unsafe { cpu::enable_interrupts() };
      func();
      get_scheduler().lock().unschedule_current();
    };

    let c = Context::new(run_thunk, box p as Box<Fn() -> ()>, stack);
    Tcb { context: c }
  }

  pub fn schedule(&mut self, func: Box<Fn() -> ()>) {
    unsafe { cpu::disable_interrupts() };

    let new_tcb = self.new_tcb(func);
    self.queue.push_back(new_tcb);

    unsafe { cpu::enable_interrupts(); }
  }

  fn unschedule_current(&mut self) -> ! {
    debug!("unscheduling");

    unsafe { cpu::disable_interrupts() };

    self.queue.pop_front(); // get rid of current
    let next = self.queue.pop_back().unwrap();
    self.queue.push_front(next);

    let mut dont_care = Context::empty();
    Context::swap(&mut dont_care, &mut self.queue.front_mut().unwrap().context);
    unreachable!();
  }

  pub fn switch(&mut self) {
    debug_assert!(self.queue.len() >= 1);
    unsafe { cpu::disable_interrupts() };
    if self.queue.len() < 2 {
      return;
    }
    let old = self.queue.pop_front().unwrap();
    let next = self.queue.pop_back().unwrap();
    self.queue.push_front(next);
    self.queue.push_back(old);

    let (back, front) = unsafe { ends_mut(&mut self.queue) }.unwrap();
    Context::swap(&mut back.context, &front.context);

    unsafe { cpu::enable_interrupts(); } // TODO(ryan): make a mutex as enabling/disabling interrupts
  }

}

unsafe fn ends_mut<T>(ll: &mut LinkedList<T>) -> Option<(&mut T, &mut T)> {
  let b: *mut T = match ll.back_mut() {
    Some(x) => x as *mut T,
    None => return None,
  };
  let f = match ll.front_mut() {
    Some(x) => x,
    None => ::core::intrinsics::unreachable(),
  };
  Some((transmute(b), f))
}

fn inner_thread_test(arg: usize) {
  debug!("arg is {}", arg)
}

extern "C" fn test_thread() {
  debug!("in a test thread!");
  inner_thread_test(11);
  unsafe {
    let s = get_scheduler();
    debug!("leaving test thread!");
    s.lock().unschedule_current();
  }
}

pub fn thread_stuff() {
  debug!("starting thread test");
  unsafe {
    let s = get_scheduler();

    debug!("orig sched 0x{:x}", transmute_copy::<_, u32>(&s));
    //loop {};
    let t = || { test_thread() };
    s.lock().schedule(box t);
    debug!("schedule okay");
    s.lock().switch();
    debug!("back");
  }
}
