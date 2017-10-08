use std::cell::UnsafeCell;
use epoch::unprotected;

use internal::{Global, Local};

lazy_static!{ static ref GLOBAL: Global = Global::default(); }
thread_local!{ static HANDLE: UnsafeCell<Handle> = { UnsafeCell::new(Handle::new()) }; }

struct Handle(Local);

impl Handle {
    pub fn new() -> Self {
        Self { 0: Local::new(&GLOBAL) }
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        unsafe { self.0.unregister(&GLOBAL) }
    }
}

/// Log a line.
///
/// The logged line may be buffered. To force buffered lines to be printed, use `flush`.
pub fn log(line: String) {
    let l = line.clone();
    if HANDLE.try_with(|handle| unsafe { (&mut *(handle.get())).0.log(&GLOBAL, l) }).is_err() {
        unsafe { unprotected(|scope| GLOBAL.push(line, scope)) }
    }
}

/// Flush the buffer.
///
/// Flushes any buffered log lines, printing them to stdout. These lines may be interpositioned
/// arbitrarily with lines printed from other threads.
pub fn flush() {
    if HANDLE.try_with(|handle| unsafe { (&mut *handle.get()).0.flush(&GLOBAL) }).is_err() {
        unsafe { unprotected(|scope| GLOBAL.flush(scope)) }
    }
}
