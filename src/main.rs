#[macro_use]
extern crate clap;

extern crate time;

use std::ffi::OsStr;
use std::process;
use std::time::{Duration, Instant};
use std::thread::sleep;

fn main() {
    let matches = 
        clap_app!(myapp =>
            (version: "1.0")
            (author: "Joe Frikker")
            (about: "polls")
            (@setting TrailingVarArg)
            (@arg TIMESTAMP: -t --timestamp "Print timestamps")
            (@arg INTERVAL: -i --interval +takes_value "Polling interval, in seconds")
            (@arg CMD: ... * "Command to run")
        ).get_matches();

    let args: Vec<&OsStr> = matches.values_of_os("CMD").unwrap().collect();

    let interval_sec: u64 = matches.value_of("INTERVAL")
        .map_or(1, |s| s.parse().unwrap());

    let print_timestamp = matches.is_present("TIMESTAMP");

    let mut timer = Timer::new(interval_sec * 1000);
    let mut last = String::from("");

    loop {
        timer.wait();
        let cmd_result = exit_code(&args);
        if cmd_result == last {
            continue;
        }

        if print_timestamp {
            let timestamp = time::strftime("%F %H:%M:%S", &time::now()).unwrap();
            print!("{} - ", timestamp);
        }

        print!("{}\n", cmd_result);
        last = cmd_result
    }
}

struct Timer {
    interval_ms: u64,
    last: Option<Instant>
}

impl Timer {
    fn new(interval_ms: u64) -> Timer {
        Timer {
            interval_ms: interval_ms,
            last: None
        }
    }

    fn wait(&mut self) {
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

fn exit_code(cmd: &Vec<&OsStr>) -> String {
    let exit = process::Command::new(&cmd[0])
        .args(&cmd[1..])
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
