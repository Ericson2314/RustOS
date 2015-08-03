pub use self::imp::{
  vga,
  context,
  cpu,
  keyboard,
};

#[cfg(target_arch = "x86")]
#[path="x86"]
mod imp {
  pub mod vga;
  pub mod context;
  pub mod cpu;
  pub mod keyboard;
}
