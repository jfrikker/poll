use std::thread::sleep;
use std::time::{Duration, Instant};

pub struct Timer {
    interval_ms: u64,
    last: Option<Instant>
}

impl Timer {
    pub fn new(interval_ms: u64) -> Timer {
        Timer {
            interval_ms: interval_ms,
            last: None
        }
    }

    pub fn wait(&mut self) {
        let now = Instant::now();
        self.last = Some(match self.last {
            Some(l) => {
                let elapsed = to_millis(&now.duration_since(l));
                if elapsed < self.interval_ms {
                    sleep(Duration::from_millis(self.interval_ms - elapsed));
                    Instant::now()
                } else {
                    now
                }
            },
            None => now
        });
    }
}

fn to_millis(d: &Duration) -> u64 {
    (d.as_secs() * 1000) + (d.subsec_nanos() / 1000000) as u64
}
