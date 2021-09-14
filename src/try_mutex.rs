use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicBool, Ordering};

struct TryMutex<T> {
    locked: AtomicBool,
    value: UnsafeCell<T>,
}

pub struct TryMutexGuard<'a, T: 'a> {
    lock: &'a TryMutex<T>,
}

impl<'a, T> Deref for TryMutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.lock.value.get() }
    }
}

impl<'a, T> DerefMut for TryMutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.lock.value.get() }
    }
}

impl<'a, T> Drop for TryMutexGuard<'a, T> {
    fn drop(&mut self) {
        self.lock.locked.fetch_and(false, Ordering::Release);
    }
}

impl<T> TryMutex<T> {
    pub fn new(value: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            value: UnsafeCell::new(value),
        }
    }

    pub fn try_lock(&self) -> Option<TryMutexGuard<T>> {
        if self.locked.fetch_or(true, Ordering::AcqRel) == false {
            Some(TryMutexGuard { lock: self })
        } else {
            None
        }
    }
}

unsafe impl<T: Send> Sync for TryMutex<T> {}
unsafe impl<T: Send> Send for TryMutex<T> {}
unsafe impl<'a, T: Sync> Sync for TryMutexGuard<'a, T> {}
unsafe impl<'a, T: Send> Send for TryMutexGuard<'a, T> {}
