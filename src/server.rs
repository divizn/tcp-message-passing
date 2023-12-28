use std::io::{Read, Write};
use std::env::args;
use std::net::{TcpListener, TcpStream, SocketAddr};
use std::thread;

const PORT: &str = "6969";


fn main() {
    let ip = args().nth(1).unwrap() + ":" + PORT;

    println!("{ip}");

    let server = match TcpListener::bind(ip) {
        Ok(server) => server,
        Err(eip) => panic!("Incorrect socket address configuration! Please change IP, current IP {eip}")
    };

    for connection in server.incoming() {
        match connection {
            Ok(stream) => {
                let addr = stream.peer_addr().expect("Unable to read peer address");
                thread::spawn(move || handle_connection(stream, addr));
            },
            Err(e) => panic!("{e}"),
        }
    }

}

fn handle_connection(mut stream: TcpStream, addr: SocketAddr) {
    println!("Accepted connection from: {addr}");

    let mut buffer = [0; 1024];
    loop {
        match stream.read(&mut buffer) {
            Ok(0) => {println!("Connection has been closed by {addr}"); break},
            Ok(n) => {
                // Handle the data (e.g., echo it back)
                stream.write_all(&buffer[0..n]).unwrap();
            }
            Err(e) => {
                eprintln!("Error reading from socket: {}", e);
                break;
            }
        }
    }
}
