use std::net::TcpListener;

fn main() -> std::io::Result<()> {
    let address = "localhost:3000";
    let listener = TcpListener::bind(address)?;
    println!("Listening on {address}");

    for _stream in listener.incoming() {
        println!("A stream has been found!");
    }

    Ok(())
}
