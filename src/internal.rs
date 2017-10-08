use std::sync::atomic::{AtomicUsize, Ordering};
use epoch::{self as epoch, Scope};
use queue::Queue;

const MAX_GLOBAL_LINES: usize = 32;
const MAX_LOCAL_LINES: usize = 8;

pub struct Global {
    epoch: epoch::Global,
    lines: Queue<String>,
    counter: AtomicUsize,
}

/// Local data for a logger.
pub struct Local {
    epoch: epoch::Local,
    lines: Vec<String>,
}

impl Default for Global {
    fn default() -> Global {
        Global {
            epoch: epoch::Global::new(),
            lines: Queue::new(),
            counter: AtomicUsize::default(),
        }
    }
}

impl Global {
    pub fn extend<'s>(&'s self, lines: &'s mut Vec<String>, scope: &'s Scope) {
        while let Some(line) = lines.pop() {
            self.push(line, scope);
        }
    }

    pub fn push<'s>(&'s self, line: String, scope: &'s Scope) {
        if self.counter.fetch_add(1, Ordering::Relaxed) % MAX_GLOBAL_LINES == 0 {
            self.flush(scope);
        }
        self.lines.push(line, scope);
    }

    pub fn flush<'s>(&'s self, scope: &'s Scope) {
        while let Some(line) = self.lines.try_pop(scope) {
            println!("{}", line);
        }
    }
}

impl Local {
    pub fn new(global: &Global) -> Self {
        Self {
            epoch: epoch::Local::new(&global.epoch),
            lines: Vec::with_capacity(MAX_LOCAL_LINES),
        }
    }

    /// Log a line.
    ///
    /// The logged line may be buffered. To force buffered lines to be printed, use `flush`.
    pub unsafe fn log(&mut self, global: &Global, line: String) {
        if self.lines.len() == MAX_LOCAL_LINES {
            self.flush(&global);
        }
        self.lines.push(line);
    }

    /// Flush the buffer.
    ///
    /// Flushes any buffered log lines, printing them to stdout. These lines may be interpositioned
    /// arbitrarily with lines printed from other handles.
    pub unsafe fn flush(&mut self, global: &Global) {
        let mut lines = &mut self.lines;
        self.epoch.pin(&global.epoch, |scope| {
            global.extend(&mut lines, scope);
            global.flush(scope);
        });
    }

    pub unsafe fn unregister<'s>(&'s mut self, global: &'s Global) {
        let mut lines = &mut self.lines;
        self.epoch.pin(&global.epoch, |scope| {
            global.extend(&mut lines, scope);
        });

        self.epoch.unregister(&global.epoch);
    }
}
