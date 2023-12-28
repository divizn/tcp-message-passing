use std::io::{Read, Write};
use std::env::args;
use std::net::{TcpListener, TcpStream, SocketAddr};
use std::sync::{Arc, Mutex};
use std::thread;

const PORT: &str = "6969";


fn main() {
    let ip = args().nth(1).unwrap() + ":" + PORT;

    println!("{ip}");

    let connections: Arc<Mutex<Vec<TcpStream>>> = Arc::new(Mutex::new(Vec::new()));

    let server = match TcpListener::bind(ip) {
        Ok(server) => server,
        Err(eip) => panic!("Incorrect socket address configuration! Please change IP, current IP {eip}")
    };

    for connection in server.incoming() {
        match connection {
            Ok(stream) => {
                let addr = stream.peer_addr().expect("Unable to read peer address");
                let connections_clone = Arc::clone(&connections);
                thread::spawn(move || {
                    handle_connection(stream, addr, connections_clone);
                }); //maybe rc with all streams so i can send message from one to all
            },
            Err(e) => panic!("{e}"),
        }
    }

}

fn handle_connection(mut stream: TcpStream, addr: SocketAddr, connections: Arc<Mutex<Vec<TcpStream>>>) {
    println!("Accepted connection from: {addr}");

    {
        connections.lock().unwrap().push(stream.try_clone().expect("Could not clone client stream"));
    }

    let mut buffer = [0; 1024];
    loop {
        match stream.read(&mut buffer) {
            Ok(0) => {println!("Connection has been closed by {addr}"); break},
            Ok(n) => {
                let inp = &buffer[0..n].to_ascii_lowercase();
                // Handle the data (e.g., echo it back)
                println!("Received {inp:?} from {addr}");
                let streams = connections.lock().expect("Unable to lock streams");
                for mut stream in streams.iter() {
                    if stream.peer_addr().unwrap() != addr {
                        stream.write_all(&buffer[0..n]).unwrap();
                    }
                }
            }
            Err(e) => {
                eprintln!("Error reading from socket: {}", e);
                break;
            }
        }
    }

    {
        connections.lock().unwrap().retain(|stream| {
            stream.peer_addr().expect("unable to get client address") != addr
        })
    }
}
