use std::io;

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
    }
}
