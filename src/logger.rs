use std::sync::Arc;

use internal::{Global, Local};

pub struct Logger(Arc<Global>);

impl Logger {
    pub fn new() -> Self {
        Self { 0: Arc::new(Global::default()) }
    }

    #[inline]
    pub fn handle(&self) -> Handle {
        Handle::new(self.0.clone())
    }
}

/// A logger that buffers logged lines.
pub struct Handle {
    global: Arc<Global>,
    local: Local,
}

impl Clone for Handle {
    fn clone(&self) -> Self {
        Self::new(self.global.clone())
    }
}

impl Handle {
    /// Create a new logger.
    fn new(global: Arc<Global>) -> Self {
        let local = Local::new(&global);
        Self { global, local }
    }

    /// Log a line.
    ///
    /// The logged line may be buffered. To force buffered lines to be printed, use `flush`.
    #[inline]
    pub fn log(&mut self, line: String) {
        self.local.log(&self.global, line)
    }

    /// Flush the buffer.
    ///
    /// Flushes any buffered log lines, printing them to stdout. These lines may be interpositioned
    /// arbitrarily with lines printed from other handles.
    #[inline]
    pub fn flush(&mut self) {
        self.local.flush(&self.global)
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        unsafe { self.local.finalize(&self.global); }
    }
}
