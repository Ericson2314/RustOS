use core::marker::Unsize;

use collections::LinkedList;

use fringe::{Context, ThreadLocals, Stack};
use spin;
use void::{self, Void};

/// Represents a thread that yeilded of its own accord, and does not expect anything
struct Yielded(pub Context<'static, (Option<Yielded>, spin::MutexGuard<'static, Scheduler>)>);

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
  fn spawn<S, F>(self, stack: S, f: F)
    where S: Stack + Send + 'static,
          F: FnOnce(&mut ThreadLocals<S>) -> Void + Send + 'static;

  #[inline]
  fn yield_cur<S>(self, maybe_stack: Option<&mut ThreadLocals<S>>)
    where S: Unsize<Stack> + Send;

  #[inline]
  fn exit<S>(self,
                 maybe_stack: Option<&mut ThreadLocals<S>>)
                 -> !
    where S: Unsize<Stack> + Send;
}

impl SchedulerCapabilityExt for SchedulerCapability<'static> {
  #[inline]
  fn spawn<S, F>(mut self, stack: S, f: F)
    where S: Stack + Send + 'static,
          F: FnOnce(&mut ThreadLocals<S>) -> Void + Send + 'static
  {
    let ctx = Context::new(stack, |tls, (old, guard)| {
      put_back(old, guard);
      f(tls)
    });
    self.run_queue.push_back(Yielded(ctx));
  }

  #[inline]
  fn yield_cur<S>(mut self, maybe_stack: Option<&mut ThreadLocals<S>>)
    where S: Unsize<Stack> + Send
  {
    let next = match self.run_queue.pop_front() {
      Some(n) => n,
      None    => {
        info!("The run queue is empty, will not yield");
        return
      },
    };
    let (old, guard) = next.0.switch(maybe_stack, |ctx| (Some(Yielded(ctx)), self));
    put_back(old, guard);
  }

  #[inline]
  fn exit<S>(mut self,
             maybe_stack: Option<&mut ThreadLocals<S>>)
             -> !
    where S: Unsize<Stack> + Send
  {
    let next = match self.run_queue.pop_front() {
      Some(n) => n,
      None    => {
        info!("The run queue is empty, will now \"shut down\"");
        drop(self); // In case we want to allow resurrections...
        ::abort()
      },
    };
    void::unreachable(next.0.switch(maybe_stack, |_| (None, self)))
  }
}

fn inner_thread_test(arg: usize) {
  debug!("arg is {}", arg)
}

fn test_thread<S>(tl: &mut ThreadLocals<S>) -> Void
  where S: Stack + Send + 'static
{
  debug!("in a test thread!");
  inner_thread_test(11);
  let s = lock_scheduler();
  debug!("leaving test thread!");
  s.exit(Some(tl))
}

pub fn thread_stuff<S>(tl: &mut ThreadLocals<S>)
  where S: Stack + Send + 'static
{
  use ::sync::BoxStack;

  debug!("starting thread test");
  let s = lock_scheduler();

  debug!("orig sched {:p}", &s);
  s.spawn(BoxStack::new(512), test_thread);
  debug!("schedule okay");
  lock_scheduler().yield_cur(Some(tl));
  debug!("back");
}
