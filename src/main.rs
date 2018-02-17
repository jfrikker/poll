#[macro_use] extern crate quick_error;
#[macro_use] extern crate clap;
extern crate time;
extern crate sha1;

mod error;
mod timer;

use std::ffi::{OsStr, OsString};
use std::io::{self, Write, stdout, stderr};
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::process;
use std::time::Duration;

use error::PollError;
use timer::Timer;

fn main() {
    let matches = 
        clap_app!(poll =>
            (version: "1.0.1")
            (about: "Runs a command repeatedly on some interval. Watches for changes to the output, and reacts to them.")
            (@setting TrailingVarArg)
            (@arg TIMESTAMP: -t --timestamp "Print timestamps")
            (@arg TIMESTAMP_FORMAT: --ts_format +takes_value requires[TIMESTAMP] "The format to use for timestamps (strftime format). Defaults to \"%F %H:%M:%S\"")
            (@arg EXIT_CODE: -x --exit_code "Poll exit code, not stdout")
            (@arg UNTIL_SUCCESS: -u --until_success requires[EXIT_CODE] "Exit on success")
            (@arg UNTIL_FAILURE: -f --until_failure requires[EXIT_CODE] conflicts_with[UNTIL_SUCCESS] "Exit on failure")
            (@arg SHELL: -s --shell "Expect a single argument, which will be run in a shell")
            (@arg QUIET: -q --quiet "Suppress output")
            (@arg INTERVAL: -i --interval +takes_value "Polling interval, in seconds (default 1 second)")
            (@arg RUN_CMD: -r --run +takes_value "Command to run (in a shell) for each line of output")
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

fn do_loop(matches: &clap::ArgMatches) -> Result<(), PollError> {
    let print_timestamp = matches.is_present("TIMESTAMP");
    let timestamp_format = matches.value_of("TIMESTAMP_FORMAT").unwrap_or("%F %H:%M:%S");
    let use_code = matches.is_present("EXIT_CODE");
    let until_success = matches.is_present("UNTIL_SUCCESS");
    let until_failure = matches.is_present("UNTIL_FAILURE");
    let quiet = matches.is_present("QUIET");
    let use_shell = matches.is_present("SHELL");

    let interval_sec: u64 = try!(
        matches.value_of("INTERVAL").map_or(Ok(1), |s| s.parse()));

    let args: Vec<&OsStr> = if use_shell {
        vec!(OsStr::new("sh"), OsStr::new("-c"), matches.value_of_os("CMD").unwrap())
    } else {
        matches.values_of_os("CMD").unwrap().collect()
    };

    let run_cmd = matches.value_of_os("RUN_CMD");

    let mut timer = Timer::new(Duration::from_secs(interval_sec));
    let mut last = None;

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
            try!(output(&args))
        };

        let digest = Some(hash(cmd_result.as_os_str()));

        if digest == last {
            continue;
        }

        if !quiet {
            if print_timestamp {
                let timestamp = try!(time::strftime(timestamp_format, &time::now()));
                print!("{} - ", timestamp);
            }

            try!(stdout().write(cmd_result.as_os_str().as_bytes()));

            if !ends_with_newline(&cmd_result) {
                try!(write!(stdout(), "\n"));
            }

            try!(stdout().flush());
        }

        run_cmd.map(|cmd| do_run_cmd(cmd, cmd_result.as_os_str()));

        last = digest
    }
}

fn exit_code(cmd: &Vec<&OsStr>) -> Result<process::ExitStatus, io::Error> {
    process::Command::new(&cmd[0])
        .args(&cmd[1..])
        .stdout(process::Stdio::null())
        .stderr(process::Stdio::null())
        .status()
}

fn output(cmd: &Vec<&OsStr>) -> Result<OsString, io::Error> {
    process::Command::new(&cmd[0])
        .args(&cmd[1..])
        .stderr(process::Stdio::null())
        .output()
        .map(|res| OsString::from_vec(res.stdout))
}

fn do_run_cmd(cmd: &OsStr, output: &OsStr) -> Result<process::ExitStatus, io::Error> {
    let mut child = try!(process::Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .stdin(process::Stdio::piped())
        .spawn());
    {
        let stdin = child.stdin.as_mut().unwrap();
        try!(stdin.write(output.as_bytes()));
    }
    child.wait()
}

fn hash(data: &OsStr) -> sha1::Digest {
    let mut hasher = sha1::Sha1::new();
    hasher.update(data.as_bytes());
    hasher.digest()
}

fn ends_with_newline(string: &OsStr) -> bool {
    let as_string = string.to_string_lossy();
    as_string.chars().last() == Some('\n')
}
