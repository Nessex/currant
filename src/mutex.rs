use std::sync::atomic::{AtomicBool, Ordering};
use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut};
use std::time::Duration;

pub struct Mutex<T> {
    locked: AtomicBool,
    value: UnsafeCell<T>,
}

pub struct MutexGuard<'a, T: 'a> {
    lock: &'a Mutex<T>,
}

impl<'a, T> Deref for MutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.lock.value.get() }
    }
}

impl<'a, T> DerefMut for MutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.lock.value.get() }
    }
}

impl<'a, T> Drop for MutexGuard<'a, T> {
    fn drop(&mut self) {
        self.lock.locked.fetch_and(false, Ordering::Release);
    }
}

impl<T> Mutex<T> {
    pub fn new(value: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            value: UnsafeCell::new(value),
        }
    }

    pub fn spin_lock(&self) -> MutexGuard<T> {
        loop {
            if self.locked.fetch_or(true, Ordering::Acquire) == false {
                return MutexGuard { lock: self };
            }
        }
    }

    pub fn yield_lock(&self) -> MutexGuard<T> {
        loop {
            if self.locked.fetch_or(true, Ordering::Acquire) == false {
                return MutexGuard { lock: self };
            }

            std::thread::yield_now();
        }
    }

    pub fn exp_backoff_lock(&self) -> MutexGuard<T> {
        let mut backoff = Duration::from_millis(1);
        loop {
            if self.locked.fetch_or(true, Ordering::Acquire) == false {
                return MutexGuard { lock: self };
            }

            std::thread::sleep(backoff);
            backoff *= 2;
        }
    }

    pub fn try_lock(&self) -> Option<MutexGuard<T>> {
        if self.locked.fetch_or(true, Ordering::Acquire) == false {
            Some(MutexGuard { lock: self })
        } else {
            None
        }
    }
}

unsafe impl<T: Send> Sync for Mutex<T> {}
unsafe impl<T: Send> Send for Mutex<T> {}
unsafe impl<'a, T: Sync> Sync for MutexGuard<'a, T> {}
unsafe impl<'a, T: Send> Send for MutexGuard<'a, T> {}

#[cfg(test)]
mod tests {
    use crate::Mutex;
    use std::sync::Arc;
    use std::time::Duration;

    #[test]
    fn try_lock() {
        let mtx = Arc::new(Mutex::new(0usize));
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

    #[test]
    fn spin_lock() {
        let mtx = Arc::new(Mutex::new(0usize));
        let mtx_2 = mtx.clone();

        let h1 = std::thread::spawn(move || {
            let g = mtx.spin_lock();

            assert_eq!(*g, 0);
            std::thread::sleep(Duration::from_millis(500));
        });

        std::thread::sleep(Duration::from_millis(50));

        let h2 = std::thread::spawn(move || {
            let g = mtx_2.spin_lock();

            assert_eq!(*g, 0);
        });

        h1.join().unwrap();
        h2.join().unwrap();
    }

    #[test]
    fn yield_lock() {
        let mtx = Arc::new(Mutex::new(0usize));
        let mtx_2 = mtx.clone();

        let h1 = std::thread::spawn(move || {
            let g = mtx.yield_lock();

            assert_eq!(*g, 0);
            std::thread::sleep(Duration::from_millis(500));
        });

        std::thread::sleep(Duration::from_millis(50));

        let h2 = std::thread::spawn(move || {
            let g = mtx_2.yield_lock();

            assert_eq!(*g, 0);
        });

        h1.join().unwrap();
        h2.join().unwrap();
    }

    #[test]
    fn exp_backoff_lock() {
        let mtx = Arc::new(Mutex::new(0usize));
        let mtx_2 = mtx.clone();

        let h1 = std::thread::spawn(move || {
            let g = mtx.exp_backoff_lock();

            assert_eq!(*g, 0);
            std::thread::sleep(Duration::from_millis(500));
        });

        std::thread::sleep(Duration::from_millis(50));

        let h2 = std::thread::spawn(move || {
            let g = mtx_2.exp_backoff_lock();

            assert_eq!(*g, 0);
        });

        h1.join().unwrap();
        h2.join().unwrap();
    }
}
