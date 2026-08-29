use std::fmt::Debug;
use std::io::{BufRead as _, BufReader};
use std::net::TcpListener;

#[allow(unused)]
fn tcp() -> std::io::Result<()> {
    let address = "localhost:3000";
    let listener = TcpListener::bind(address)?;
    println!("Listening on {address}");

    for stream in listener.incoming() {
        let stream = stream?;

        let connector = stream.peer_addr()?;
        println!("Talking to: {}", connector);

        let mut reader = BufReader::new(stream.try_clone()?);

        let mut line = String::new();
        loop {
            reader.read_line(&mut line)?;
            print!("{line}");

            if line == "\r\n" {
                break;
            }
            line.clear();
        }
        println!("Finished reading!");
    }

    Ok(())
}

#[derive(PartialEq, Eq)]
enum ParseResult<'a, T> {
    Found { subject: T, rest: &'a [u8] },
    Missed { rest: &'a [u8] },
}

impl<'a, T: Debug> Debug for ParseResult<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Found { subject, rest } => f
                .debug_struct("ParseResult::Found")
                .field("subject", subject)
                .field(
                    "rest",
                    &std::str::from_utf8(rest).unwrap_or("non-utf8 bytes"),
                )
                .finish(),
            Self::Missed { rest } => f
                .debug_struct("ParseResult::Missed")
                .field(
                    "rest",
                    &std::str::from_utf8(rest).unwrap_or("non-utf8 bytes"),
                )
                .finish(),
        }
    }
}

fn term_p<'i, 't>(input: &'i [u8], term: &'t [u8]) -> ParseResult<'i, &'t [u8]> {
    if input.starts_with(term) {
        ParseResult::Found {
            subject: term,
            rest: &input[term.len()..],
        }
    } else {
        ParseResult::Missed { rest: input }
    }
}

fn main() -> std::io::Result<()> {
    // tcp()?;

    let outcome = term_p(b"hello world", b"hello");

    println!("We got: {:?}", outcome);

    Ok(())
}
