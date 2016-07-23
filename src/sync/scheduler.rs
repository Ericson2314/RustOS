use collections::LinkedList;

use fringe::session::ThreadLocals;
use fringe::session::cycle::{C1, Cycle};
use spin;
use void::Void;

use sync::stack::BoxStack;

/// Represents a thread that yeilded of its own accord, and does not expect anything
struct Yielded(C1<'static, BoxStack, spin::MutexGuard<'static, Scheduler>>);

/// Queue of threadfs waiting to be run. Current thread is NOT in queue.
pub struct Scheduler {
  run_queue: LinkedList<Yielded>
}

pub type SchedulerCapability<'a> = spin::MutexGuard<'a, Scheduler>;

lazy_static! {
  static ref SCHEDULER: spin::Mutex<Scheduler> = spin::Mutex::new(Scheduler::new());
}

pub fn lock_scheduler() -> SchedulerCapability<'static> {
  SCHEDULER.lock()
}

impl Scheduler {
  pub fn new() -> Scheduler {
    Scheduler { run_queue: LinkedList::new() }
  }
}

fn put_back(old: Option<Yielded>, mut guard: SchedulerCapability)
{
  let ctx = match old {
    None      => return,
    Some(ctx) => ctx,
  };
  guard.run_queue.push_back(ctx);
}

pub trait SchedulerCapabilityExt {
  #[inline]
  fn spawn<F>(self, stack: BoxStack, f: F)
    where F: FnOnce(&mut ThreadLocals<BoxStack>) -> Void + Send + 'static;

  #[inline]
  fn yield_cur(self, maybe_stack: Option<&mut ThreadLocals<BoxStack>>);

  #[inline]
  fn exit(self, maybe_stack: Option<&mut ThreadLocals<BoxStack>>) -> !;
}

impl SchedulerCapabilityExt for SchedulerCapability<'static> {
  #[inline]
  fn spawn<F>(mut self, stack: BoxStack, f: F)
    where F: FnOnce(&mut ThreadLocals<BoxStack>) -> Void + Send + 'static
  {
    let ctx = C1::new(stack, |tls, (old, guard)| {
      put_back(old.map(Yielded), guard);
      f(tls)
    });
    self.run_queue.push_back(Yielded(ctx));
  }

  #[inline]
  fn yield_cur(mut self, maybe_stack: Option<&mut ThreadLocals<BoxStack>>)
  {
    let next = match self.run_queue.pop_front() {
      Some(n) => n,
      None    => {
        info!("The run queue is empty, will not yield");
        return
      },
    };
    let (old, guard) = next.0.swap(maybe_stack, self);
    put_back(old.map(Yielded), guard);
  }

  #[inline]
  fn exit(mut self, maybe_stack: Option<&mut ThreadLocals<BoxStack>>) -> !
  {
    let next = match self.run_queue.pop_front() {
      Some(n) => n,
      None    => {
        info!("The run queue is empty, will now \"shut down\"");
        drop(self); // In case we want to allow resurrections...
        ::abort()
      },
    };
    next.0.kontinue(maybe_stack, self)
  }
}

fn inner_thread_test(arg: usize) {
  debug!("arg is {}", arg)
}

fn test_thread(tl: &mut ThreadLocals<BoxStack>) -> Void {
  debug!("in a test thread!");
  inner_thread_test(11);
  let s = lock_scheduler();
  debug!("leaving test thread!");
  s.exit(Some(tl))
}

pub fn thread_stuff(tl: &mut ThreadLocals<BoxStack>) {
  debug!("starting thread test");
  let s = lock_scheduler();

  debug!("orig sched {:p}", &s);
  s.spawn(BoxStack::new(512), test_thread);
  debug!("schedule okay");
  lock_scheduler().yield_cur(Some(tl));
  debug!("back");
}
