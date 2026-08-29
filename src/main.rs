use std::net::TcpListener;

fn main() -> std::io::Result<()> {
    let address = "localhost:3000";
    let listener = TcpListener::bind(address)?;
    println!("Listening on {address}");

    for stream in listener.incoming() {
        let stream = stream?;

        let connector = stream.peer_addr()?;
        println!("Talking to: {}", connector);
    }

    Ok(())
}
