use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicBool, Ordering};

pub struct TryMutex<T> {
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

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::time::Duration;
    use crate::TryMutex;

    #[test]
    fn try_lock() {
        let mtx = Arc::new(TryMutex::new(0usize));
        let mtx_2 = mtx.clone();

        let h1 = std::thread::spawn(move || {
            let g = match mtx.try_lock() {
                None => panic!(),
                Some(g) => g,
            };

            assert_eq!(*g, 0);
            std::thread::sleep(Duration::from_millis(500));
        });

        std::thread::sleep(Duration::from_millis(50));

        let h2 = std::thread::spawn(move || {
            match mtx_2.try_lock() {
                None => assert!(true),
                Some(_) => panic!(),
            }
        });

        h1.join().unwrap();
        h2.join().unwrap();
    }
}