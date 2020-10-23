use std::sync::atomic::{AtomicU8, Ordering::Relaxed};
use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut};

pub struct SpinLock<T> {
  mark: AtomicU8,
  obj: UnsafeCell<T>,
}

pub struct SpinLockGuard<'a, T> {
  lock: &'a SpinLock<T>, 
}

impl<T> SpinLock<T> {
  pub fn new(obj: T) -> Self {
    Self {
      mark: AtomicU8::new(0),
      obj: UnsafeCell::new(obj),
    }
  }

  pub fn lock(&self) -> SpinLockGuard<T> {
    let backoff = crossbeam_utils::Backoff::new();
    while self.mark.compare_and_swap(0, 1, Relaxed) != 0 {
      backoff.spin();
    }
    SpinLockGuard {
      lock: self
    }
  }
}

impl <'a, T> Deref for SpinLockGuard<'a, T> {
  type Target = T;
  fn deref(&self) -> &'a Self::Target {
    unsafe {
      &*self.lock.obj.get()
    }
  }
}

impl <'a, T> DerefMut for SpinLockGuard<'a, T> {
  fn deref_mut(&mut self) -> &'a mut T {
    unsafe {
      &mut *self.lock.obj.get()
    }
  }
}

impl <'a, T> Drop for SpinLockGuard<'a, T> {
  fn drop(&mut self) {
    self.lock.mark.store(0, Relaxed);
  }
}