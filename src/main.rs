use std::io::{BufRead as _, BufReader};
use std::net::TcpListener;

mod either;
mod parser;

use crate::parser::{Parser as _, TermParser};

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

fn main() -> std::io::Result<()> {
    // tcp()?;

    let parser = TermParser::new(b"hello").or(TermParser::new(b"stuff"));
    dbg!(parser.parse(b"hello world!"));
    dbg!(parser.parse(b"stuff and things"));
    dbg!(parser.parse(b"things and stuff"));

    Ok(())
}
