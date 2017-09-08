// TODO: Add parametric allocators

#![feature(thread_local_state)]

#[macro_use]
extern crate lazy_static;
use std::sync::{Arc, Mutex};
use std::cell::UnsafeCell;

const MAX_LOCAL_LINES: usize = 8;
const MAX_GLOBAL_LINES: usize = 32;

struct Global {
    lines: Vec<String>,
}

impl Default for Global {
    fn default() -> Global {
        Global{
            lines: Vec::with_capacity(MAX_GLOBAL_LINES),
        }
    }
}

impl Global {
    fn extend(&mut self, lines: &mut Vec<String>) {
        while let Some(line) = lines.pop() {
            self.push(line);
        }
    }

    fn push(&mut self, line: String) {
        if self.lines.len() == MAX_GLOBAL_LINES {
            self.flush();
        }
        self.lines.push(line);
    }

    fn flush(&mut self) {
        while let Some(line) = self.lines.pop() {
            println!("{}", line);
        }
    }
}

/// A logger that buffers logged lines.
pub struct Logger {
    lines: Vec<String>,
    global: Arc<Mutex<Global>>,
}

impl Default for Logger {
    fn default() -> Logger {
        Logger {
            lines: Vec::with_capacity(MAX_LOCAL_LINES),
            global: Arc::new(Mutex::new(Global::default())),
        }
    }
}

impl Logger {
    /// Log a line.
    ///
    /// The logged line may be buffered. To force buffered lines to be printed, use `flush`.
    pub fn log(&mut self, line: String) {
        if self.lines.len() == MAX_LOCAL_LINES {
            self.flush();
        }
        self.lines.push(line);
    }

    pub fn flush(&mut self) {
        let mut guard = self.global.lock().unwrap();
        guard.extend(&mut self.lines);
        guard.flush();
    }

    fn log_global(&self, line: String) {
        self.global.lock().unwrap().push(line);
    }

    pub fn flush_global(&self) {
        self.global.lock().unwrap().flush();
    }
}

impl Clone for Logger {
    fn clone(&self) -> Logger {
        Logger{
            lines: Vec::with_capacity(MAX_LOCAL_LINES),
            global: self.global.clone(),
        }
    }
}

impl Drop for Logger {
    fn drop(&mut self) {
        self.global.lock().unwrap().extend(&mut self.lines);
    }
}

lazy_static!{ static ref GLOBAL_HANDLE: Logger = Logger::default(); }
thread_local!{ static TLS_HANDLE: UnsafeCell<Logger> = UnsafeCell::new(GLOBAL_HANDLE.clone()); }

/// Log a line.
///
/// The logged line may be buffered. To force buffered lines to be printed, use `flush`.
pub fn log(line: String) {
    let l = line.clone();
    if TLS_HANDLE.try_with(|handle| unsafe { (&mut *handle.get()).log(l) }).is_err() {
        GLOBAL_HANDLE.log_global(line);
    }
}

pub fn flush() {
    if TLS_HANDLE.try_with(|handle| unsafe { (&mut *handle.get()).flush() }).is_err() {
        GLOBAL_HANDLE.flush_global();
    }
}
