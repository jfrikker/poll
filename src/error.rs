use std::io;
use std::num::ParseIntError;

use time;

quick_error! {
    #[derive(Debug)]
    pub enum PollError {
        Io(err: io::Error) {
            cause(err)
            display("{}", err)
            description(err.description())
            from()
        }
        TimeParse(err: time::ParseError) {
            cause(err)
            display("{}", err)
            description(err.description())
            from()
        }
        IntParse(err: ParseIntError) {
            cause(err)
            display("{}", err)
            description(err.description())
            from()
        }
    }
}
