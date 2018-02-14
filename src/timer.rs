use std::thread::sleep;
use std::time::{Duration, Instant};

pub struct Timer {
    interval: Duration,
    last: Option<Instant>
}

impl Timer {
    pub fn new(interval: Duration) -> Timer {
        Timer {
            interval: interval,
            last: None
        }
    }

    pub fn wait(&mut self) {
        let now = Instant::now();
        self.last = Some(match self.last {
            Some(l) => {
                let elapsed = now.duration_since(l);
                if elapsed < self.interval {
                    sleep(self.interval - elapsed);
                    Instant::now()
                } else {
                    now
                }
            },
            None => now
        });
    }
}
