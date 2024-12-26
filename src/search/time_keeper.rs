use std::time;

pub struct TimeKeeper {
    start: time::Instant,
    timeout: time::Duration,
}

impl TimeKeeper {
    pub fn new(timeout: time::Duration) -> Self {
        TimeKeeper {
            start: time::Instant::now(),
            timeout,
        }
    }

    pub fn is_timeout(&self) -> bool {
        self.start.elapsed() >= self.timeout
    }
}
