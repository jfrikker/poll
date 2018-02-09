use std::env;
use std::process;
use std::time::{Duration, Instant};
use std::thread::sleep;

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut stream = diffing(with_interval(1000, cmd_exit_code(args[1..].to_vec())));

    loop {
        print!("{}\n", stream.next());
    }
}

trait CmdStream<T> {
    fn next(&mut self) -> T;
}

struct CmdExitCode {
    args: Vec<String>
}

fn cmd_exit_code(args: Vec<String>) -> CmdExitCode {
    CmdExitCode {
        args : args
    }
}

impl CmdStream<String> for CmdExitCode {
    fn next(&mut self) -> String {
        let exit = process::Command::new(&self.args[0])
            .args(&self.args[1..])
            .stdout(process::Stdio::null())
            .stderr(process::Stdio::null())
            .status()
            .unwrap();

        if exit.success() {
            String::from("Success")
        } else {
            format!("Failed ({})", exit.code()
                .unwrap_or(1000))
        }
    }
}

struct Timer<S> {
    underlying: S,
    interval_ms: u64,
    last: Option<Instant>
}

fn with_interval<T, S: CmdStream<T>>(interval_ms: u64, underlying: S) -> Timer<S> {
    Timer {
        underlying: underlying,
        interval_ms: interval_ms,
        last: None
    }
}

impl <T, S: CmdStream<T>> CmdStream<T> for Timer<S> {
    fn next(&mut self) -> T {
        let now = Instant::now();
        self.last = Some(match self.last {
            Some(l) => {
                let elapsed = to_millis(now.duration_since(l));
                if elapsed < self.interval_ms {
                    sleep(Duration::from_millis(self.interval_ms - elapsed));
                    Instant::now()
                } else {
                    now
                }
            },
            None => now
        });

        self.underlying.next()
    }
}

fn to_millis(d: Duration) -> u64 {
    (d.as_secs() * 1000) + (d.subsec_nanos() / 1000000) as u64
}

struct Diff<S, T> {
    underlying: S,
    last: Option<T>
}

fn diffing<T, S: CmdStream<T>>(underlying: S) -> Diff<S, T> {
    Diff {
        underlying: underlying,
        last: None
    }
}

impl <T: Eq + Clone + std::fmt::Display, S: CmdStream<T>> CmdStream<T> for Diff<S, T> {
    fn next(&mut self) -> T {
        let mut done = false;
        while !done {
            let next = self.underlying.next();
            match self.last {
                Some(ref l) => {
                    if next != *l {
                        done = true;
                    }
                },
                None => {}
            }

            self.last = Some(next);
        }

        self.last.as_ref().unwrap().clone()
    }
}
