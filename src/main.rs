use std::io::{BufRead as _, BufReader};
use std::net::TcpListener;

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

    let hello_p = TermParser::new(b"hello")
        .inspect()
        .map(|term| term.to_ascii_uppercase());
    let outcome = hello_p.parse(b"hello world!");

    println!("We got: {:?}", outcome);

    Ok(())
}
