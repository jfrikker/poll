#[macro_use]
extern crate clap;

extern crate time;

use std::ffi::{OsStr, OsString};
use std::io::{Error, Write, stdout, stderr};
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::process;
use std::time::{Duration, Instant};
use std::thread::sleep;

fn main() {
    let matches = 
        clap_app!(poll =>
            (version: "1.0")
            (about: "Runs a command repeatedly on some interval. Watches for changes to the output, and reacts to them.")
            (@setting TrailingVarArg)
            (@arg TIMESTAMP: -t --timestamp "Print timestamps")
            (@arg EXIT_CODE: -x --exit_code "Poll exit code, not stdout")
            (@arg UNTIL_SUCCESS: -u --until_success requires[EXIT_CODE] "Exit on success")
            (@arg UNTIL_FAILURE: -f --until_failure requires[EXIT_CODE] conflicts_with[UNTIL_SUCCESS] "Exit on failure")
            (@arg SHELL: -s --shell "Expect a single argument, which will be run in a shell")
            (@arg QUIET: -q --quiet "Suppress output")
            (@arg INTERVAL: -i --interval +takes_value "Polling interval, in seconds (default 1 second)")
            (@arg CMD: ... * "Command to run")
        ).get_matches();
    
    match do_loop(&matches) {
        Ok(x) => x,
        Err(err) => {
            let mut stderr = stderr();
            write!(&mut stderr, "{}\n", err.to_string()).unwrap();
            process::exit(1);
        }
    }
}

fn do_loop(matches: &clap::ArgMatches) -> Result<(), Error> {
    let print_timestamp = matches.is_present("TIMESTAMP");
    let use_code = matches.is_present("EXIT_CODE");
    let until_success = matches.is_present("UNTIL_SUCCESS");
    let until_failure = matches.is_present("UNTIL_FAILURE");
    let quiet = matches.is_present("QUIET");
    let use_shell = matches.is_present("SHELL");

    let interval_sec: u64 = matches.value_of("INTERVAL")
        .map_or(1, |s| s.parse().unwrap());

    let args: Vec<&OsStr> = if use_shell {
        vec!(OsStr::new("sh"), OsStr::new("-c"), matches.value_of_os("CMD").unwrap())
    } else {
        matches.values_of_os("CMD").unwrap().collect()
    };

    let mut timer = Timer::new(interval_sec * 1000);
    let mut last = OsString::new();

    loop {
        timer.wait();
        let cmd_result = if use_code {
            let status = try!(exit_code(&args));

            if status.success() {
                if until_success {
                    return Ok(());
                }

                OsString::from("Success\n")
            } else {
                if until_failure {
                    return Ok(());
                }

                OsString::from(format!("Failed ({})\n", status.code()
                    .unwrap_or(1000)))
            }
        } else {
            output(&args).unwrap()
        };

        if cmd_result == last {
            continue;
        }

        if !quiet {
            if print_timestamp {
                let timestamp = time::strftime("%F %H:%M:%S", &time::now()).unwrap();
                print!("{} - ", timestamp);
            }

            try!(stdout().write(cmd_result.as_os_str().as_bytes()));
            try!(stdout().flush());
        }

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

fn exit_code(cmd: &Vec<&OsStr>) -> Result<process::ExitStatus, Error> {
    process::Command::new(&cmd[0])
        .args(&cmd[1..])
        .stdout(process::Stdio::null())
        .stderr(process::Stdio::null())
        .status()
}

fn output(cmd: &Vec<&OsStr>) -> Result<OsString, Error> {
    process::Command::new(&cmd[0])
        .args(&cmd[1..])
        .stderr(process::Stdio::null())
        .output()
        .map(|res| OsString::from_vec(res.stdout))
}
